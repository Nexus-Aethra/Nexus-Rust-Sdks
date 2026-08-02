//! 端到端示例: 用 SDK 连接真实 Story Loom + Portal
//!
//! 运行: RUST_LOG=info cargo run --example quick

use nexus_storyloom_sdk::{Client, Config, CreateStoryInput, ListOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Portal 登录获取 token
    let token = login().await?;
    println!("[1] Portal token 获取成功 ({} chars)", token.len());

    // 2. 创建 SDK 客户端
    let client = Client::new(Config::new("http://localhost:8081", token));

    // 3. 创建世界
    let world = client
        .create_story(&CreateStoryInput {
            name: format!("Rust SDK Demo {}", chrono_name()),
            description: "由 Rust SDK 创建".into(),
            visibility: "private".into(),
            ..Default::default()
        })
        .await?;
    println!("[2] 创建世界: {} ({})", world.name, world.id);

    // 4. 列出世界
    let worlds = client.list_stories(&ListOptions::new().limit(5)).await?;
    println!("[3] 世界列表: {} 个", worlds.len());

    // 5. 获取详情
    let detail = client.get_story_detail(&world.id).await?;
    let begin_count = detail
        .chapters
        .iter()
        .filter(|c| c.is_begin)
        .count();
    println!(
        "[4] 世界详情: chapters={} begin_chapters={}",
        detail.chapters.len(),
        begin_count
    );

    println!("\n✅ Rust SDK 全链路验证通过");
    Ok(())
}

/// 通过 Portal admin 登录获取 access token
async fn login() -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::Client::new()
        .post("http://localhost:8080/admin/api/v1/auth/login")
        .json(&serde_json::json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await?;
    let body: serde_json::Value = resp.json().await?;
    let token = body["data"]["access_token"]
        .as_str()
        .ok_or("login failed: no access_token")?
        .to_string();
    Ok(token)
}

fn chrono_name() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("demo_{secs}")
}
