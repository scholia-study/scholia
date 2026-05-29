use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::modules::corpus::reading::books::models::{
    AboutThisTextResponse, BookDetail, BookSummary,
};
use crate::system::error::AppError;
use crate::system::state::AppState;

/// List all books
#[utoipa::path(
    get,
    path = "/api/books",
    responses(
        (status = 200, description = "List of books", body = Vec<BookSummary>)
    ),
    tag = "books"
)]
pub async fn list_books(State(state): State<AppState>) -> Result<Json<Vec<BookSummary>>, AppError> {
    let books = crate::modules::corpus::reading::books::db::list_books(&state.pool).await?;
    Ok(Json(books))
}

/// Get a book by slug
#[utoipa::path(
    get,
    path = "/api/books/{slug}",
    params(("slug" = String, Path, description = "Book slug")),
    responses(
        (status = 200, description = "Book details", body = BookDetail),
        (status = 404, description = "Book not found")
    ),
    tag = "books"
)]
pub async fn get_book(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<BookDetail>, AppError> {
    let book =
        crate::modules::corpus::reading::books::db::get_book_by_slug(&state.pool, &slug).await?;
    Ok(Json(book))
}

#[derive(Deserialize)]
pub struct AboutQuery {
    pub node: Option<String>,
}

/// Bibliographic info for the "About this text" panel. With `?node=`,
/// walks the toc-ancestor chain to surface the constituent work the
/// reader is in (e.g. Genesis within KJV); otherwise returns the
/// hosted book's source.
#[utoipa::path(
    get,
    path = "/api/books/{slug}/about",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ("node" = Option<String>, Query, description = "Active toc node slug for contextual resolution")
    ),
    responses(
        (status = 200, description = "Bibliographic info for the active text", body = AboutThisTextResponse),
        (status = 404, description = "Book or node not found")
    ),
    tag = "books"
)]
pub async fn get_book_about(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<AboutQuery>,
) -> Result<Json<AboutThisTextResponse>, AppError> {
    let about = crate::modules::corpus::reading::books::db::get_about_this_text(
        &state.pool,
        &slug,
        query.node.as_deref(),
    )
    .await?;
    Ok(Json(about))
}
