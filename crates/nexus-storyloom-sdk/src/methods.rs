//! Story Loom API 方法 — 世界/章节/节点/边/人物/钩子

use crate::client::Client;
use crate::entities::*;
use crate::errors::Result;
use crate::inputs::*;
use serde::Serialize;

impl Client {
    // ──────────── StoryWorld ────────────

    /// 创建故事世界
    pub async fn create_story(&self, input: &CreateStoryInput) -> Result<StoryWorld> {
        self.post("/api/v1/worlds", &to_value(input)).await
    }

    /// 获取故事完整详情 (story + chapters + nodes + characters + edges + hooks)
    pub async fn get_story_detail(&self, story_id: &str) -> Result<StoryDetail> {
        self.get(&format!("/api/v1/worlds/{story_id}")).await
    }

    /// 获取故事世界
    pub async fn get_story(&self, story_id: &str) -> Result<StoryWorld> {
        self.get(&format!("/api/v1/worlds/{story_id}")).await
    }

    /// 列出当前用户的所有故事
    pub async fn list_stories(&self, opts: &ListOptions) -> Result<Vec<StoryWorld>> {
        let q = opts.query();
        self.get_list(&format!("/api/v1/worlds{q}")).await
    }

    /// 删除故事
    pub async fn delete_story(&self, story_id: &str) -> Result<()> {
        self.delete(&format!("/api/v1/worlds/{story_id}")).await
    }

    // ──────────── Chapter ────────────

    /// 创建章节
    pub async fn create_chapter(
        &self,
        story_id: &str,
        input: &CreateChapterInput,
    ) -> Result<Chapter> {
        self.post(
            &format!("/api/v1/worlds/{story_id}/chapters"),
            &to_value(input),
        )
        .await
    }

    /// 列出故事下的所有章节
    pub async fn list_chapters(&self, story_id: &str) -> Result<Vec<Chapter>> {
        self.get_list(&format!("/api/v1/worlds/{story_id}/chapters"))
            .await
    }

    /// 删除章节
    pub async fn delete_chapter(&self, chapter_id: &str) -> Result<()> {
        self.delete(&format!("/api/v1/chapters/{chapter_id}")).await
    }

    // ──────────── Node ────────────

    /// 在章节下创建节点
    pub async fn create_node(&self, chapter_id: &str, input: &CreateNodeInput) -> Result<Node> {
        self.post(
            &format!("/api/v1/chapters/{chapter_id}/nodes"),
            &to_value(input),
        )
        .await
    }

    /// 获取节点详情
    pub async fn get_node(&self, node_id: &str) -> Result<Node> {
        self.get(&format!("/api/v1/nodes/{node_id}")).await
    }

    /// 列出章节下的所有节点
    pub async fn list_nodes(&self, chapter_id: &str) -> Result<Vec<Node>> {
        self.get_list(&format!("/api/v1/chapters/{chapter_id}/nodes"))
            .await
    }

    /// 删除节点
    pub async fn delete_node(&self, node_id: &str) -> Result<()> {
        self.delete(&format!("/api/v1/nodes/{node_id}")).await
    }

    // ──────────── Edge (节点边) ────────────

    /// 创建从 from_node_id 出发的关系线
    pub async fn create_edge(&self, from_node_id: &str, input: &CreateEdgeInput) -> Result<Edge> {
        self.post(
            &format!("/api/v1/nodes/{from_node_id}/edges"),
            &to_value(input),
        )
        .await
    }

    /// 列出节点的所有出边
    pub async fn list_edges(&self, node_id: &str) -> Result<Vec<Edge>> {
        self.get_list(&format!("/api/v1/nodes/{node_id}/edges"))
            .await
    }

    /// 列出章节内所有边 (含反向)
    pub async fn list_edges_by_chapter(&self, chapter_id: &str) -> Result<Vec<Edge>> {
        self.get_list(&format!("/api/v1/chapters/{chapter_id}/edges"))
            .await
    }

    /// 删除节点边
    pub async fn delete_edge(&self, edge_id: &str) -> Result<()> {
        self.delete(&format!("/api/v1/edges/{edge_id}")).await
    }

    // ──────────── ChapterEdge (章节边) ────────────

    /// 创建章节间有向边
    pub async fn create_chapter_edge(
        &self,
        story_id: &str,
        from_chapter_id: &str,
        input: &CreateChapterEdgeInput,
    ) -> Result<ChapterEdge> {
        self.post(
            &format!("/api/v1/worlds/{story_id}/chapter-edges/{from_chapter_id}"),
            &to_value(input),
        )
        .await
    }

    /// 列出故事的章节边
    pub async fn list_chapter_edges(&self, story_id: &str) -> Result<Vec<ChapterEdge>> {
        self.get_list(&format!("/api/v1/worlds/{story_id}/chapter-edges"))
            .await
    }

    /// 删除章节边
    pub async fn delete_chapter_edge(&self, edge_id: &str) -> Result<()> {
        self.delete(&format!("/api/v1/chapter-edges/{edge_id}"))
            .await
    }

    // ──────────── Character ────────────

    /// 创建人物
    pub async fn create_character(
        &self,
        story_id: &str,
        input: &CreateCharacterInput,
    ) -> Result<Character> {
        self.post(
            &format!("/api/v1/worlds/{story_id}/characters"),
            &to_value(input),
        )
        .await
    }

    /// 列出故事下的所有人物
    pub async fn list_characters(&self, story_id: &str) -> Result<Vec<Character>> {
        self.get_list(&format!("/api/v1/worlds/{story_id}/characters"))
            .await
    }

    /// 获取人物详情
    pub async fn get_character(&self, character_id: &str) -> Result<Character> {
        self.get(&format!("/api/v1/characters/{character_id}"))
            .await
    }

    /// 删除人物
    pub async fn delete_character(&self, character_id: &str) -> Result<()> {
        self.delete(&format!("/api/v1/characters/{character_id}"))
            .await
    }

    // ──────────── NodeCharacter (节点-人物关联) ────────────

    /// 将人物关联到节点
    pub async fn create_node_character(
        &self,
        node_id: &str,
        input: &CreateNodeCharacterInput,
    ) -> Result<NodeCharacter> {
        self.post(
            &format!("/api/v1/nodes/{node_id}/characters"),
            &to_value(input),
        )
        .await
    }

    /// 列出节点的关联人物
    pub async fn list_node_characters(&self, node_id: &str) -> Result<Vec<NodeCharacter>> {
        self.get_list(&format!("/api/v1/nodes/{node_id}/characters"))
            .await
    }

    /// 移除节点-人物关联
    pub async fn delete_node_character(&self, node_id: &str, nc_id: &str) -> Result<()> {
        self.delete(&format!("/api/v1/nodes/{node_id}/characters/{nc_id}"))
            .await
    }

    // ──────────── Hook ────────────

    /// 在节点下创建推进钩子
    pub async fn create_hook(&self, node_id: &str, input: &CreateHookInput) -> Result<Hook> {
        self.post(
            &format!("/api/v1/nodes/{node_id}/hooks"),
            &to_value(input),
        )
        .await
    }

    /// 列出节点下的所有钩子
    pub async fn list_hooks(&self, node_id: &str) -> Result<Vec<Hook>> {
        self.get_list(&format!("/api/v1/nodes/{node_id}/hooks")).await
    }

    /// 切换钩子启用状态
    pub async fn toggle_hook(&self, hook_id: &str) -> Result<()> {
        self.post_void(&format!("/api/v1/hooks/{hook_id}/toggle"), &serde_json::json!({}))
            .await
    }
}

/// 列表查询选项
#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl ListOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, n: i64) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn offset(mut self, n: i64) -> Self {
        self.offset = Some(n);
        self
    }

    fn query(&self) -> String {
        let mut parts = Vec::new();
        if let Some(l) = self.limit {
            parts.push(format!("limit={l}"));
        }
        if let Some(o) = self.offset {
            parts.push(format!("offset={o}"));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        }
    }
}

fn to_value<T: Serialize>(input: &T) -> serde_json::Value {
    serde_json::to_value(input).unwrap_or_default()
}
