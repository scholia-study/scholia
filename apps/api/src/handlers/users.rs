use axum::Json;
use axum::extract::{Path, Query, State};

use crate::auth::handle::validate_handle;
use crate::db;
use crate::error::AppError;
use crate::models::article::PublicArticleListQuery;
use crate::models::user::{PublicProfileResponse, UserHandleResponse};
use crate::state::AppState;

/// Default page size for the profile's "published articles" list. The
/// `articles` field on the response is the first page; consumers wanting
/// more paginate via `GET /api/articles?author_handle=…&page=…`
/// (which doesn't exist yet — for v1 we just include the first page).
const PROFILE_ARTICLE_PAGE_SIZE: i32 = 20;

/// Public profile by handle.
#[utoipa::path(
    get,
    path = "/api/users/{handle}",
    params(
        ("handle" = String, Path, description = "User handle"),
        PublicArticleListQuery,
    ),
    responses(
        (status = 200, description = "User profile", body = PublicProfileResponse),
        (status = 400, description = "Invalid handle"),
        (status = 404, description = "User not found")
    ),
    tag = "users"
)]
pub async fn get_public_profile(
    State(state): State<AppState>,
    Path(handle): Path<String>,
    Query(params): Query<PublicArticleListQuery>,
) -> Result<Json<PublicProfileResponse>, AppError> {
    // Validate before hitting the DB so we don't 404 on garbage input.
    validate_handle(&handle)?;

    let row = db::users::get_public_profile_by_handle(&state.pool, &handle).await?;

    let roles = db::users::list_role_names(&state.pool, row.id).await?;
    let public_roles = crate::auth::permissions::filter_public_roles(&roles);

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params
        .per_page
        .unwrap_or(PROFILE_ARTICLE_PAGE_SIZE)
        .clamp(1, 100);
    let (articles, article_total) =
        db::articles::list_published_articles_by_author(&state.pool, row.id, page, per_page)
            .await?;

    Ok(Json(PublicProfileResponse {
        id: row.id.to_string(),
        handle: row.handle,
        display_name: row.display_name,
        bio: row.bio,
        title: row.title,
        location: row.location,
        website_url: row.website_url,
        avatar_url: row.avatar_url,
        public_roles,
        created_at: row
            .created_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        articles,
        article_total,
    }))
}

/// Resolve a user UUID to the user's *current* handle. Frontend uses
/// this to redirect from durable URLs (`/users/by-id/<uuid>`) to the
/// canonical handle URL.
#[utoipa::path(
    get,
    path = "/api/users/by-id/{id}",
    params(("id" = String, Path, description = "User UUID")),
    responses(
        (status = 200, description = "Current handle", body = UserHandleResponse),
        (status = 400, description = "Invalid UUID"),
        (status = 404, description = "User not found")
    ),
    tag = "users"
)]
pub async fn get_handle_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<UserHandleResponse>, AppError> {
    let user_id =
        uuid::Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid user ID".into()))?;
    let handle = db::users::get_handle_by_id(&state.pool, user_id).await?;
    Ok(Json(UserHandleResponse { handle }))
}
