use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams)]
pub struct ArticleReferenceQuery {
    pub start: i32,
    pub end: i32,
    #[serde(default = "default_body")]
    pub kind: crate::modules::corpus::SentenceKind,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

fn default_body() -> crate::modules::corpus::SentenceKind {
    crate::modules::corpus::SentenceKind::Body
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PassageArticleResponse {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub author_user_id: String,
    pub author_display_name: String,
    /// Current handle of the author. Use `/users/<handle>` for the
    /// canonical link, or `/users/by-id/<author_user_id>` for a
    /// rename-durable link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    /// The editions whose passages this article's matched references
    /// anchor on, for the selection being viewed. Contains the requested
    /// book itself when the article quotes it directly; other entries
    /// mean the article reached this selection across editions (ADR
    /// 0008) — the reader badges those by language/edition.
    pub origins: Vec<PassageArticleOrigin>,
}

/// One edition an article's matched references anchor on.
#[derive(Debug, Clone, Serialize, serde::Deserialize, ToSchema)]
pub struct PassageArticleOrigin {
    pub book_slug: String,
    pub language: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PassageArticleListResponse {
    pub articles: Vec<PassageArticleResponse>,
    pub total: i64,
}
