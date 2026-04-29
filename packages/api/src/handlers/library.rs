use axum::Json;
use axum::extract::State;

use crate::db;
use crate::error::AppError;
use crate::models::library::LibraryResponse;
use crate::state::AppState;

/// Get the grouped library tree (authors → works → versions) with stats.
#[utoipa::path(
    get,
    path = "/api/library",
    responses(
        (status = 200, description = "Library tree", body = LibraryResponse)
    ),
    tag = "books"
)]
pub async fn get_library(State(state): State<AppState>) -> Result<Json<LibraryResponse>, AppError> {
    let library = db::library::get_library(&state.pool).await?;
    Ok(Json(library))
}
