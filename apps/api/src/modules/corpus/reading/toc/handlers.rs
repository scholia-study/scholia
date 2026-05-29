use axum::Json;
use axum::extract::{Path, State};

use crate::modules::corpus::reading::toc::models::TocNodeResponse;
use crate::system::error::AppError;
use crate::system::state::AppState;

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
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<TocNodeResponse>>, AppError> {
    let tree = crate::modules::corpus::reading::toc::db::get_toc_tree(&state.pool, &slug).await?;
    Ok(Json(tree))
}
