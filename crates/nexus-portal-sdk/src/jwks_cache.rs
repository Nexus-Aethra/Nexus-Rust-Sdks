//! JWKS 缓存 — 惰性拉取 + TTL 过期 + kid 未命中强制刷新

use crate::jwks::{Jwk, Jwks};
use crate::{Result, SdkError};
use jsonwebtoken::DecodingKey;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// JWKS 缓存
#[derive(Clone)]
pub struct JwksCache {
    portal_url: String,
    http: reqwest::Client,
    ttl: Duration,
    inner: Arc<RwLock<Inner>>,
}

struct Inner {
    keys: HashMap<String, DecodingKey>,
    last_fetched: Option<Instant>,
}

impl JwksCache {
    pub fn new(portal_url: String, http: reqwest::Client, ttl: Duration) -> Self {
        Self {
            portal_url,
            http,
            ttl,
            inner: Arc::new(RwLock::new(Inner {
                keys: HashMap::new(),
                last_fetched: None,
            })),
        }
    }

    /// 获取指定 kid 的公钥 (只读, 不触发网络)
    pub async fn get(&self, kid: &str) -> Option<DecodingKey> {
        let inner = self.inner.read().await;
        inner.keys.get(kid).cloned()
    }

    /// 判断是否过期需要刷新
    pub async fn needs_refresh(&self) -> bool {
        let inner = self.inner.read().await;
        match inner.last_fetched {
            Some(t) => t.elapsed() > self.ttl,
            None => true, // 从未拉取
        }
    }

    /// 强制刷新 JWKS (支持 ETag 条件请求)
    pub async fn refresh(&self) -> Result<()> {
        let url = format!("{}/jwks.json", self.portal_url);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| SdkError::PortalUnreachable(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
            // 304: 缓存仍然有效
            let mut inner = self.inner.write().await;
            inner.last_fetched = Some(Instant::now());
            return Ok(());
        }
        if !resp.status().is_success() {
            return Err(SdkError::PortalInvalidResponse(format!(
                "status {}",
                resp.status()
            )));
        }

        let jwks: Jwks = resp
            .json()
            .await
            .map_err(|e| SdkError::PortalInvalidResponse(e.to_string()))?;

        let mut new_keys = HashMap::new();
        for k in jwks.keys {
            match Self::parse(&k) {
                Ok(key) => {
                    new_keys.insert(k.key_id.clone(), key);
                }
                Err(e) => {
                    // 跳过无效 JWK, 与 Go SDK 一致
                    eprintln!("[nexus-portal-sdk] skip invalid jwk kid={}: {e}", k.key_id);
                }
            }
        }

        let mut inner = self.inner.write().await;
        inner.keys = new_keys;
        inner.last_fetched = Some(Instant::now());
        Ok(())
    }

    /// 直接拉取 JWKS (不走缓存, 用于 GetJwks)
    pub async fn fetch(&self) -> Result<Jwks> {
        let url = format!("{}/jwks.json", self.portal_url);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| SdkError::PortalUnreachable(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(SdkError::PortalInvalidResponse(format!(
                "status {}",
                resp.status()
            )));
        }
        resp.json()
            .await
            .map_err(|e| SdkError::PortalInvalidResponse(e.to_string()))
    }

    fn parse(k: &Jwk) -> Result<DecodingKey> {
        k.to_decoding_key()
    }
}
