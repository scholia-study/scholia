use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

// ── Response types ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TopicResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TopicListResponse {
    pub topics: Vec<TopicResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleResponse {
    pub id: String,
    pub title: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    pub author_display_name: String,
    pub topics: Vec<TopicResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleDetailResponse {
    pub id: String,
    pub title: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub markdown: String,
    pub html: String,
    pub status: String,
    pub author_display_name: String,
    pub topics: Vec<TopicResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleListResponse {
    pub articles: Vec<ArticleResponse>,
    pub limits: ArticleLimitsResponse,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PublishedArticleListResponse {
    pub articles: Vec<ArticleResponse>,
    pub total: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleLimitsResponse {
    pub max_active: i32,
    pub current_active: i64,
    pub max_archive: i32,
    pub current_archive: i64,
}

// ── Batch sentence types for quotation card hydration ─────

#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchSentenceRequest {
    pub book_slug: String,
    pub node_slug: String,
    pub start_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_number: Option<i32>,
    pub kind: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchSentencesRequest {
    pub items: Vec<BatchSentenceRequest>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SentenceData {
    pub sentence_number: i32,
    pub html: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_html: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SourceContext {
    pub book_slug: String,
    pub book_title: String,
    pub node_slug: String,
    pub node_label: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchSentenceResponseItem {
    pub book_slug: String,
    pub book_title: String,
    pub node_slug: String,
    pub node_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceContext>,
    pub sentences: Vec<SentenceData>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchSentencesResponse {
    pub items: Vec<BatchSentenceResponseItem>,
}

// ── Request types ──────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateArticleRequest {
    pub title: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateArticleRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub markdown: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub topic_ids: Option<Vec<String>>,
}

// ── Query types ────────────────────────────────────────────

#[derive(Debug, Deserialize, IntoParams)]
pub struct ArticleListQuery {
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct PublicArticleListQuery {
    #[serde(default)]
    pub topic_slug: Option<String>,
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub per_page: Option<i32>,
}
