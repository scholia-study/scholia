use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::db;
use crate::error::AppError;
use crate::models::page::NodePage;
use crate::state::AppState;

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
    /// Comma-separated source node UUIDs — fetch nodes whose source_node_id matches
    #[serde(default)]
    source_nodes: Option<String>,
    /// Comma-separated node UUIDs — fetch nodes by their own ID
    #[serde(default)]
    node_ids: Option<String>,
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
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<PageParams>,
) -> Result<Json<NodePage>, AppError> {
    let pool = &state.pool;
    let include_original = params.original.unwrap_or(false);

    if let Some(ref source_nodes_str) = params.source_nodes {
        let source_node_ids: Vec<Uuid> = source_nodes_str
            .split(',')
            .filter_map(|s| s.trim().parse::<Uuid>().ok())
            .collect();
        if source_node_ids.is_empty() {
            return Ok(Json(NodePage {
                nodes: vec![],
                has_more: false,
                has_previous: false,
            }));
        }
        let page =
            db::page::get_nodes_by_source_ids(&pool, &slug, &source_node_ids, include_original)
                .await?;
        return Ok(Json(page));
    }

    if let Some(ref node_ids_str) = params.node_ids {
        let ids: Vec<Uuid> = node_ids_str
            .split(',')
            .filter_map(|s| s.trim().parse::<Uuid>().ok())
            .collect();
        if ids.is_empty() {
            return Ok(Json(NodePage {
                nodes: vec![],
                has_more: false,
                has_previous: false,
            }));
        }
        let page = db::page::get_nodes_by_ids(&pool, &slug, &ids, include_original).await?;
        return Ok(Json(page));
    }

    let limit = params.limit.unwrap_or(20).clamp(1, 50);
    let page = db::page::get_node_page(
        &pool,
        &slug,
        params.after,
        params.before,
        limit,
        include_original,
    )
    .await?;
    Ok(Json(page))
}
