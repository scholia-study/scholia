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

/// Public-facing editorial label. `applied_by`/`applied_at` are
/// deliberately not exposed — readers don't need to know which editor
/// chipped an article or when.
///
/// `revokes_on_edit` IS exposed so the article editor can warn authors
/// before they make changes that would strip the chip; it's a property
/// of the label itself, not user-specific or sensitive.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EditorialLabelResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub revokes_on_edit: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EditorialLabelListResponse {
    pub labels: Vec<EditorialLabelResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ApplyEditorialLabelRequest {
    pub label_slug: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleResponse {
    pub id: String,
    pub title: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    pub author_user_id: String,
    pub author_display_name: String,
    /// Current handle of the author. Use `/users/<handle>` for the
    /// canonical link, or `/users/by-id/<author_user_id>` for a
    /// rename-durable link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_handle: Option<String>,
    /// Public-facing role chips for the author (`editor`, paid tiers).
    pub author_public_roles: Vec<String>,
    pub topics: Vec<TopicResponse>,
    /// Editor/admin-applied editorial labels. Empty for drafts and for
    /// articles no editor has chipped. Ordered by `editorial_labels.sort_order`.
    pub labels: Vec<EditorialLabelResponse>,
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
    pub author_user_id: String,
    pub author_display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_handle: Option<String>,
    pub author_public_roles: Vec<String>,
    pub topics: Vec<TopicResponse>,
    pub labels: Vec<EditorialLabelResponse>,
    /// Labels whose `revokes_on_edit` flag is `true` and which were
    /// stripped by the most recent author edit. Empty unless this
    /// response is the result of a markdown update that revoked chips.
    /// Frontend uses this to toast the author.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub revoked_labels: Vec<EditorialLabelResponse>,
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

/// One system's contribution to a quotation's citation, resolved over the cited
/// range. `template` carries `{parent}`/`{self}`/`{ref}` tokens (the frontend
/// substitutes the node labels and the `first_ref`[–`last_ref`] range). Parts
/// are ordered by the system's `cite_priority`; multiple parts are joined for
/// multi-system citations (e.g. Kant A/B). An empty list means the book has no
/// default citation system, so the card falls back to `s. N`.
#[derive(Debug, Serialize, ToSchema)]
pub struct CitationPart {
    pub template: String,
    pub first_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_ref: Option<String>,
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
    /// Label of the cited node's parent in the toc tree, when one
    /// exists. For bibles this is the bible-book ("Romans"); for Milton
    /// the work ("Paradise Lost"). Substituted into `{parent}` of a
    /// citation template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_node_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceContext>,
    /// Resolved citation parts (ordered by `cite_priority`). Empty = no
    /// default citation system; the card falls back to `s. N`.
    pub citation: Vec<CitationPart>,
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
    /// Filter the listing to articles bearing this editorial label slug
    /// (e.g. `featured`, `high-quality`). Single label per request — no
    /// AND-ing across labels in v1.
    #[serde(default)]
    pub label_slug: Option<String>,
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub per_page: Option<i32>,
}
