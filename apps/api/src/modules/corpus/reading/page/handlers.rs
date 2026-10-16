use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::modules::corpus::reading::page::db::{self, PageWindow};
use crate::modules::corpus::reading::page::models::NodePage;
use crate::system::error::AppError;
use crate::system::state::AppState;

/// Cap on the `node_ids` / `source_nodes` batch. These branches fetch full
/// content per id and skip the normal page-size clamp, so an unbounded list is
/// a cheap amplification vector on this public endpoint. The largest legitimate
/// caller is the interleaved-translation view fetching every loaded companion
/// node; the biggest structurally-paired work is ~120 nodes, and browsers cap a
/// URL near ~200 ids regardless, so 256 leaves ample headroom.
const MAX_NODE_BATCH: usize = 256;

#[derive(Deserialize, IntoParams)]
pub struct PageParams {
    /// sort_order cursor — fetch nodes after this value
    #[serde(default)]
    after: Option<i32>,
    /// sort_order cursor — fetch nodes before this value
    #[serde(default)]
    before: Option<i32>,
    /// include original_text/original_html fields
    #[serde(default)]
    original: Option<bool>,
    /// Comma-separated source node UUIDs — fetch nodes whose source_node_id matches
    #[serde(default)]
    source_nodes: Option<String>,
    /// Comma-separated node UUIDs — fetch nodes by their own ID
    #[serde(default)]
    node_ids: Option<String>,
    /// Anchor node slug — return the anchor and what follows, with no nodes
    /// before it
    #[serde(default)]
    at: Option<String>,
}

/// Get paginated nodes for infinite scroll
///
/// Pages are sized by a byte budget, not a node count, so their length varies
/// with the book's node sizes.
#[utoipa::path(
    get,
    path = "/api/books/{slug}/nodes",
    params(
        ("slug" = String, Path, description = "Book slug"),
        PageParams,
    ),
    responses(
        (status = 200, description = "Page of nodes with content", body = NodePage),
        (status = 400, description = "Too many node_ids/source_nodes"),
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

    let book_exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM books WHERE slug = $1) AS "exists!""#,
        slug
    )
    .fetch_one(pool)
    .await?;
    if !book_exists {
        return Err(AppError::NotFound("Book not found".into()));
    }

    if let Some(ref source_nodes_str) = params.source_nodes {
        let source_node_ids: Vec<Uuid> = source_nodes_str
            .split(',')
            .filter_map(|s| s.trim().parse::<Uuid>().ok())
            .collect();
        if source_node_ids.len() > MAX_NODE_BATCH {
            return Err(AppError::BadRequest(format!(
                "Too many source_nodes (max {MAX_NODE_BATCH})"
            )));
        }
        if source_node_ids.is_empty() {
            return Ok(Json(NodePage {
                nodes: vec![],
                has_more: false,
                has_previous: false,
            }));
        }
        let page =
            db::get_nodes_by_source_ids(pool, &slug, &source_node_ids, include_original).await?;
        return Ok(Json(page));
    }

    if let Some(ref node_ids_str) = params.node_ids {
        let ids: Vec<Uuid> = node_ids_str
            .split(',')
            .filter_map(|s| s.trim().parse::<Uuid>().ok())
            .collect();
        if ids.len() > MAX_NODE_BATCH {
            return Err(AppError::BadRequest(format!(
                "Too many node_ids (max {MAX_NODE_BATCH})"
            )));
        }
        if ids.is_empty() {
            return Ok(Json(NodePage {
                nodes: vec![],
                has_more: false,
                has_previous: false,
            }));
        }
        let page = db::get_nodes_by_ids(pool, &slug, &ids, include_original).await?;
        return Ok(Json(page));
    }

    let window = match (&params.at, params.before) {
        (Some(node_slug), _) => PageWindow::At(node_slug),
        (None, Some(before)) => PageWindow::Before(before),
        (None, None) => PageWindow::After(params.after.unwrap_or(-1)),
    };
    let page = db::get_node_page(pool, &slug, window, include_original).await?;
    Ok(Json(page))
}
