//! Story Loom SDK 错误 — 与 Go SDK 语义对齐

use thiserror::Error;

/// Story Loom SDK 错误
#[derive(Debug, Error)]
pub enum SdkError {
    #[error("storyloomsdk: unauthorized: {0}")]
    Unauthorized(String),

    #[error("storyloomsdk: not found: {0}")]
    NotFound(String),

    #[error("storyloomsdk: invalid input: {0}")]
    InvalidInput(String),

    #[error("storyloomsdk: conflict: {0}")]
    Conflict(String),

    #[error("storyloomsdk: server error: {0}")]
    Server(String),

    #[error("storyloomsdk: network error: {0}")]
    Network(String),

    #[error("storyloomsdk: api error ({code}): {message}")]
    Api { code: String, message: String },

    #[error("storyloomsdk: invalid response: {0}")]
    InvalidResponse(String),
}

impl SdkError {
    /// 业务错误码
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::NotFound(_) => "NOT_FOUND",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::Conflict(_) => "VERSION_CONFLICT",
            Self::Server(_) => "INTERNAL",
            Self::Network(_) => "NETWORK",
            Self::Api { .. } => "API",
            Self::InvalidResponse(_) => "INVALID_RESPONSE",
        }
    }
}

impl From<reqwest::Error> for SdkError {
    fn from(e: reqwest::Error) -> Self {
        Self::Network(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SdkError>;
