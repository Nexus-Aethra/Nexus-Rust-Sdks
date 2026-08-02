//! Story Loom 实体类型 (与 Go SDK 对齐)

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// 反序列化 Vec, 容忍 `null` / 缺失 → 空数组 (后端可能返回 null)
pub(crate) fn vec_or_default<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<Vec<T>>::deserialize(d).map(|o| o.unwrap_or_default())
}

/// 反序列化 HashMap, 容忍 `null` / 缺失 → 空 map
pub(crate) fn map_or_default<'de, D, V>(d: D) -> Result<HashMap<String, V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    Option::<HashMap<String, V>>::deserialize(d).map(|o| o.unwrap_or_default())
}

/// 故事世界
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryWorld {
    #[serde(default)]
    pub id: String,
    #[serde(default, rename = "user_id")]
    pub user_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, rename = "cover_image_url")]
    pub cover_image_url: String,
    #[serde(default, rename = "cover_pixels")]
    pub cover_pixels: Vec<String>,
    #[serde(default)]
    pub visibility: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub status: String,
    #[serde(default, rename = "created_at")]
    pub created_at: Option<String>,
    #[serde(default, rename = "updated_at")]
    pub updated_at: Option<String>,
}

/// 章节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    #[serde(default)]
    pub id: String,
    #[serde(default, rename = "story_id")]
    pub story_id: String,
    #[serde(default, rename = "parent_chapter_id")]
    pub parent_chapter_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, rename = "scope_type")]
    pub scope_type: String,
    #[serde(default, rename = "scope_data", deserialize_with = "map_or_default")]
    pub scope_data: HashMap<String, serde_json::Value>,
    #[serde(default, rename = "is_begin")]
    pub is_begin: bool,
    #[serde(default, rename = "sort_order")]
    pub sort_order: i64,
    #[serde(default)]
    pub status: String,
    #[serde(default, rename = "created_at")]
    pub created_at: Option<String>,
    #[serde(default, rename = "updated_at")]
    pub updated_at: Option<String>,
}

/// 剧情节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    #[serde(rename = "chapter_id")]
    pub chapter_id: String,
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(rename = "type", default)]
    pub node_type: String,
    #[serde(default, rename = "position_x")]
    pub position_x: f64,
    #[serde(default, rename = "position_y")]
    pub position_y: f64,
    #[serde(default, deserialize_with = "map_or_default")]
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub version: i64,
    #[serde(default, rename = "created_at")]
    pub created_at: Option<String>,
    #[serde(default, rename = "updated_at")]
    pub updated_at: Option<String>,
}

/// 节点关系线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    #[serde(rename = "from_node_id")]
    pub from_node_id: String,
    #[serde(rename = "to_node_id")]
    pub to_node_id: String,
    #[serde(rename = "type", default)]
    pub edge_type: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub priority: i64,
    #[serde(default, deserialize_with = "map_or_default")]
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(default, rename = "created_at")]
    pub created_at: Option<String>,
    #[serde(default, rename = "updated_at")]
    pub updated_at: Option<String>,
}

/// 章节间有向边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterEdge {
    pub id: String,
    #[serde(rename = "story_id")]
    pub story_id: String,
    #[serde(rename = "from_chapter_id")]
    pub from_chapter_id: String,
    #[serde(rename = "to_chapter_id")]
    pub to_chapter_id: String,
    #[serde(rename = "type", default)]
    pub edge_type: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub priority: i64,
    #[serde(default, deserialize_with = "map_or_default")]
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(default, rename = "created_at")]
    pub created_at: Option<String>,
    #[serde(default, rename = "updated_at")]
    pub updated_at: Option<String>,
}

/// 人物
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    #[serde(rename = "story_id")]
    pub story_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, rename = "avatar_url")]
    pub avatar_url: String,
    #[serde(default, deserialize_with = "map_or_default")]
    pub personality: HashMap<String, serde_json::Value>,
    #[serde(default, rename = "voice_config", deserialize_with = "map_or_default")]
    pub voice_config: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub status: String,
    #[serde(default, rename = "created_at")]
    pub created_at: Option<String>,
    #[serde(default, rename = "updated_at")]
    pub updated_at: Option<String>,
}

/// 节点-人物关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCharacter {
    pub id: String,
    #[serde(rename = "node_id")]
    pub node_id: String,
    #[serde(rename = "character_id")]
    pub character_id: String,
    #[serde(default)]
    pub role: String,
    #[serde(default, rename = "dialogue_template")]
    pub dialogue_template: String,
    #[serde(default, rename = "action_hint")]
    pub action_hint: String,
    #[serde(default, rename = "sort_order")]
    pub sort_order: i64,
    #[serde(default, deserialize_with = "map_or_default")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 推进钩子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    pub id: String,
    #[serde(rename = "node_id")]
    pub node_id: String,
    #[serde(default, rename = "edge_id")]
    pub edge_id: Option<String>,
    pub name: String,
    #[serde(rename = "type", default)]
    pub hook_type: String,
    #[serde(default, rename = "condition_logic", deserialize_with = "map_or_default")]
    pub condition_logic: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, rename = "created_at")]
    pub created_at: Option<String>,
    #[serde(default, rename = "updated_at")]
    pub updated_at: Option<String>,
}

/// 故事完整详情
#[derive(Debug, Clone, Deserialize)]
pub struct StoryDetail {
    #[serde(default)]
    pub story: Option<StoryWorld>,
    #[serde(default, deserialize_with = "vec_or_default")]
    pub chapters: Vec<Chapter>,
    #[serde(default, deserialize_with = "vec_or_default")]
    pub nodes: Vec<Node>,
    #[serde(default, deserialize_with = "vec_or_default")]
    pub characters: Vec<Character>,
    #[serde(default, deserialize_with = "vec_or_default")]
    pub edges: Vec<Edge>,
    #[serde(default, rename = "chapter_edges", deserialize_with = "vec_or_default")]
    pub chapter_edges: Vec<ChapterEdge>,
    #[serde(default, deserialize_with = "vec_or_default")]
    pub hooks: Vec<Hook>,
}
