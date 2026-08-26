//! Nexus Portal 官方 Rust SDK
//!
//! 主要能力:
//!   - JWKS 自动拉取与缓存 (kid 未命中自动刷新重试)
//!   - 用户 Token 本地验证 (零网络)
//!   - Token Portal 端验证 (/auth/verify)
//!   - 模块自身管理 (密钥轮转, HMAC 签名)
//!
//! 示例:
//!
//! ```no_run
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! use nexus_portal_sdk::Client;
//!
//! let client = Client::new("http://portal:8080");
//! let claims = client.verify_token("your.jwt.token").await?;
//! println!("{}", claims.username);
//! # Ok(())
//! # }
//! ```

pub mod claims;
pub mod client;
pub mod errors;
pub mod jwks;
pub mod jwks_cache;
mod util;

pub use claims::Claims;
pub use client::{Client, ModuleCredentials, Options};
pub use errors::{PortalError, Result, SdkError};
pub use jwks::{Jwk, Jwks, ModuleKey};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claims_roles() {
        let c = Claims {
            user_id: "u1".into(),
            username: "admin".into(),
            roles: vec!["user".into(), "admin".into()],
            issuer: "portal".into(),
            key_id: "k1".into(),
            token_type: "access".into(),
            exp: 1_000_000,
            iat: 1,
        };
        assert!(c.is_admin());
        assert!(c.has_role("user"));
        assert!(!c.has_role("mod"));
        assert!(c.is_expired()); // exp 远早于现在
    }

    #[test]
    fn claims_not_expired() {
        let future = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600) as i64;
        let c = Claims {
            user_id: "u1".into(),
            username: "a".into(),
            roles: vec![],
            issuer: "portal".into(),
            key_id: "k1".into(),
            token_type: "access".into(),
            exp: future,
            iat: 0,
        };
        assert!(!c.is_expired());
    }

    #[test]
    fn jwk_algorithm() {
        let jwk = Jwk {
            key_type: "RSA".into(),
            key_id: "k1".into(),
            use_: "sig".into(),
            algorithm: "RS256".into(),
            n: "abc".into(),
            e: "AQAB".into(),
            crv: String::new(),
            x: String::new(),
            y: String::new(),
        };
        assert!(jwk.is_rsa());
        assert_eq!(jwk.algorithm().unwrap(), jsonwebtoken::Algorithm::RS256);
    }

    #[test]
    fn claims_deserialize_portal_jwt_fields() {
        let claims: Claims = serde_json::from_value(serde_json::json!({
            "iss": "nexus-portal",
            "sub": "user-1",
            "username": "alice",
            "roles": ["user"],
            "type": "access",
            "exp": 2_000_000_000,
            "iat": 1_900_000_000
        }))
        .unwrap();
        assert_eq!(claims.issuer, "nexus-portal");
        assert_eq!(claims.user_id, "user-1");
        assert!(claims.key_id.is_empty());
    }

    #[test]
    fn claims_deserialize_portal_verify_response_fields() {
        let claims: Claims = serde_json::from_value(serde_json::json!({
            "sub": "user-1",
            "username": "alice",
            "roles": [],
            "type": "access",
            "exp": 2_000_000_000,
            "iat": 1_900_000_000
        }))
        .unwrap();
        assert_eq!(claims.issuer, "");
        assert_eq!(claims.key_id, "");
    }
}
