# Nexus Rust SDKs

Nexus 平台各模块的官方 Rust SDK 集合。

每个子目录是一个独立的 crate（`workspace` 统一管理），对应 Go SDK 的功能并做 Rust 习惯化：

| Crate | 状态 | 模块 | 对应 Go SDK |
|-------|------|------|-------------|
| [`nexus-portal-sdk`](./crates/nexus-portal-sdk) | ✅ 可用 | Nexus Portal 认证网关 | `Nexus-Go-SDKs/portal` |
| [`nexus-storyloom-sdk`](./crates/nexus-storyloom-sdk) | ✅ 可用 | Nexus Story Loom 叙事引擎 | `Nexus-Go-SDKs/storyloom` |

## 快速开始

```bash
cargo build
cargo test
```

## Portal SDK

能力：JWKS 自动拉取与缓存（kid 未命中自动刷新）、Token 本地验证（零网络）、Portal 端验证、模块密钥轮转（HMAC 签名）。

```rust,no_run
use nexus_portal_sdk::Client;

let client = Client::new("http://portal:8080");
let claims = client.verify_token("your.jwt.token").await?;
println!("{} {}", claims.user_id, claims.username);

// 后台定时刷新 JWKS
let mut client = Client::new("http://portal:8080");
client.start_refresh_loop();
```

### Claims

| 方法 | 说明 |
|------|------|
| `is_expired()` | 是否过期（不含时钟偏差） |
| `is_admin()` | 是否管理员角色 |
| `has_role(r)` | 是否拥有指定角色 |

### 错误

`SdkError` 枚举（`thiserror`），语义对齐 Go SDK：

```
TokenMalformed / TokenExpired / TokenNotYetValid /
SignatureInvalid / UnknownKeyId / PortalUnreachable /
PortalInvalidResponse / ModuleCredentialsMissing / PortalError
```

## Story Loom SDK

能力：世界 / 章节 / 节点 / 边 / 章节边 / 人物 / 节点-人物关联 / 钩子 的完整 API。

```rust,no_run
use nexus_storyloom_sdk::{Client, Config, CreateStoryInput};

let client = Client::new(Config::new("http://localhost:8081", "your.token"));
let world = client
    .create_story(&CreateStoryInput {
        name: "Dragon's Quest".into(),
        ..Default::default()
    })
    .await?;

let chapters = client.list_chapters(&world.id).await?;
```

### 方法清单

| 领域 | 方法 |
|------|------|
| 世界 | `create_story` `get_story` `get_story_detail` `list_stories` `delete_story` |
| 章节 | `create_chapter` `list_chapters` `delete_chapter` |
| 节点 | `create_node` `get_node` `list_nodes` `delete_node` |
| 节点边 | `create_edge` `list_edges` `list_edges_by_chapter` `delete_edge` |
| 章节边 | `create_chapter_edge` `list_chapter_edges` `delete_chapter_edge` |
| 人物 | `create_character` `list_characters` `get_character` `delete_character` |
| 节点-人物 | `create_node_character` `list_node_characters` `delete_node_character` |
| 钩子 | `create_hook` `list_hooks` `toggle_hook` |

### 列表分页

```rust,no_run
let worlds = client
    .list_stories(&ListOptions::new().limit(20).offset(40))
    .await?;
```

### 错误

`SdkError`：`Unauthorized / NotFound / InvalidInput / Conflict / Server / Network / Api / InvalidResponse`。

## 版本

| Tag | 含义 |
|-----|------|
| `v0.1.0` | 首次发布（后续用 git tag 管理，遵循 SemVer） |

## 与 Go SDK 的差异

- Rust 用 `async/await`（tokio）而非 context 参数
- 输入类型为 `Create*Input`，实体为 `Deserialize`（带 `serde(default)` 容错）
- 错误用 `thiserror` 枚举，`code()` 方法与 Go 的业务码对应
- `get_story_detail` 返回 `StoryDetail` 聚合结构（story+chapters+nodes+characters+edges+hooks）

## 贡献

```bash
cargo build
cargo test
cargo clippy -- -D warnings
```
