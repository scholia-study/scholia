use axum::Json;
use axum::extract::{Path, Query, State};

use crate::auth::middleware::AuthUser;
use crate::auth::permissions::Permission;
use crate::db;
use crate::error::AppError;
use crate::models::resource::{
    CreateSourceRequest, LinkSourcePersonRequest, ReferenceCheckResponse, SearchQuery,
    SourceResponse, SourceSearchResponse, UpdateSourceRequest,
};
use crate::state::AppState;

/// Search sources by title
#[utoipa::path(
    get,
    path = "/api/sources",
    params(SearchQuery),
    responses(
        (status = 200, description = "List of matching sources", body = Vec<SourceSearchResponse>),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "sources"
)]
pub async fn search_sources(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<SourceSearchResponse>>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let results = db::sources::search_sources(&state.pool, &params.q).await?;
    Ok(Json(results))
}

/// Create a new source
#[utoipa::path(
    post,
    path = "/api/sources",
    request_body = CreateSourceRequest,
    responses(
        (status = 200, description = "Source created", body = SourceResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "sources"
)]
pub async fn create_source(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateSourceRequest>,
) -> Result<Json<SourceResponse>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let parent_source_id = body
        .parent_source_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid parent_source_id".into()))?;

    let translation_of_id = body
        .translation_of_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid translation_of_id".into()))?;

    let source = db::sources::create_source(
        &state.pool,
        &body.source_type,
        &body.title,
        body.title_display.as_deref(),
        body.publication_year,
        body.publisher.as_deref(),
        body.isbn.as_deref(),
        body.doi.as_deref(),
        body.edition.as_deref(),
        body.volume.as_deref(),
        body.journal_name.as_deref(),
        body.url.as_deref(),
        body.page_start,
        body.page_end,
        parent_source_id,
        translation_of_id,
        user.id,
    )
    .await?;

    Ok(Json(source))
}

/// Update an existing source
#[utoipa::path(
    put,
    path = "/api/sources/{id}",
    params(("id" = String, Path, description = "Source ID")),
    request_body = UpdateSourceRequest,
    responses(
        (status = 200, description = "Source updated", body = SourceResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Source not found")
    ),
    tag = "sources"
)]
pub async fn update_source(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<UpdateSourceRequest>,
) -> Result<Json<SourceResponse>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let source_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid source ID".into()))?;

    let parent_source_id = body
        .parent_source_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid parent_source_id".into()))?;

    let translation_of_id = body
        .translation_of_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid translation_of_id".into()))?;

    let source = db::sources::update_source(
        &state.pool,
        source_id,
        body.source_type.as_deref(),
        body.title.as_deref(),
        body.title_display.as_deref(),
        body.publication_year,
        body.publisher.as_deref(),
        body.isbn.as_deref(),
        body.doi.as_deref(),
        body.edition.as_deref(),
        body.volume.as_deref(),
        body.journal_name.as_deref(),
        body.url.as_deref(),
        body.page_start,
        body.page_end,
        parent_source_id,
        translation_of_id,
    )
    .await?;

    Ok(Json(source))
}

/// Link a person to a source with a role
#[utoipa::path(
    post,
    path = "/api/sources/{id}/persons",
    params(("id" = String, Path, description = "Source ID")),
    request_body = LinkSourcePersonRequest,
    responses(
        (status = 200, description = "Person linked to source"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "sources"
)]
pub async fn add_source_person(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<LinkSourcePersonRequest>,
) -> Result<Json<()>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let source_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid source ID".into()))?;
    let person_id = uuid::Uuid::parse_str(&body.person_id)
        .map_err(|_| AppError::BadRequest("Invalid person_id".into()))?;

    db::sources::link_source_person(
        &state.pool,
        source_id,
        person_id,
        &body.role,
        body.position.unwrap_or(0),
    )
    .await?;

    Ok(Json(()))
}

/// Remove a person from a source
#[utoipa::path(
    delete,
    path = "/api/sources/{id}/persons/{person_id}/{role}",
    params(
        ("id" = String, Path, description = "Source ID"),
        ("person_id" = String, Path, description = "Person ID"),
        ("role" = String, Path, description = "Role to remove")
    ),
    responses(
        (status = 200, description = "Person unlinked from source"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "sources"
)]
pub async fn remove_source_person(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, person_id, role)): Path<(String, String, String)>,
) -> Result<Json<()>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let source_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid source ID".into()))?;
    let person_uuid = uuid::Uuid::parse_str(&person_id)
        .map_err(|_| AppError::BadRequest("Invalid person ID".into()))?;

    db::sources::unlink_source_person(&state.pool, source_id, person_uuid, &role).await?;

    Ok(Json(()))
}

/// Check if a source can be deleted (has active references)
#[utoipa::path(
    get,
    path = "/api/sources/{id}/references",
    params(("id" = String, Path, description = "Source ID")),
    responses(
        (status = 200, description = "Reference check result", body = ReferenceCheckResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "sources"
)]
pub async fn check_source_references(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<ReferenceCheckResponse>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let source_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid source ID".into()))?;

    let (count, resource_ids) =
        db::sources::check_source_references(&state.pool, source_id).await?;

    Ok(Json(ReferenceCheckResponse { count, resource_ids }))
}
