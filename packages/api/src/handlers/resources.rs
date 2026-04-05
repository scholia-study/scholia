use axum::Json;
use axum::extract::{Path, Query, State};

use crate::auth::middleware::AuthUser;
use crate::auth::permissions::Permission;
use crate::db;
use crate::error::AppError;
use crate::models::resource::{
    CreateResourceRequest, ResourceListResponse, ResourceQuery, UpdateResourceRequest,
};
use crate::state::AppState;

/// List resources for a sentence range (public)
#[utoipa::path(
    get,
    path = "/api/books/{slug}/resources",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ResourceQuery,
    ),
    responses(
        (status = 200, description = "Resources for the sentence range", body = ResourceListResponse),
        (status = 404, description = "Book not found")
    ),
    tag = "resources"
)]
pub async fn list_resources(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<ResourceQuery>,
) -> Result<Json<ResourceListResponse>, AppError> {
    let book_id = db::books::get_book_id_by_slug(&state.pool, &slug).await?;

    let resources =
        db::resources::list_resources(&state.pool, book_id, params.start, params.end, &params.kind)
            .await?;

    Ok(Json(ResourceListResponse { resources }))
}

/// Create a new resource (editor only)
#[utoipa::path(
    post,
    path = "/api/books/{slug}/resources",
    params(("slug" = String, Path, description = "Book slug")),
    request_body = CreateResourceRequest,
    responses(
        (status = 200, description = "Resource created", body = ResourceListResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "resources"
)]
pub async fn create_resource(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<CreateResourceRequest>,
) -> Result<Json<ResourceListResponse>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let book_id = db::books::get_book_id_by_slug(&state.pool, &slug).await?;

    let source_id = body
        .source_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid source_id".into()))?;

    let _resource_id = db::resources::create_resource(
        &state.pool,
        book_id,
        &body.resource_type,
        body.verbatim_kind.as_deref(),
        body.sentence_start,
        body.sentence_end,
        &body.sentence_kind,
        source_id,
        body.source_page_start,
        body.source_page_end,
        body.source_location_freeform.as_deref(),
        body.quoted_text.as_deref(),
        body.editor_note.as_deref(),
        body.is_featured.unwrap_or(false),
        body.admin_notes.as_deref(),
    )
    .await?;

    // Return the resource in context (re-fetch with full joins)
    let resources = db::resources::list_resources(
        &state.pool,
        book_id,
        body.sentence_start,
        body.sentence_end.unwrap_or(body.sentence_start),
        &body.sentence_kind,
    )
    .await?;

    Ok(Json(ResourceListResponse { resources }))
}

/// Update an existing resource (editor only)
#[utoipa::path(
    put,
    path = "/api/books/{slug}/resources/{id}",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ("id" = String, Path, description = "Resource ID")
    ),
    request_body = UpdateResourceRequest,
    responses(
        (status = 200, description = "Resource updated"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Resource not found")
    ),
    tag = "resources"
)]
pub async fn update_resource(
    State(state): State<AppState>,
    user: AuthUser,
    Path((slug, id)): Path<(String, String)>,
    Json(body): Json<UpdateResourceRequest>,
) -> Result<Json<()>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let book_id = db::books::get_book_id_by_slug(&state.pool, &slug).await?;
    let resource_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid resource ID".into()))?;

    let source_id = body
        .source_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid source_id".into()))?;

    db::resources::update_resource(
        &state.pool,
        resource_id,
        book_id,
        body.resource_type.as_deref(),
        body.verbatim_kind.as_deref(),
        body.sentence_start,
        body.sentence_end,
        body.sentence_kind.as_deref(),
        source_id,
        body.source_page_start,
        body.source_page_end,
        body.source_location_freeform.as_deref(),
        body.quoted_text.as_deref(),
        body.editor_note.as_deref(),
        body.is_featured,
        body.admin_notes.as_deref(),
    )
    .await?;

    Ok(Json(()))
}

/// Soft-delete a resource (editor only)
#[utoipa::path(
    delete,
    path = "/api/books/{slug}/resources/{id}",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ("id" = String, Path, description = "Resource ID")
    ),
    responses(
        (status = 200, description = "Resource archived"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Resource not found")
    ),
    tag = "resources"
)]
pub async fn delete_resource(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_slug, id)): Path<(String, String)>,
) -> Result<Json<()>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let resource_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid resource ID".into()))?;

    db::resources::soft_delete_resource(&state.pool, resource_id, user.id).await?;

    Ok(Json(()))
}
