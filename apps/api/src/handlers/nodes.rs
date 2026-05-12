use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::db;
use crate::error::AppError;
use crate::models::node::NodeDetail;
use crate::state::AppState;

#[derive(Deserialize, IntoParams)]
pub struct NodeParams {
    /// include original_text/original_html fields
    #[serde(default)]
    original: Option<bool>,
}

/// Get node content (blocks + sentences)
#[utoipa::path(
    get,
    path = "/api/books/{slug}/nodes/{node_slug}",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ("node_slug" = String, Path, description = "Node slug"),
        NodeParams,
    ),
    responses(
        (status = 200, description = "Node with content blocks and sentences", body = NodeDetail),
        (status = 404, description = "Node not found")
    ),
    tag = "nodes"
)]
pub async fn get_node(
    State(state): State<AppState>,
    Path((slug, node_slug)): Path<(String, String)>,
    Query(params): Query<NodeParams>,
) -> Result<Json<NodeDetail>, AppError> {
    let include_original = params.original.unwrap_or(false);
    let node =
        db::nodes::get_node_content(&state.pool, &slug, &node_slug, include_original).await?;
    Ok(Json(node))
}
