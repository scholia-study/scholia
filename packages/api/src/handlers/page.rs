use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::IntoParams;

use crate::db;
use crate::error::AppError;
use crate::models::page::NodePage;

#[derive(Deserialize, IntoParams)]
pub struct PageParams {
    /// sort_order cursor — fetch nodes after this value
    #[serde(default)]
    after: Option<i32>,
    /// sort_order cursor — fetch nodes before this value
    #[serde(default)]
    before: Option<i32>,
    /// page size, default 20, max 50
    limit: Option<i32>,
    /// include original_text/original_html fields
    #[serde(default)]
    original: Option<bool>,
}

/// Get paginated nodes for infinite scroll
#[utoipa::path(
    get,
    path = "/api/books/{slug}/nodes",
    params(
        ("slug" = String, Path, description = "Book slug"),
        PageParams,
    ),
    responses(
        (status = 200, description = "Page of nodes with content", body = NodePage),
        (status = 404, description = "Book not found")
    ),
    tag = "nodes"
)]
pub async fn get_node_page(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
    Query(params): Query<PageParams>,
) -> Result<Json<NodePage>, AppError> {
    let limit = params.limit.unwrap_or(20).min(50).max(1);
    let include_original = params.original.unwrap_or(false);
    let page = db::page::get_node_page(&pool, &slug, params.after, params.before, limit, include_original).await?;
    Ok(Json(page))
}
