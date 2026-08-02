//! 创建输入类型 — 与实体分离, 避免调用方传 server 端字段

use serde::Serialize;
use std::collections::HashMap;

/// 创建故事参数
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateStoryInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, rename = "cover_image_url")]
    pub cover_image_url: String,
    #[serde(default, rename = "cover_pixels")]
    pub cover_pixels: Vec<String>,
    #[serde(default)]
    pub visibility: String, // private / unlisted / public
}

/// 创建章节参数
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateChapterInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, rename = "scope_type")]
    pub scope_type: String, // geographic / temporal / logical
    #[serde(default, rename = "scope_data")]
    pub scope_data: HashMap<String, serde_json::Value>,
    #[serde(default, rename = "sort_order")]
    pub sort_order: i64,
}

/// 创建节点参数
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateNodeInput {
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(rename = "type", default)]
    pub node_type: String, // opening / event / dialogue / choice / quest / ending / note
    #[serde(default, rename = "position_x")]
    pub position_x: f64,
    #[serde(default, rename = "position_y")]
    pub position_y: f64,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 创建关系线参数
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateEdgeInput {
    #[serde(rename = "to_node_id")]
    pub to_node_id: String,
    #[serde(rename = "type", default)]
    pub edge_type: String, // sequence / choice / conditional / teleport / loopback
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 创建章节边参数
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateChapterEdgeInput {
    #[serde(rename = "to_chapter_id")]
    pub to_chapter_id: String,
    #[serde(rename = "type", default)]
    pub edge_type: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 创建人物参数
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateCharacterInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, rename = "avatar_url")]
    pub avatar_url: String,
    #[serde(default)]
    pub personality: HashMap<String, serde_json::Value>,
    #[serde(default, rename = "voice_config")]
    pub voice_config: HashMap<String, serde_json::Value>,
}

/// 创建节点-人物关联参数
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateNodeCharacterInput {
    #[serde(rename = "character_id")]
    pub character_id: String,
    #[serde(default)]
    pub role: String, // protagonist / antagonist / npc / ...
    #[serde(default, rename = "dialogue_template")]
    pub dialogue_template: String,
    #[serde(default, rename = "action_hint")]
    pub action_hint: String,
    #[serde(default, rename = "sort_order")]
    pub sort_order: i64,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 创建钩子参数
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateHookInput {
    pub name: String,
    #[serde(rename = "type", default)]
    pub hook_type: String,
    #[serde(default, rename = "edge_id")]
    pub edge_id: String,
    #[serde(default, rename = "condition_logic")]
    pub condition_logic: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub priority: i64,
}
