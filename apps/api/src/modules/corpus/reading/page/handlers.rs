use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::modules::corpus::reading::page::models::NodePage;
use crate::system::error::AppError;
use crate::system::state::AppState;

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
    /// Anchor node slug — return a forward window centered on this node
    /// (combine with `back` to include nodes before the anchor)
    #[serde(default)]
    at: Option<String>,
    /// Number of nodes to include before the `at` anchor (default 0)
    #[serde(default)]
    back: Option<i32>,
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
        let page = crate::modules::corpus::reading::page::db::get_nodes_by_source_ids(
            pool,
            &slug,
            &source_node_ids,
            include_original,
        )
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
        let page = crate::modules::corpus::reading::page::db::get_nodes_by_ids(
            pool,
            &slug,
            &ids,
            include_original,
        )
        .await?;
        return Ok(Json(page));
    }

    let limit = params.limit.unwrap_or(20).clamp(1, 50);

    // Per-book reader pagination directive: a small `nodes_per_page` (e.g.
    // Paradise Lost = 1) makes the reader load that many nodes per page instead
    // of the whole work. NULL leaves every existing text on the path below,
    // unchanged. This branch sizes the page itself (and, for the anchor window,
    // keeps the target node included even at sort_order 0) so it never has to
    // touch the shared default-path math.
    let nodes_per_page: Option<i16> =
        sqlx::query_scalar!("SELECT nodes_per_page FROM books WHERE slug = $1", slug)
            .fetch_optional(pool)
            .await?
            .flatten();

    if let Some(p) = nodes_per_page {
        let page_size = (p as i32).max(1);
        if let Some(ref node_slug) = params.at {
            // Small back-buffer that still leaves room for the target + a lead.
            let back = params.back.unwrap_or(0).max(0).min(page_size - 1);
            let target_sort: Option<i32> = sqlx::query_scalar!(
                "SELECT tn.sort_order FROM toc_nodes tn
                 JOIN books b ON b.id = tn.book_id
                 WHERE b.slug = $1 AND tn.slug = $2",
                slug,
                node_slug,
            )
            .fetch_optional(pool)
            .await?;
            let page = match target_sort {
                // `after` may go to -1 (sort_order > -1 includes node 0).
                Some(ts) => {
                    crate::modules::corpus::reading::page::db::get_node_page(
                        pool,
                        &slug,
                        Some(ts - 1 - back),
                        None,
                        back + page_size,
                        include_original,
                    )
                    .await?
                }
                None => NodePage {
                    nodes: vec![],
                    has_more: false,
                    has_previous: false,
                },
            };
            return Ok(Json(page));
        }
        let page = crate::modules::corpus::reading::page::db::get_node_page(
            pool,
            &slug,
            params.after,
            params.before,
            page_size,
            include_original,
        )
        .await?;
        return Ok(Json(page));
    }

    if let Some(ref node_slug) = params.at {
        let back = params.back.unwrap_or(0).max(0);
        let page = crate::modules::corpus::reading::page::db::get_node_page_at(
            pool,
            &slug,
            node_slug,
            back,
            limit,
            include_original,
        )
        .await?;
        return Ok(Json(page));
    }
    let page = crate::modules::corpus::reading::page::db::get_node_page(
        pool,
        &slug,
        params.after,
        params.before,
        limit,
        include_original,
    )
    .await?;
    Ok(Json(page))
}
