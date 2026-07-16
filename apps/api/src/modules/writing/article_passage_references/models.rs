use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams)]
pub struct ArticleReferenceQuery {
    pub start: i32,
    pub end: i32,
    #[serde(default = "default_body")]
    pub kind: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

fn default_body() -> String {
    "body".to_string()
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
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PassageArticleListResponse {
    pub articles: Vec<PassageArticleResponse>,
    pub total: i64,
}
