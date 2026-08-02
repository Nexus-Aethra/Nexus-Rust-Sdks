//! JWKS 结构 (RFC 7517) 与 JWK → RSA 公钥解析

use jsonwebtoken::{Algorithm, DecodingKey};
use serde::{Deserialize, Serialize};

/// JWKS — Portal 公钥集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

/// JWK — 单个公钥
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwk {
    #[serde(rename = "kty")]
    pub key_type: String, // RSA / EC / OKP
    #[serde(rename = "kid")]
    pub key_id: String,
    #[serde(rename = "use", default)]
    pub use_: String, // sig
    #[serde(rename = "alg", default)]
    pub algorithm: String, // RS256
    #[serde(rename = "n", default)]
    pub n: String, // RSA modulus (base64url)
    #[serde(rename = "e", default)]
    pub e: String, // RSA exponent (base64url)
    #[serde(rename = "crv", default)]
    pub crv: String, // EC curve
    #[serde(rename = "x", default)]
    pub x: String,
    #[serde(rename = "y", default)]
    pub y: String,
}

impl Jwk {
    /// 校验算法是否为 RS256
    pub fn is_rsa(&self) -> bool {
        self.key_type == "RSA"
    }

    /// JWK → jsonwebtoken 解码密钥
    pub fn to_decoding_key(&self) -> crate::Result<DecodingKey> {
        if !self.is_rsa() {
            return Err(crate::SdkError::PortalInvalidResponse(format!(
                "unsupported kty: {}",
                self.key_type
            )));
        }
        // from_rsa_components 直接接收 base64url 编码的 modulus/exponent
        DecodingKey::from_rsa_components(&self.n, &self.e)
            .map_err(|e| crate::SdkError::PortalInvalidResponse(format!("rsa components: {e}")))
    }

    /// 该 JWK 支持的验证算法 (当前仅 RS256)
    pub fn algorithm(&self) -> crate::Result<Algorithm> {
        match self.algorithm.as_str() {
            "" | "RS256" => Ok(Algorithm::RS256),
            other => Err(crate::SdkError::PortalInvalidResponse(format!(
                "unsupported alg: {other}"
            ))),
        }
    }
}

/// 模块密钥对 (模块管理 API 返回)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleKey {
    pub id: String,
    #[serde(rename = "kid")]
    pub key_id: String,
    pub version: i64,
    pub public_key: String, // PEM
    pub private_key: String, // PEM, 仅生成时返回
    pub algorithm: String,
    pub issued_at: i64,
}
