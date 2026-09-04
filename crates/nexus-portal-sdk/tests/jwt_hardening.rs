//! Integration tests for JWT hardening:
//!   - issuer mismatch is rejected
//!   - audience mismatch is rejected (when configured)
//!   - default Options uses expected_issuer = "nexus-portal"
//!   - SDK's default http client does NOT follow 30x (SSRF protection)
//!
//! Each test stands up a tiny in-process axum server that:
//!   1. Generates an RSA key pair.
//!   2. Exposes /jwks.json with the public key (JWK).
//!   3. Lets the test sign arbitrary JWTs against the private key.

use axum::{response::Redirect, routing::get, Json, Router};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use nexus_portal_sdk::{Client, Options, SdkError};
use rsa::pkcs8::EncodePrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;

#[derive(Clone)]
struct MockPortal {
    n_b64: String,
    e_b64: String,
    jwks_hits: Arc<std::sync::atomic::AtomicUsize>,
}

#[derive(Serialize)]
struct Jwk {
    kty: &'static str,
    kid: &'static str,
    #[serde(rename = "use")]
    use_: &'static str,
    alg: &'static str,
    n: String,
    e: String,
}

#[derive(Serialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

fn base64url_no_pad(bytes: &[u8]) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    URL_SAFE_NO_PAD.encode(bytes)
}

async fn spawn_mock_portal() -> (SocketAddr, Arc<std::sync::atomic::AtomicUsize>, RsaPrivateKey) {
    let mut rng = rand::thread_rng();
    let private = RsaPrivateKey::new(&mut rng, 2048).expect("generate rsa key");
    let public = rsa::RsaPublicKey::from(&private);
    let n_b64 = base64url_no_pad(public.n().to_bytes_be().as_slice());
    let e_b64 = base64url_no_pad(public.e().to_bytes_be().as_slice());

    let jwks_hits = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let portal = MockPortal {
        n_b64,
        e_b64,
        jwks_hits: jwks_hits.clone(),
    };

    let app = Router::new().route(
        "/jwks.json",
        get(move || {
            let portal = portal.clone();
            async move {
                portal
                    .jwks_hits
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Json(Jwks {
                    keys: vec![Jwk {
                        kty: "RSA",
                        kid: "test-kid",
                        use_: "sig",
                        alg: "RS256",
                        n: portal.n_b64.clone(),
                        e: portal.e_b64.clone(),
                    }],
                })
            }
        }),
    );

    let ready = Arc::new(Notify::new());
    let ready_in = ready.clone();
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    tokio::spawn(async move {
        // Signal that the spawned task has been scheduled, BEFORE
        // axum::serve starts its lazy accept loop. This combined
        // with `wait_until_reachable` is the only reliable handshake
        // for `tokio::spawn`ed servers in `#[tokio::test]`.
        ready_in.notify_one();
        let _ = axum::serve(listener, app).await;
    });
    ready.notified().await;
    wait_until_reachable(addr).await;
    (addr, jwks_hits, private)
}

/// Spawn a mock that 302s /jwks.json to /does-not-exist (which returns
/// an empty keys list). The SDK must NOT follow the redirect.
async fn spawn_redirect_mock() -> SocketAddr {
    let redirect_app = Router::new()
        .route(
            "/jwks.json",
            get(|| async { Redirect::to("/does-not-exist") }),
        )
        .route(
            "/does-not-exist",
            get(|| async { axum::Json(serde_json::json!({"keys": []})) }),
        );
    let ready = Arc::new(Notify::new());
    let ready_in = ready.clone();
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    tokio::spawn(async move {
        ready_in.notify_one();
        let _ = axum::serve(listener, redirect_app).await;
    });
    ready.notified().await;
    wait_until_reachable(addr).await;
    addr
}

/// Poll-connect to the bound address until the listener accepts a
/// connection or the timeout elapses. This is the only reliable
/// "server is ready" handshake for `tokio::spawn`ed axum servers in
/// `#[tokio::test]` — `yield_now()` is not enough because axum's
/// accept loop only starts after several scheduler rounds.
async fn wait_until_reachable(addr: SocketAddr) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if TcpStream::connect(addr).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("server at {addr} did not accept connections within 2s");
}

fn sign_jwt(
    private: &RsaPrivateKey,
    kid: &str,
    iss: &str,
    aud: Option<&str>,
    exp_offset_secs: i64,
) -> String {
    use serde_json::json;
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());
    let pkcs8_pem = private
        .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
        .expect("pkcs8 pem");
    let key = EncodingKey::from_rsa_pem(pkcs8_pem.as_bytes()).expect("encoding key");
    let mut claims = json!({
        "iss": iss,
        "sub": "user-1",
        "username": "alice",
        "roles": ["user"],
        "type": "access",
        "exp": jsonwebtoken::get_current_timestamp() as i64 + exp_offset_secs,
        "iat": jsonwebtoken::get_current_timestamp() as i64,
    });
    if let Some(a) = aud {
        claims["aud"] = serde_json::Value::String(a.to_string());
    }
    encode(&header, &claims, &key).expect("sign jwt")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn default_options_reject_wrong_issuer() {
    let (addr, _hits, private) = spawn_mock_portal().await;
    let client = Client::new(&format!("http://{addr}"));

    let token = sign_jwt(&private, "test-kid", "evil-portal", None, 3600);
    let err = client.verify_token(&token).await.expect_err("must reject");
    assert!(
        matches!(err, SdkError::InvalidIssuer),
        "expected InvalidIssuer, got {err:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn default_options_accept_nexus_portal_issuer() {
    let (addr, _hits, private) = spawn_mock_portal().await;
    let client = Client::new(&format!("http://{addr}"));

    let token = sign_jwt(&private, "test-kid", "nexus-portal", None, 3600);
    let claims = client.verify_token(&token).await.expect("valid token must verify");
    assert_eq!(claims.issuer, "nexus-portal");
    assert_eq!(claims.user_id, "user-1");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expected_audience_rejects_mismatch() {
    let (addr, _hits, private) = spawn_mock_portal().await;
    let opts = Options {
        expected_audience: Some("nexus-study".to_string()),
        ..Options::default()
    };
    let client = Client::with_options(&format!("http://{addr}"), opts);

    let token = sign_jwt(&private, "test-kid", "nexus-portal", Some("not-study"), 3600);
    let err = client.verify_token(&token).await.expect_err("must reject");
    assert!(matches!(err, SdkError::InvalidAudience), "got {err:?}");

    let token = sign_jwt(&private, "test-kid", "nexus-portal", Some("nexus-study"), 3600);
    let claims = client.verify_token(&token).await.expect("must accept");
    assert_eq!(claims.username, "alice");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expected_audience_none_accepts_any_or_no_aud() {
    let (addr, _hits, private) = spawn_mock_portal().await;
    let client = Client::new(&format!("http://{addr}"));

    // No aud
    let token = sign_jwt(&private, "test-kid", "nexus-portal", None, 3600);
    let claims = client.verify_token(&token).await.expect("must accept");
    assert_eq!(claims.user_id, "user-1");

    // Some aud (ignored because expected_audience is None)
    let token = sign_jwt(&private, "test-kid", "nexus-portal", Some("anything"), 3600);
    let claims = client.verify_token(&token).await.expect("must accept");
    assert_eq!(claims.user_id, "user-1");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expected_issuer_can_be_disabled_explicitly() {
    let (addr, _hits, private) = spawn_mock_portal().await;
    let opts = Options {
        expected_issuer: None,
        ..Options::default()
    };
    let client = Client::with_options(&format!("http://{addr}"), opts);

    let token = sign_jwt(&private, "test-kid", "some-other-issuer", None, 3600);
    let claims = client
        .verify_token(&token)
        .await
        .expect("iss check disabled must accept");
    assert_eq!(claims.issuer, "some-other-issuer");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expired_token_rejected_even_with_correct_issuer() {
    let (addr, _hits, private) = spawn_mock_portal().await;
    let client = Client::new(&format!("http://{addr}"));
    let token = sign_jwt(&private, "test-kid", "nexus-portal", None, -3600);
    let err = client.verify_token(&token).await.expect_err("expired");
    assert!(matches!(err, SdkError::TokenExpired), "got {err:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn custom_http_client_with_redirect_none_skips_302() {
    // A 302-redirecting mock plus an injected reqwest client with
    // Policy::none(): the SDK must NOT follow the redirect, so
    // verify_token fails because the empty JWKS (or the 302 body)
    // does not yield the kid the token names.
    let redirect_addr = spawn_redirect_mock().await;
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("http client");
    let client = Client::with_http_client(&format!("http://{redirect_addr}"), http);

    let token = sign_jwt(
        &RsaPrivateKey::new(&mut rand::thread_rng(), 2048).unwrap(),
        "any-kid",
        "nexus-portal",
        None,
        3600,
    );
    let err = client.verify_token(&token).await.expect_err("must error");
    assert!(
        matches!(
            err,
            SdkError::PortalInvalidResponse(_) | SdkError::UnknownKeyId(_)
        ),
        "expected redirect-blocked error, got {err:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn default_http_client_blocks_302_even_without_injection() {
    // The SDK's *default* builder must apply redirect::Policy::none().
    // Use the default Client (no injection) against a 302-redirecting
    // mock; verify_token must fail.
    let redirect_addr = spawn_redirect_mock().await;
    let client = Client::new(&format!("http://{redirect_addr}"));
    let token = sign_jwt(
        &RsaPrivateKey::new(&mut rand::thread_rng(), 2048).unwrap(),
        "any-kid",
        "nexus-portal",
        None,
        3600,
    );
    let err = client.verify_token(&token).await.expect_err("must error");
    assert!(
        matches!(
            err,
            SdkError::PortalInvalidResponse(_) | SdkError::UnknownKeyId(_)
        ),
        "default SDK client must NOT follow 302, got {err:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn with_options_and_http_client_keeps_options() {
    // Verify the full constructor wires both the options and the
    // injected client together.
    let (addr, _hits, private) = spawn_mock_portal().await;
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("http");
    let opts = Options {
        expected_issuer: Some("nexus-portal".to_string()),
        expected_audience: None,
        ..Options::default()
    };
    let client =
        Client::with_options_and_http_client(&format!("http://{addr}"), opts, http);

    let token = sign_jwt(&private, "test-kid", "evil-portal", None, 3600);
    let err = client.verify_token(&token).await.expect_err("must reject");
    assert!(matches!(err, SdkError::InvalidIssuer), "got {err:?}");
}
