use axum::Json;
use axum::extract::State;

use crate::modules::corpus::reading::library::models::LibraryResponse;
use crate::system::error::AppError;
use crate::system::state::AppState;

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
    let library = crate::modules::corpus::reading::library::db::get_library(&state.pool).await?;
    Ok(Json(library))
}
