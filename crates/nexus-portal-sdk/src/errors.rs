//! SDK 错误类型 — 与 Go SDK 错误语义对齐

use thiserror::Error;

/// Portal SDK 错误
#[derive(Debug, Error)]
pub enum SdkError {
    #[error("portalsdk: token malformed: {0}")]
    TokenMalformed(String),

    #[error("portalsdk: token expired")]
    TokenExpired,

    #[error("portalsdk: token not yet valid")]
    TokenNotYetValid,

    #[error("portalsdk: signature invalid")]
    SignatureInvalid,

    #[error("portalsdk: token issuer mismatch (expected one of the configured issuers)")]
    InvalidIssuer,

    #[error("portalsdk: token audience mismatch (expected one of the configured audiences)")]
    InvalidAudience,

    #[error("portalsdk: unknown key id: {0}")]
    UnknownKeyId(String),

    #[error("portalsdk: portal unreachable: {0}")]
    PortalUnreachable(String),

    #[error("portalsdk: invalid response from portal: {0}")]
    PortalInvalidResponse(String),

    #[error("portalsdk: module credentials required")]
    ModuleCredentialsMissing,

    #[error("portalsdk: portal error: {code}: {message}")]
    PortalError { code: String, message: String },

    #[error("portalsdk: io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("portalsdk: json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl SdkError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::TokenMalformed(_) => "TOKEN_MALFORMED",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::TokenNotYetValid => "TOKEN_NOT_YET_VALID",
            Self::SignatureInvalid => "SIGNATURE_INVALID",
            Self::InvalidIssuer => "INVALID_ISSUER",
            Self::InvalidAudience => "INVALID_AUDIENCE",
            Self::UnknownKeyId(_) => "UNKNOWN_KEY_ID",
            Self::PortalUnreachable(_) => "PORTAL_UNREACHABLE",
            Self::PortalInvalidResponse(_) => "PORTAL_INVALID_RESPONSE",
            Self::ModuleCredentialsMissing => "MODULE_CREDENTIALS_MISSING",
            Self::PortalError { .. } => "PORTAL_ERROR",
            Self::Io(_) => "IO",
            Self::Json(_) => "JSON",
        }
    }
}

/// Portal 返回的业务错误
#[derive(Debug, Clone)]
pub struct PortalError {
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for PortalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "portalsdk: portal error: {}: {}", self.code, self.message)
    }
}

impl std::error::Error for PortalError {}

pub type Result<T> = std::result::Result<T, SdkError>;
