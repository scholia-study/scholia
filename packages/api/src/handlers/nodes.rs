use axum::extract::{Path, State};
use axum::Json;
use sqlx::PgPool;

use crate::db;
use crate::error::AppError;
use crate::models::node::NodeDetail;

/// Get node content (blocks + sentences)
#[utoipa::path(
    get,
    path = "/api/books/{slug}/nodes/{node_slug}",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ("node_slug" = String, Path, description = "Node slug"),
    ),
    responses(
        (status = 200, description = "Node with content blocks and sentences", body = NodeDetail),
        (status = 404, description = "Node not found")
    ),
    tag = "nodes"
)]
pub async fn get_node(
    State(pool): State<PgPool>,
    Path((slug, node_slug)): Path<(String, String)>,
) -> Result<Json<NodeDetail>, AppError> {
    let node = db::nodes::get_node_content(&pool, &slug, &node_slug).await?;
    Ok(Json(node))
}
