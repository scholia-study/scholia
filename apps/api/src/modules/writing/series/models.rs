use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::modules::writing::articles::models::ArticleResponse;

#[derive(Debug, Serialize, ToSchema)]
pub struct SeriesResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub pinned: bool,
    /// Published members only.
    pub article_count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeriesListResponse {
    pub series: Vec<SeriesResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeriesDetailResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// One page of published members, position-ascending.
    pub articles: Vec<ArticleResponse>,
    /// Total published members across all pages.
    pub total: i64,
}

/// Series row for the manage page: public fields plus member counts of
/// every status (delete is refused while `article_count` is non-zero).
#[derive(Debug, Serialize, ToSchema)]
pub struct SeriesAdminResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub pinned: bool,
    pub sort_order: i32,
    /// Members of any status.
    pub article_count: i64,
    /// Members currently visible on public series surfaces.
    pub published_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeriesAdminListResponse {
    pub series: Vec<SeriesAdminResponse>,
}

/// `slug` is generated from the name at creation and immutable after —
/// it appears in shareable URLs (`/articles/series/<slug>`).
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSeriesRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Partial update; the slug never changes. An empty-string `description`
/// clears it.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSeriesRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub pinned: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

/// Ordered member row for the manage drawer. Unlike the public surfaces
/// this includes non-published members (rendered greyed with their
/// status) — membership survives unpublish.
#[derive(Debug, Serialize, ToSchema)]
pub struct SeriesMemberResponse {
    pub article_id: String,
    pub title: String,
    pub slug: String,
    pub status: String,
    pub author_display_name: String,
    pub position: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeriesMemberListResponse {
    pub articles: Vec<SeriesMemberResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddSeriesArticleRequest {
    pub article_id: String,
}

/// The full member list in its new order — must be an exact permutation
/// of the series' current members.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReorderSeriesArticlesRequest {
    pub article_ids: Vec<String>,
}

/// A series the article belongs to, as shown on the article page strip.
/// Neighbors are published members only; `prev` is toward position 1
/// (the newest end for prepend-ordered series like the blog).
#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleSeriesContext {
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<SeriesNeighbor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<SeriesNeighbor>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeriesNeighbor {
    pub title: String,
    pub slug: String,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct SeriesListQuery {
    /// `true` restricts the list to pinned series.
    #[serde(default)]
    pub pinned: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct SeriesDetailQuery {
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub per_page: Option<i32>,
}
