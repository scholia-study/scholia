use axum::extract::{Path, State};
use axum::Json;
use sqlx::PgPool;

use crate::db;
use crate::error::AppError;
use crate::models::toc::TocNodeResponse;

/// Get the full TOC tree for a book
#[utoipa::path(
    get,
    path = "/api/books/{slug}/toc",
    params(("slug" = String, Path, description = "Book slug")),
    responses(
        (status = 200, description = "TOC tree", body = Vec<TocNodeResponse>),
        (status = 404, description = "Book not found")
    ),
    tag = "toc"
)]
pub async fn get_toc(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<TocNodeResponse>>, AppError> {
    let tree = db::toc::get_toc_tree(&pool, &slug).await?;
    Ok(Json(tree))
}
