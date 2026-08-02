//! Nexus Story Loom 官方 Rust SDK
//!
//! 提供世界 / 章节 / 节点 / 边 / 人物 / 钩子 的完整 API 封装。
//!
//! 示例:
//!
//! ```no_run
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! use nexus_storyloom_sdk::{Client, Config, inputs::CreateStoryInput};
//!
//! let client = Client::new(Config::new("http://localhost:8081", "your.token"));
//! let world = client
//!     .create_story(&CreateStoryInput {
//!         name: "Dragon's Quest".into(),
//!         ..Default::default()
//!     })
//!     .await?;
//! println!("world: {}", world.name);
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod entities;
pub mod errors;
pub mod inputs;
mod methods;

pub use client::{Client, Config};
pub use entities::*;
pub use errors::{Result, SdkError};
pub use inputs::*;
pub use methods::ListOptions;

#[cfg(test)]
mod tests {
    use super::client::{parse_envelope as parse_env, parse_list as parse_list_fn};
    use super::entities::StoryWorld;
    use super::errors::SdkError;

    #[test]
    fn parse_envelope_ok() {
        let text = r#"{"code":"OK","data":{"id":"w1","name":"Demo","user_id":"u1"}}"#;
        let w: StoryWorld = parse_env(text).unwrap();
        assert_eq!(w.id, "w1");
        assert_eq!(w.name, "Demo");
    }

    #[test]
    fn parse_envelope_api_error() {
        let text = r#"{"code":"NOT_FOUND","message":"no world"}"#;
        let r = parse_env::<StoryWorld>(text);
        assert!(matches!(r, Err(SdkError::Api { code, .. }) if code == "NOT_FOUND"));
    }

    #[test]
    fn parse_list_ok_and_empty() {
        let text = r#"{"code":"OK","data":{"items":[{"id":"c1","title":"A"}]}}"#;
        let list: Vec<super::Chapter> = parse_list_fn(text).unwrap();
        assert_eq!(list.len(), 1);

        // data 为 null → 空列表
        let text2 = r#"{"code":"OK","data":null}"#;
        let list2: Vec<super::Chapter> = parse_list_fn(text2).unwrap();
        assert!(list2.is_empty());
    }

    #[test]
    fn input_serialize_fields() {
        let input = super::CreateStoryInput {
            name: "Demo".into(),
            description: "desc".into(),
            cover_image_url: "http://x/c.png".into(),
            cover_pixels: vec!["abc".into()],
            visibility: "private".into(),
        };
        let v = serde_json::to_value(&input).unwrap();
        assert_eq!(v["name"], "Demo");
        assert_eq!(v["cover_image_url"], "http://x/c.png");
        assert_eq!(v["visibility"], "private");
    }
}
