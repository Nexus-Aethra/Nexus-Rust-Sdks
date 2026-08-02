//! Client — 与 Nexus Story Loom 通信的主入口
//!
//! 线程安全: Client 实例可在多个任务间共享 (reqwest::Client 内部 Arc 克隆)。

use crate::errors::{Result, SdkError};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use std::time::Duration;

/// SDK 配置
#[derive(Debug, Clone)]
pub struct Config {
    /// Story Loom 服务地址, 例如 http://localhost:8081
    pub base_url: String,
    /// 调用方持有的 Bearer Token (Portal 签发)
    pub token: String,
    /// HTTP 超时, 默认 10 秒
    pub timeout: Duration,
}

impl Config {
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            token: token.into(),
            timeout: Duration::from_secs(10),
        }
    }
}

/// Story Loom SDK 客户端
#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    token: String,
    http: reqwest::Client,
}

impl Client {
    /// 创建客户端
    pub fn new(cfg: Config) -> Self {
        assert!(!cfg.base_url.is_empty(), "storyloomsdk: base_url is required");
        let http = reqwest::Client::builder()
            .timeout(cfg.timeout)
            .build()
            .expect("storyloomsdk: build http client");
        Self {
            base_url: cfg.base_url.trim_end_matches('/').to_string(),
            token: cfg.token,
            http,
        }
    }

    /// 更新 Token (同一实例复用于不同用户)
    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = token.into();
    }

    /// 当前 Token
    pub fn token(&self) -> &str {
        &self.token
    }

    // ──────────── 内部 HTTP ────────────

    async fn request_text(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<String> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.request(method, &url);
        if let Some(b) = body {
            req = req.json(&b);
        }
        if !self.token.is_empty() {
            req = req.bearer_auth(&self.token);
        }

        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        if status.is_client_error() || status.is_server_error() {
            return Err(map_http_error(status, &text));
        }
        Ok(text)
    }

    /// GET + 解析单个对象信封 {code, data}
    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let text = self.request_text(reqwest::Method::GET, path, None).await?;
        parse_envelope::<T>(&text)
    }

    /// GET + 解析列表信封 {code, data: {items}}
    pub(crate) async fn get_list<T: DeserializeOwned>(&self, path: &str) -> Result<Vec<T>> {
        let text = self.request_text(reqwest::Method::GET, path, None).await?;
        parse_list::<T>(&text)
    }

    /// POST + 解析单个对象信封
    pub(crate) async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T> {
        let text = self
            .request_text(reqwest::Method::POST, path, Some(body.clone()))
            .await?;
        parse_envelope::<T>(&text)
    }

    /// POST 无返回体
    pub(crate) async fn post_void(&self, path: &str, body: &serde_json::Value) -> Result<()> {
        self.request_text(reqwest::Method::POST, path, Some(body.clone()))
            .await?;
        Ok(())
    }

    /// DELETE
    pub(crate) async fn delete(&self, path: &str) -> Result<()> {
        self.request_text(reqwest::Method::DELETE, path, None).await?;
        Ok(())
    }
}

/// 解析 {code, data} 信封 → 单个对象
pub(crate) fn parse_envelope<T: DeserializeOwned>(text: &str) -> Result<T> {
    #[derive(serde::Deserialize)]
    struct Envelope {
        code: String,
        #[serde(default)]
        message: String,
        #[serde(default)]
        data: Option<serde_json::Value>,
    }
    let env: Envelope =
        serde_json::from_str(text).map_err(|e| SdkError::InvalidResponse(e.to_string()))?;
    if env.code != "OK" {
        return Err(SdkError::Api {
            code: env.code,
            message: env.message,
        });
    }
    match env.data {
        Some(v) if !v.is_null() => {
            serde_json::from_value(v).map_err(|e| SdkError::InvalidResponse(e.to_string()))
        }
        _ => Err(SdkError::InvalidResponse("empty data".into())),
    }
}

/// 解析 {code, data: {items}} 信封 → 列表
pub(crate) fn parse_list<T: DeserializeOwned>(text: &str) -> Result<Vec<T>> {
    #[derive(serde::Deserialize)]
    struct ListData {
        #[serde(default)]
        items: Option<serde_json::Value>,
    }
    #[derive(serde::Deserialize)]
    struct ListEnv {
        code: String,
        #[serde(default)]
        data: Option<ListData>,
    }
    let env: ListEnv =
        serde_json::from_str(text).map_err(|e| SdkError::InvalidResponse(e.to_string()))?;
    if env.code != "OK" {
        return Err(SdkError::Api {
            code: env.code,
            message: String::new(),
        });
    }
    match env.data.and_then(|d| d.items) {
        Some(v) if !v.is_null() => {
            serde_json::from_value(v).map_err(|e| SdkError::InvalidResponse(e.to_string()))
        }
        _ => Ok(Vec::new()),
    }
}

fn map_http_error(status: StatusCode, body: &str) -> SdkError {
    let mut code = String::new();
    if body.starts_with('{') {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
            code = v
                .get("code")
                .and_then(|c| c.as_str())
                .unwrap_or_default()
                .to_string();
        }
    }
    let message = body.to_string();

    match status {
        StatusCode::UNAUTHORIZED => SdkError::Unauthorized(message),
        StatusCode::NOT_FOUND => SdkError::NotFound(message),
        StatusCode::BAD_REQUEST => SdkError::InvalidInput(message),
        StatusCode::CONFLICT => SdkError::Conflict(message),
        _ if status.is_server_error() => SdkError::Server(message),
        _ => SdkError::Api { code, message },
    }
}
