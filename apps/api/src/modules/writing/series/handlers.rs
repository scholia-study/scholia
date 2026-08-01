use axum::Json;
use axum::extract::{Path, Query, State};

use crate::modules::writing::series::db::{SeriesCreate, SeriesUpdate};
use crate::modules::writing::series::models::{
    AddSeriesArticleRequest, CreateSeriesRequest, ReorderSeriesArticlesRequest,
    SeriesAdminListResponse, SeriesAdminResponse, SeriesDetailQuery, SeriesDetailResponse,
    SeriesListQuery, SeriesListResponse, SeriesMemberListResponse, UpdateSeriesRequest,
};
use crate::system::auth::middleware::AuthUser;
use crate::system::auth::permissions::Permission;
use crate::system::cache;
use crate::system::error::AppError;
use crate::system::state::AppState;

fn require_series_manage(user: &AuthUser) -> Result<(), AppError> {
    user.require_permission(Permission::SeriesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))
}

fn validated_series_name(name: &str) -> Result<&str, AppError> {
    use crate::system::validation::{MAX_SERIES_NAME, check_max_len};
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("Series name cannot be empty".into()));
    }
    check_max_len("Series name", name, MAX_SERIES_NAME)?;
    Ok(name)
}

/// Trims and length-checks; keeps an empty string (the clear-description
/// signal on PATCH — the create path maps it to None instead).
fn validated_series_description(description: &str) -> Result<&str, AppError> {
    use crate::system::validation::{MAX_SERIES_DESCRIPTION, check_max_len};
    let description = description.trim();
    check_max_len("Series description", description, MAX_SERIES_DESCRIPTION)?;
    Ok(description)
}

fn parse_series_id(id: &str) -> Result<uuid::Uuid, AppError> {
    uuid::Uuid::parse_str(id).map_err(|_| AppError::BadRequest("Invalid series ID".into()))
}

/// Cache paths to invalidate when a series appears, changes, or is
/// removed. `/articles` is included because pinned-series shelves render
/// there. Membership changes additionally purge the affected article's
/// own paths (its strip changed); neighbors' strips ride out the TTL,
/// like article content edits do for listings.
pub(crate) fn series_cache_paths(slug: &str) -> Vec<String> {
    vec![
        format!("/articles/series/{slug}"),
        "/articles".to_string(),
        format!("/api/series/{slug}"),
        "/api/series".to_string(),
    ]
}

/// List series, pinned first, then by editor-set sort order
#[utoipa::path(
    get,
    path = "/api/series",
    params(SeriesListQuery),
    responses(
        (status = 200, description = "All series", body = SeriesListResponse)
    ),
    tag = "series"
)]
pub async fn list_series(
    State(state): State<AppState>,
    Query(params): Query<SeriesListQuery>,
) -> Result<Json<SeriesListResponse>, AppError> {
    let series =
        crate::modules::writing::series::db::list_series(&state.pool, params.pinned).await?;
    Ok(Json(SeriesListResponse { series }))
}

/// Get a series with one page of its published articles
#[utoipa::path(
    get,
    path = "/api/series/{slug}",
    params(
        ("slug" = String, Path, description = "Series slug"),
        SeriesDetailQuery,
    ),
    responses(
        (status = 200, description = "Series detail", body = SeriesDetailResponse),
        (status = 404, description = "Series not found")
    ),
    tag = "series"
)]
pub async fn get_series(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<SeriesDetailQuery>,
) -> Result<Json<SeriesDetailResponse>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let series = crate::modules::writing::series::db::get_series_detail_by_slug(
        &state.pool,
        &slug,
        page,
        per_page,
    )
    .await?;
    Ok(Json(series))
}

/// List series with member counts for the manage page
#[utoipa::path(
    get,
    path = "/api/admin/series",
    responses(
        (status = 200, description = "Series with member counts", body = SeriesAdminListResponse),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "admin"
)]
pub async fn admin_list_series(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<SeriesAdminListResponse>, AppError> {
    require_series_manage(&user)?;
    let series = crate::modules::writing::series::db::admin_list_series(&state.pool).await?;
    Ok(Json(SeriesAdminListResponse { series }))
}

/// Create a series; the slug is generated from the name and immutable
#[utoipa::path(
    post,
    path = "/api/admin/series",
    request_body = CreateSeriesRequest,
    responses(
        (status = 200, description = "Series created", body = SeriesAdminResponse),
        (status = 400, description = "Invalid or duplicate name"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "admin"
)]
pub async fn create_series(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateSeriesRequest>,
) -> Result<Json<SeriesAdminResponse>, AppError> {
    require_series_manage(&user)?;
    let name = validated_series_name(&body.name)?;
    let description = body
        .description
        .as_deref()
        .map(validated_series_description)
        .transpose()?
        .filter(|d| !d.is_empty());

    let series = crate::modules::writing::series::db::create_series(
        &state.pool,
        SeriesCreate { name, description },
    )
    .await?;

    let mut paths = series_cache_paths(&series.slug);
    paths.extend(crate::modules::writing::articles::handlers::sitemap_cache_paths());
    cache::invalidate(&state, paths);
    Ok(Json(series))
}

/// Update series metadata (name, description, pinned, sort order)
#[utoipa::path(
    patch,
    path = "/api/admin/series/{id}",
    params(("id" = String, Path, description = "Series ID")),
    request_body = UpdateSeriesRequest,
    responses(
        (status = 200, description = "Series updated", body = SeriesAdminResponse),
        (status = 400, description = "Invalid or duplicate name"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Series not found")
    ),
    tag = "admin"
)]
pub async fn update_series(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<UpdateSeriesRequest>,
) -> Result<Json<SeriesAdminResponse>, AppError> {
    require_series_manage(&user)?;
    let series_id = parse_series_id(&id)?;
    let name = body
        .name
        .as_deref()
        .map(validated_series_name)
        .transpose()?;
    let description = body
        .description
        .as_deref()
        .map(validated_series_description)
        .transpose()?;

    let series = crate::modules::writing::series::db::update_series(
        &state.pool,
        series_id,
        SeriesUpdate {
            name,
            description,
            pinned: body.pinned,
            sort_order: body.sort_order,
        },
    )
    .await?;

    cache::invalidate(&state, series_cache_paths(&series.slug));
    Ok(Json(series))
}

/// Delete a series; refused while any article is still attached
#[utoipa::path(
    delete,
    path = "/api/admin/series/{id}",
    params(("id" = String, Path, description = "Series ID")),
    responses(
        (status = 200, description = "Series deleted"),
        (status = 400, description = "Series still has articles"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Series not found")
    ),
    tag = "admin"
)]
pub async fn delete_series(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_series_manage(&user)?;
    let series_id = parse_series_id(&id)?;
    let slug = crate::modules::writing::series::db::delete_series(&state.pool, series_id).await?;

    let mut paths = series_cache_paths(&slug);
    paths.extend(crate::modules::writing::articles::handlers::sitemap_cache_paths());
    cache::invalidate(&state, paths);
    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// List a series' members in order, all statuses (manage drawer)
#[utoipa::path(
    get,
    path = "/api/admin/series/{id}/articles",
    params(("id" = String, Path, description = "Series ID")),
    responses(
        (status = 200, description = "Ordered members", body = SeriesMemberListResponse),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Series not found")
    ),
    tag = "admin"
)]
pub async fn list_series_members(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<SeriesMemberListResponse>, AppError> {
    require_series_manage(&user)?;
    let series_id = parse_series_id(&id)?;
    let articles =
        crate::modules::writing::series::db::list_series_members(&state.pool, series_id).await?;
    Ok(Json(SeriesMemberListResponse { articles }))
}

/// Add a published article to a series (prepends at position 1)
#[utoipa::path(
    post,
    path = "/api/admin/series/{id}/articles",
    params(("id" = String, Path, description = "Series ID")),
    request_body = AddSeriesArticleRequest,
    responses(
        (status = 200, description = "Article added"),
        (status = 400, description = "Article not published or already in the series"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Series or article not found")
    ),
    tag = "admin"
)]
pub async fn add_series_article(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<AddSeriesArticleRequest>,
) -> Result<Json<()>, AppError> {
    require_series_manage(&user)?;
    let series_id = parse_series_id(&id)?;
    let article_id = uuid::Uuid::parse_str(&body.article_id)
        .map_err(|_| AppError::BadRequest("Invalid article ID".into()))?;

    let (series_slug, article_slug) =
        crate::modules::writing::series::db::add_series_article(&state.pool, series_id, article_id)
            .await?;

    let mut paths = series_cache_paths(&series_slug);
    paths.extend(crate::modules::writing::articles::handlers::article_cache_paths(&article_slug));
    cache::invalidate(&state, paths);
    Ok(Json(()))
}

/// Remove an article from a series
#[utoipa::path(
    delete,
    path = "/api/admin/series/{id}/articles/{article_id}",
    params(
        ("id" = String, Path, description = "Series ID"),
        ("article_id" = String, Path, description = "Article ID"),
    ),
    responses(
        (status = 200, description = "Article removed"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Series, article, or membership not found")
    ),
    tag = "admin"
)]
pub async fn remove_series_article(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, article_id)): Path<(String, String)>,
) -> Result<Json<()>, AppError> {
    require_series_manage(&user)?;
    let series_id = parse_series_id(&id)?;
    let article_id = uuid::Uuid::parse_str(&article_id)
        .map_err(|_| AppError::BadRequest("Invalid article ID".into()))?;

    let (series_slug, article_slug) = crate::modules::writing::series::db::remove_series_article(
        &state.pool,
        series_id,
        article_id,
    )
    .await?;

    let mut paths = series_cache_paths(&series_slug);
    paths.extend(crate::modules::writing::articles::handlers::article_cache_paths(&article_slug));
    cache::invalidate(&state, paths);
    Ok(Json(()))
}

/// Reorder a series' members (full permutation of current members)
#[utoipa::path(
    put,
    path = "/api/admin/series/{id}/articles/order",
    params(("id" = String, Path, description = "Series ID")),
    request_body = ReorderSeriesArticlesRequest,
    responses(
        (status = 200, description = "Members reordered"),
        (status = 400, description = "Not a permutation of the current members"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Series not found")
    ),
    tag = "admin"
)]
pub async fn reorder_series_articles(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<ReorderSeriesArticlesRequest>,
) -> Result<Json<()>, AppError> {
    require_series_manage(&user)?;
    let series_id = parse_series_id(&id)?;
    let article_ids: Vec<uuid::Uuid> = body
        .article_ids
        .iter()
        .map(|s| {
            uuid::Uuid::parse_str(s)
                .map_err(|_| AppError::BadRequest(format!("Invalid article ID: {s}")))
        })
        .collect::<Result<_, _>>()?;

    let series_slug = crate::modules::writing::series::db::reorder_series_articles(
        &state.pool,
        series_id,
        &article_ids,
    )
    .await?;

    cache::invalidate(&state, series_cache_paths(&series_slug));
    Ok(Json(()))
}
