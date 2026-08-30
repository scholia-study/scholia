use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// An article's lifecycle state. Stored as the Postgres enum
/// `article_status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "article_status", rename_all = "lowercase")]
pub enum ArticleStatus {
    Draft,
    Published,
    Archived,
}

use crate::modules::writing::series::models::ArticleSeriesContext;

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

/// Topic row for the admin panel: public fields plus how many articles
/// currently carry the topic (delete is refused while non-zero).
#[derive(Debug, Serialize, ToSchema)]
pub struct TopicAdminResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub article_count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TopicAdminListResponse {
    pub topics: Vec<TopicAdminResponse>,
}

/// `slug` is generated from the name at creation and immutable after —
/// it appears in shareable URLs (`/articles?topic_slug=…`).
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTopicRequest {
    pub name: String,
}

/// Renames the display name only; the slug never changes.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTopicRequest {
    pub name: String,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetArticleQuotingRequest {
    pub quoting_disabled: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleQuotingResponse {
    pub slug: String,
    pub quoting_disabled: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleResponse {
    pub id: String,
    pub title: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: ArticleStatus,
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
    /// Pending review request on this article, if any. Owner-facing
    /// listings only; absent on public endpoints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_review_request_id: Option<String>,
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
    pub status: ArticleStatus,
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
    /// Pending review request on this article, if any. Populated for the
    /// owner (`get_user_article`) only; absent on public endpoints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_review_request_id: Option<String>,
    /// Most recent review request of any status (owner only) — the
    /// author's entry point to the review page and its history.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_review_request_id: Option<String>,
    /// Series this article belongs to, with published prev/next
    /// neighbors — drives the article-page series strip. Empty for
    /// articles in no series (and on owner-facing endpoints).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub series: Vec<ArticleSeriesContext>,
    /// Admin-set: suppress the reader's sentence-selection layer, for
    /// articles that read as blog posts rather than quotable scholarship.
    pub quoting_disabled: bool,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchSentenceRequest {
    pub book_slug: String,
    pub node_slug: String,
    /// Body/footnote sentence-number addressing. Optional when `start_id`
    /// addresses the range by sentence UUID instead (quotations anchored
    /// on unnumbered sentences: figure captions, headings).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_id: Option<String>,
    pub kind: crate::modules::corpus::SentenceKind,
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
    /// For a quotation anchored on a figure's caption: the figure block's
    /// verbatim `<figure>` markup, so the embed renders the whole figure
    /// exactly as the reader does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub figure_html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub figure_original_html: Option<String>,
    /// The figure's book-wide display number — the reader deep-link key is
    /// `fig{N}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub figure_number: Option<i32>,
    pub sentences: Vec<SentenceData>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchSentencesResponse {
    pub items: Vec<BatchSentenceResponseItem>,
}

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

#[derive(Debug, Deserialize, IntoParams)]
pub struct ArticleListQuery {
    #[serde(default)]
    pub status: Option<String>,
}

/// Query for the public article listing. A superset of
/// `PublicArticleListQuery` — kept separate so the profile endpoint
/// (which reuses that struct) doesn't advertise search params it
/// ignores. `q`/`author` also serve the series manage drawer's finder.
#[derive(Debug, Deserialize, IntoParams)]
pub struct PublishedArticleSearchQuery {
    #[serde(default)]
    pub topic_slug: Option<String>,
    /// Filter the listing to articles bearing this editorial label slug
    /// (e.g. `featured`, `high-quality`).
    #[serde(default)]
    pub label_slug: Option<String>,
    /// Case-insensitive contains-match on the article title.
    #[serde(default)]
    pub q: Option<String>,
    /// Case-insensitive contains-match on the author's display name or
    /// handle.
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub per_page: Option<i32>,
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
