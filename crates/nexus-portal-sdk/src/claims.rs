//! JWT Claims — 与 Portal 签发的用户 Token 对应

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// 用户身份信息 (JWT Claims 的业务子集)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// 用户唯一 ID (sub)
    #[serde(rename = "sub")]
    pub user_id: String,
    /// 用户名
    pub username: String,
    /// 角色列表
    #[serde(default)]
    pub roles: Vec<String>,
    /// Token 签发者 (标准 JWT 字段 `iss`; `issuer` 保留作兼容别名)
    #[serde(rename = "iss", alias = "issuer", default)]
    pub issuer: String,
    /// 签名密钥 ID (kid) — 由 JWT Header 注入，不在 Payload 中
    #[serde(default)]
    pub key_id: String,
    /// Token 类型: access / refresh / service
    #[serde(rename = "type", default)]
    pub token_type: String,
    /// 过期时间 (Unix seconds)
    pub exp: i64,
    /// 签发时间 (Unix seconds)
    pub iat: i64,
}

impl Claims {
    /// 是否已过期 (不含时钟偏差)
    pub fn is_expired(&self) -> bool {
        now_unix() >= self.exp
    }

    /// 是否为管理员
    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin")
    }

    /// 是否拥有指定角色
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
