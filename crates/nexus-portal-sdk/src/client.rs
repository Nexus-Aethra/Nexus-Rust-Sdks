//! Client — 与 Nexus Portal 通信的主入口
//!
//! 能力:
//!   - JWKS 自动拉取与缓存 (kid 未命中自动刷新重试)
//!   - 用户 Token 本地验证 (零网络, 默认校验 iss 匹配 "nexus-portal")
//!   - Token Portal 端验证 (/auth/verify)
//!   - 模块自身管理 (密钥轮转, HMAC 签名)
//!
//! 安全默认值 (CWE-918 / CWE-347 缓解):
//!   - 默认 HTTP 客户端 `redirect::Policy::none()`, 拒绝跟随 30x;
//!     阻止 portal_url 通过 302 跳到内网/IMDS 造成 SSRF。
//!   - `Options::expected_issuer` 默认 `Some("nexus-portal".into())`,
//!     `verify_token` 强制校验 JWT 的 `iss` 字段, 防止跨租户伪造 token。
//!   - `Options::expected_audience` 默认为 `None` (Portal 当前不签 aud);
//!     设为 `Some(...)` 后开启 aud 校验。

use crate::claims::Claims;
use crate::errors::{PortalError, Result, SdkError};
use crate::jwks::{Jwks, ModuleKey};
use crate::jwks_cache::JwksCache;
use jsonwebtoken::{decode_header, Algorithm, DecodingKey, Validation};
use std::time::Duration;
use tokio::sync::oneshot;

/// JWT 验证的 iss 默认值。Portal 当前所有 token 都签 `iss: "nexus-portal"`。
pub const DEFAULT_EXPECTED_ISSUER: &str = "nexus-portal";

/// SDK 配置项 (零值采用合理默认)
#[derive(Debug, Clone)]
pub struct Options {
    /// JWKS 缓存有效期, 默认 5 分钟
    pub jwks_cache_ttl: Duration,
    /// HTTP 请求超时, 默认 5 秒
    pub timeout: Duration,
    /// 时钟偏差容忍, 默认 30 秒
    pub max_clock_skew: Duration,
    /// 模块自身凭证 (可选, 用于模块管理 API)
    pub module: Option<ModuleCredentials>,
    /// JWT 必须匹配的 `iss` 字段值。默认 `Some(DEFAULT_EXPECTED_ISSUER.into())`。
    /// 设为 `None` 会**关闭** iss 校验 (不推荐)。
    pub expected_issuer: Option<String>,
    /// JWT 必须匹配的 `aud` 字段值。默认 `None` (Portal 当前不签 aud,
    /// 校验关闭)。一旦 Portal 引入 aud claim, 部署方应设此值。
    pub expected_audience: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            jwks_cache_ttl: Duration::from_secs(300),
            timeout: Duration::from_secs(5),
            max_clock_skew: Duration::from_secs(30),
            module: None,
            expected_issuer: Some(DEFAULT_EXPECTED_ISSUER.to_string()),
            expected_audience: None,
        }
    }
}

/// 模块调用 Portal 管理 API 的凭证
#[derive(Debug, Clone)]
pub struct ModuleCredentials {
    pub module_id: String,
    pub api_key: String,
    pub api_secret: String,
}

/// Portal SDK 客户端
pub struct Client {
    portal_url: String,
    http: reqwest::Client,
    cache: JwksCache,
    options: Options,
    stop_tx: Option<oneshot::Sender<()>>,
}

impl Client {
    /// 创建客户端 (默认配置)
    pub fn new(portal_url: &str) -> Self {
        Self::with_options(portal_url, Options::default())
    }

    /// 创建带自定义配置的客户端
    pub fn with_options(portal_url: &str, options: Options) -> Self {
        let http = Self::build_default_http(&options);
        Self::with_options_and_http_client(portal_url, options, http)
    }

    /// 创建客户端, 注入自定义 reqwest::Client
    ///
    /// 调用方负责 client 的 redirect / timeout / proxy 等安全配置。
    /// SDK 不会覆盖调用方注入的 redirect policy。
    pub fn with_http_client(portal_url: &str, http: reqwest::Client) -> Self {
        Self::with_options_and_http_client(portal_url, Options::default(), http)
    }

    /// 完整构造函数: 自定义 options + 自定义 http client
    pub fn with_options_and_http_client(
        portal_url: &str,
        options: Options,
        http: reqwest::Client,
    ) -> Self {
        assert!(!portal_url.is_empty(), "portalsdk: portal_url is required");
        let cache = JwksCache::new(portal_url.to_string(), http.clone(), options.jwks_cache_ttl);
        Self {
            portal_url: portal_url.trim_end_matches('/').to_string(),
            http,
            cache,
            options,
            stop_tx: None,
        }
    }

    /// SDK 默认的 reqwest::Client: 关闭 30x 跟随, 阻断 SSRF 跳板。
    fn build_default_http(options: &Options) -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(options.timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("portalsdk: build http client")
    }

    /// Portal 地址
    pub fn portal_url(&self) -> &str {
        &self.portal_url
    }

    /// Ping — 检查 Portal 可达 (拉取一次 JWKS)
    pub async fn ping(&self) -> Result<()> {
        self.cache.refresh().await
    }

    /// 本地验证用户 Token, 返回 Claims
    ///
    /// 流程: 解析 kid → 查缓存 → 命中验证 → 未命中刷新重试
    pub async fn verify_token(&self, token: &str) -> Result<Claims> {
        // 解析 Header 获取 kid
        let header = decode_header(token)
            .map_err(|e| SdkError::TokenMalformed(e.to_string()))?;
        let kid = header
            .kid
            .clone()
            .ok_or_else(|| SdkError::TokenMalformed("missing kid".into()))?;

        match self.cache.get(&kid).await {
            Some(key) => self.verify_with_key(token, &key),
            None => {
                // kid 未命中: 强制刷新再试一次
                self.cache.refresh().await?;
                let key = self
                    .cache
                    .get(&kid)
                    .await
                    .ok_or_else(|| SdkError::UnknownKeyId(kid.clone()))?;
                self.verify_with_key(token, &key)
            }
        }
    }

    fn verify_with_key(&self, token: &str, key: &DecodingKey) -> Result<Claims> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.leeway = self.options.max_clock_skew.as_secs();

        // iss 校验: 默认开启, Options.expected_issuer = Some("nexus-portal")
        // 时强制匹配。设 None 关闭 (不推荐, 仅兼容老部署过渡用)。
        if let Some(issuer) = self.options.expected_issuer.as_deref() {
            validation.set_issuer(&[issuer]);
        } else {
            // 与 jsonwebtoken 默认行为一致: 不做 iss 校验
        }

        // aud 校验: 默认关闭 (Portal 当前不签 aud), 显式设了才校验。
        if let Some(aud) = self.options.expected_audience.as_deref() {
            validation.set_audience(&[aud]);
        } else {
            validation.validate_aud = false;
        }

        let mut claims: Claims = jsonwebtoken::decode(token, key, &validation)
            .map_err(classify_error)?
            .claims;
        claims.key_id = decode_header(token)
            .map(|h| h.kid.clone().unwrap_or_default())
            .unwrap_or_default();
        Ok(claims)
    }

    /// 调用 Portal /auth/verify 端到端验证
    pub async fn verify_with_portal(&self, token: &str) -> Result<Claims> {
        #[derive(serde::Serialize)]
        struct Req<'a> {
            token: &'a str,
        }
        #[derive(serde::Deserialize)]
        struct Envelope {
            code: String,
            message: Option<String>,
            data: Option<serde_json::Value>,
        }
        #[derive(serde::Deserialize)]
        struct VerifyData {
            valid: bool,
            claims: Option<serde_json::Value>,
            error: Option<String>,
        }

        let url = format!("{}/api/v1/auth/verify", self.portal_url);
        let resp = self
            .http
            .post(&url)
            .json(&Req { token })
            .send()
            .await
            .map_err(|e| SdkError::PortalUnreachable(e.to_string()))?;
        let envelope: Envelope = resp
            .json()
            .await
            .map_err(|e| SdkError::PortalInvalidResponse(e.to_string()))?;

        if envelope.code != "OK" {
            return Err(SdkError::PortalError {
                code: envelope.code,
                message: envelope.message.unwrap_or_default(),
            });
        }
        let data: VerifyData = serde_json::from_value(envelope.data.unwrap_or_default())
            .map_err(|e| SdkError::PortalInvalidResponse(e.to_string()))?;
        if !data.valid {
            return Err(SdkError::PortalError {
                code: data.error.unwrap_or_else(|| "VERIFY_FAILED".into()),
                message: "verify failed".into(),
            });
        }
        serde_json::from_value(data.claims.unwrap_or_default())
            .map_err(|e| SdkError::PortalInvalidResponse(e.to_string()))
    }

    /// 获取当前所有活跃公钥 (直接拉取, 不走缓存)
    pub async fn get_jwks(&self) -> Result<Jwks> {
        self.cache.fetch().await
    }

    /// 轮转模块的 JWT 密钥对
    pub async fn rotate_key(&self, key_id: &str) -> Result<ModuleKey> {
        let creds = self
            .options
            .module
            .as_ref()
            .ok_or(SdkError::ModuleCredentialsMissing)?;

        let method = "POST";
        let path = format!("/api/v1/modules/{}/keys/{}/rotate", creds.module_id, key_id);
        let url = format!("{}{}", self.portal_url, path);
        let ts = crate::util::unix_now().to_string();

        // StringToSign = METHOD + "\n" + PATH + "\n" + TIMESTAMP + "\n" + sha256(body)
        let body_hash = crate::util::sha256_hex(b"{}");
        let string_to_sign = format!("{method}\n{path}\n{ts}\n{body_hash}");
        let signature = crate::util::hmac_sha256_hex(&creds.api_secret, &string_to_sign);

        let resp = self
            .http
            .post(&url)
            .header("X-Module-Key", &creds.api_key)
            .header("X-Module-Timestamp", &ts)
            .header("X-Module-Signature", &signature)
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(|e| SdkError::PortalUnreachable(e.to_string()))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| SdkError::PortalInvalidResponse(e.to_string()))?;

        if !status.is_success() {
            return Err(SdkError::PortalInvalidResponse(format!(
                "status {status}, body: {text}"
            )));
        }
        let body: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| SdkError::PortalInvalidResponse(e.to_string()))?;
        serde_json::from_value(body["key"].clone())
            .map_err(|e| SdkError::PortalInvalidResponse(e.to_string()))
    }

    /// 启动后台 JWKS 定时刷新
    pub fn start_refresh_loop(&mut self) {
        let (tx, rx) = oneshot::channel::<()>();
        self.stop_tx = Some(tx);
        let cache = self.cache.clone();
        let ttl = self.options.jwks_cache_ttl;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(ttl);
            let mut rx = rx;
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        if let Err(e) = cache.refresh().await {
                            eprintln!("[nexus-portal-sdk] jwks background refresh failed: {e}");
                        }
                    }
                    _ = &mut rx => break,
                }
            }
        });
    }

    /// 停止后台刷新协程
    pub fn close(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }
}

fn classify_error(e: jsonwebtoken::errors::Error) -> SdkError {
    use jsonwebtoken::errors::ErrorKind;
    match e.kind() {
        ErrorKind::ExpiredSignature => SdkError::TokenExpired,
        ErrorKind::ImmatureSignature => SdkError::TokenNotYetValid,
        ErrorKind::InvalidSignature => SdkError::SignatureInvalid,
        ErrorKind::InvalidAlgorithm => SdkError::SignatureInvalid,
        ErrorKind::InvalidIssuer => SdkError::InvalidIssuer,
        ErrorKind::InvalidAudience => SdkError::InvalidAudience,
        _ => SdkError::TokenMalformed(e.to_string()),
    }
}

#[allow(dead_code)]
fn _assert_portal_error(_: PortalError) {}
