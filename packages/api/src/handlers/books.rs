use axum::Json;
use axum::extract::{Path, State};

use crate::db;
use crate::error::AppError;
use crate::models::book::{BookDetail, BookSummary};
use crate::state::AppState;

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
    let books = db::books::list_books(&state.pool).await?;
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
    let book = db::books::get_book_by_slug(&state.pool, &slug).await?;
    Ok(Json(book))
}
