use axum::Json;
use axum::extract::{Path, Query, State};

use crate::auth::middleware::AuthUser;
use crate::auth::permissions::Permission;
use crate::db;
use crate::error::AppError;
use crate::models::quotation::{
    CreateNoteRequest, CreateQuotationRequest, CreateQuotationResponse, NoteListResponse,
    NoteResponse, QuotationListResponse, QuotationQuery, TagListResponse, UpdateNoteRequest,
};
use crate::state::AppState;

/// List quotations for a node (authenticated user only)
#[utoipa::path(
    get,
    path = "/api/books/{slug}/quotations",
    params(
        ("slug" = String, Path, description = "Book slug"),
        QuotationQuery,
    ),
    responses(
        (status = 200, description = "User's quotations for the node", body = QuotationListResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Book not found")
    ),
    tag = "quotations"
)]
pub async fn list_quotations(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Query(params): Query<QuotationQuery>,
) -> Result<Json<QuotationListResponse>, AppError> {
    user.require_permission(Permission::NotesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let book_id = db::books::get_book_id_by_slug(&state.pool, &slug).await?;
    let node_id = uuid::Uuid::parse_str(&params.node_id)
        .map_err(|_| AppError::BadRequest("Invalid node_id".into()))?;

    let quotations =
        db::quotations::list_quotations_for_node(&state.pool, user.id, book_id, node_id).await?;

    Ok(Json(QuotationListResponse { quotations }))
}

/// Save a quotation (returns existing if duplicate)
#[utoipa::path(
    post,
    path = "/api/books/{slug}/quotations",
    params(("slug" = String, Path, description = "Book slug")),
    request_body = CreateQuotationRequest,
    responses(
        (status = 200, description = "Quotation saved", body = CreateQuotationResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated")
    ),
    tag = "quotations"
)]
pub async fn create_quotation(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<CreateQuotationRequest>,
) -> Result<Json<CreateQuotationResponse>, AppError> {
    user.require_permission(Permission::NotesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let book_id = db::books::get_book_id_by_slug(&state.pool, &slug).await?;

    let (quotation, created) = db::quotations::create_quotation(
        &state.pool,
        user.id,
        book_id,
        body.sentence_start,
        body.sentence_end,
        &body.sentence_kind,
    )
    .await?;

    Ok(Json(CreateQuotationResponse { quotation, created }))
}

/// Delete a quotation (cascade-deletes notes)
#[utoipa::path(
    delete,
    path = "/api/books/{slug}/quotations/{id}",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ("id" = String, Path, description = "Quotation ID")
    ),
    responses(
        (status = 200, description = "Quotation deleted"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Quotation not found")
    ),
    tag = "quotations"
)]
pub async fn delete_quotation(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_slug, id)): Path<(String, String)>,
) -> Result<Json<()>, AppError> {
    user.require_permission(Permission::NotesDelete)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let quotation_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid quotation ID".into()))?;

    db::quotations::delete_quotation(&state.pool, quotation_id, user.id).await?;

    Ok(Json(()))
}

/// List notes for a quotation
#[utoipa::path(
    get,
    path = "/api/books/{slug}/quotations/{id}/notes",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ("id" = String, Path, description = "Quotation ID")
    ),
    responses(
        (status = 200, description = "Notes for the quotation", body = NoteListResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Quotation not found")
    ),
    tag = "quotations"
)]
pub async fn list_notes(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_slug, id)): Path<(String, String)>,
) -> Result<Json<NoteListResponse>, AppError> {
    user.require_permission(Permission::NotesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let quotation_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid quotation ID".into()))?;

    // Verify ownership
    let (owner_id, _) = db::quotations::get_quotation_owner(&state.pool, quotation_id).await?;
    if owner_id != user.id {
        return Err(AppError::NotFound("Quotation not found".into()));
    }

    let notes = db::quotations::list_notes(&state.pool, quotation_id).await?;

    Ok(Json(NoteListResponse { notes }))
}

/// Create a note on a quotation
#[utoipa::path(
    post,
    path = "/api/books/{slug}/quotations/{id}/notes",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ("id" = String, Path, description = "Quotation ID")
    ),
    request_body = CreateNoteRequest,
    responses(
        (status = 200, description = "Note created", body = NoteResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Quotation not found")
    ),
    tag = "quotations"
)]
pub async fn create_note(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_slug, id)): Path<(String, String)>,
    Json(body): Json<CreateNoteRequest>,
) -> Result<Json<NoteResponse>, AppError> {
    user.require_permission(Permission::NotesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let quotation_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid quotation ID".into()))?;

    // Verify ownership
    let (owner_id, _) = db::quotations::get_quotation_owner(&state.pool, quotation_id).await?;
    if owner_id != user.id {
        return Err(AppError::NotFound("Quotation not found".into()));
    }

    let note =
        db::quotations::create_note(&state.pool, user.id, quotation_id, &body.body, &body.tags)
            .await?;

    Ok(Json(note))
}

/// Update a note
#[utoipa::path(
    put,
    path = "/api/books/{slug}/quotations/{id}/notes/{note_id}",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ("id" = String, Path, description = "Quotation ID"),
        ("note_id" = String, Path, description = "Note ID")
    ),
    request_body = UpdateNoteRequest,
    responses(
        (status = 200, description = "Note updated"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Note not found")
    ),
    tag = "quotations"
)]
pub async fn update_note(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_slug, _quotation_id, note_id_str)): Path<(String, String, String)>,
    Json(body): Json<UpdateNoteRequest>,
) -> Result<Json<()>, AppError> {
    user.require_permission(Permission::NotesEdit)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let note_id = uuid::Uuid::parse_str(&note_id_str)
        .map_err(|_| AppError::BadRequest("Invalid note ID".into()))?;

    db::quotations::update_note(
        &state.pool,
        note_id,
        user.id,
        body.body.as_deref(),
        body.tags.as_deref(),
    )
    .await?;

    Ok(Json(()))
}

/// Delete a note
#[utoipa::path(
    delete,
    path = "/api/books/{slug}/quotations/{id}/notes/{note_id}",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ("id" = String, Path, description = "Quotation ID"),
        ("note_id" = String, Path, description = "Note ID")
    ),
    responses(
        (status = 200, description = "Note deleted"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Note not found")
    ),
    tag = "quotations"
)]
pub async fn delete_note(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_slug, _quotation_id, note_id_str)): Path<(String, String, String)>,
) -> Result<Json<()>, AppError> {
    user.require_permission(Permission::NotesDelete)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let note_id = uuid::Uuid::parse_str(&note_id_str)
        .map_err(|_| AppError::BadRequest("Invalid note ID".into()))?;

    db::quotations::delete_note(&state.pool, note_id, user.id).await?;

    Ok(Json(()))
}

/// List all tags for the authenticated user
#[utoipa::path(
    get,
    path = "/api/tags",
    responses(
        (status = 200, description = "User's tags", body = TagListResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "tags"
)]
pub async fn list_tags(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<TagListResponse>, AppError> {
    user.require_permission(Permission::NotesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let tags = db::quotations::list_tags(&state.pool, user.id).await?;

    Ok(Json(TagListResponse { tags }))
}
