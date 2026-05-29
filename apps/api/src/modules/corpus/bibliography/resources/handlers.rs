use axum::Json;
use axum::extract::{Path, Query, State};

use crate::modules::corpus::bibliography::models::{
    CreateResourceRequest, ResourceListResponse, ResourceQuery, UpdateResourceRequest,
};
use crate::system::auth::middleware::AuthUser;
use crate::system::auth::permissions::Permission;
use crate::system::error::AppError;
use crate::system::state::AppState;
use crate::system::validation::{
    MAX_RESOURCE_ADMIN_NOTES, MAX_RESOURCE_EDITOR_NOTE, MAX_RESOURCE_QUOTED_TEXT,
    MAX_RESOURCE_SOURCE_LOCATION, MAX_RESOURCE_SOURCE_PAGE, MIN_RESOURCE_SOURCE_PAGE,
    check_int_range, check_max_len,
};

fn validate_resource_fields(
    quoted_text: Option<&str>,
    editor_note: Option<&str>,
    admin_notes: Option<&str>,
    source_location_freeform: Option<&str>,
    source_page_start: Option<i32>,
    source_page_end: Option<i32>,
) -> Result<(), AppError> {
    if let Some(q) = quoted_text {
        check_max_len("Quoted text", q, MAX_RESOURCE_QUOTED_TEXT)?;
    }
    if let Some(e) = editor_note {
        check_max_len("Editor note", e, MAX_RESOURCE_EDITOR_NOTE)?;
    }
    if let Some(a) = admin_notes {
        check_max_len("Admin notes", a, MAX_RESOURCE_ADMIN_NOTES)?;
    }
    if let Some(l) = source_location_freeform {
        check_max_len("Source location", l, MAX_RESOURCE_SOURCE_LOCATION)?;
    }
    if let Some(p) = source_page_start {
        check_int_range(
            "Source page start",
            p,
            MIN_RESOURCE_SOURCE_PAGE,
            MAX_RESOURCE_SOURCE_PAGE,
        )?;
    }
    if let Some(p) = source_page_end {
        check_int_range(
            "Source page end",
            p,
            MIN_RESOURCE_SOURCE_PAGE,
            MAX_RESOURCE_SOURCE_PAGE,
        )?;
    }
    Ok(())
}

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
    let book_id =
        crate::modules::corpus::reading::books::db::get_book_id_by_slug(&state.pool, &slug).await?;

    let resources = crate::modules::corpus::bibliography::resources::db::list_resources(
        &state.pool,
        book_id,
        params.start,
        params.end,
        &params.kind,
    )
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

    let book_id =
        crate::modules::corpus::reading::books::db::get_book_id_by_slug(&state.pool, &slug).await?;

    let source_id = body
        .source_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid source_id".into()))?;

    validate_resource_fields(
        body.quoted_text.as_deref(),
        body.editor_note.as_deref(),
        body.admin_notes.as_deref(),
        body.source_location_freeform.as_deref(),
        body.source_page_start,
        body.source_page_end,
    )?;

    let _resource_id = crate::modules::corpus::bibliography::resources::db::create_resource(
        &state.pool,
        book_id,
        crate::modules::corpus::bibliography::resources::db::ResourceCreate {
            resource_type: &body.resource_type,
            verbatim_kind: body.verbatim_kind.as_deref(),
            sentence_start: body.sentence_start,
            sentence_end: body.sentence_end,
            sentence_kind: &body.sentence_kind,
            source_id,
            source_page_start: body.source_page_start,
            source_page_end: body.source_page_end,
            source_location_freeform: body.source_location_freeform.as_deref(),
            quoted_text: body.quoted_text.as_deref(),
            editor_note: body.editor_note.as_deref(),
            is_featured: body.is_featured.unwrap_or(false),
            admin_notes: body.admin_notes.as_deref(),
        },
    )
    .await?;

    // Return the resource in context (re-fetch with full joins)
    let resources = crate::modules::corpus::bibliography::resources::db::list_resources(
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

    let book_id =
        crate::modules::corpus::reading::books::db::get_book_id_by_slug(&state.pool, &slug).await?;
    let resource_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid resource ID".into()))?;

    let source_id = body
        .source_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid source_id".into()))?;

    validate_resource_fields(
        body.quoted_text.as_deref(),
        body.editor_note.as_deref(),
        body.admin_notes.as_deref(),
        body.source_location_freeform.as_deref(),
        body.source_page_start,
        body.source_page_end,
    )?;

    crate::modules::corpus::bibliography::resources::db::update_resource(
        &state.pool,
        resource_id,
        book_id,
        crate::modules::corpus::bibliography::resources::db::ResourceUpdate {
            resource_type: body.resource_type.as_deref(),
            verbatim_kind: body.verbatim_kind.as_deref(),
            sentence_start: body.sentence_start,
            sentence_end: body.sentence_end,
            sentence_kind: body.sentence_kind.as_deref(),
            source_id,
            source_page_start: body.source_page_start,
            source_page_end: body.source_page_end,
            source_location_freeform: body.source_location_freeform.as_deref(),
            quoted_text: body.quoted_text.as_deref(),
            editor_note: body.editor_note.as_deref(),
            is_featured: body.is_featured,
            admin_notes: body.admin_notes.as_deref(),
        },
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

    crate::modules::corpus::bibliography::resources::db::soft_delete_resource(
        &state.pool,
        resource_id,
        user.id,
    )
    .await?;

    Ok(Json(()))
}
