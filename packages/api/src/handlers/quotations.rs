use axum::Json;
use axum::extract::{Path, Query, State};

use crate::auth::middleware::AuthUser;
use crate::auth::permissions::Permission;
use crate::db;
use crate::error::AppError;
use crate::models::article_quotation::{
    UnifiedListQuery, UnifiedQuotationListResponse, UnifiedQuotationResponse,
};
use crate::models::quotation::{
    CreateNoteRequest, CreateQuotationRequest, CreateQuotationResponse, GlobalListQuery,
    NoteListResponse, NoteResponse, NoteWithContextListResponse, QuotationListResponse,
    QuotationQuery, TagListResponse, UpdateNoteRequest,
};
use crate::state::AppState;
use crate::validation::{
    MAX_NOTE_BODY, MAX_NOTE_TAG_LEN, MAX_NOTE_TAGS, check_count, check_max_len,
};

fn validate_note_fields(body: &str, tags: &[String]) -> Result<(), AppError> {
    check_max_len("Note body", body, MAX_NOTE_BODY)?;
    check_count("Tags", tags, MAX_NOTE_TAGS)?;
    for tag in tags {
        check_max_len("Tag", tag, MAX_NOTE_TAG_LEN)?;
    }
    Ok(())
}

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

    let current = db::quotations::get_user_quotation_count(&state.pool, user.id).await?;
    let max = db::quotations::get_quotation_limit(&user.roles);
    if current >= max as i64 {
        return Err(AppError::BadRequest(format!(
            "Quotation limit reached ({max}). Upgrade your plan to save more quotations."
        )));
    }

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

    validate_note_fields(&body.body, &body.tags)?;

    let current = db::quotations::get_user_note_count(&state.pool, user.id).await?;
    let max = db::quotations::get_note_limit(&user.roles);
    if current >= max as i64 {
        return Err(AppError::BadRequest(format!(
            "Note limit reached ({max}). Upgrade your plan to save more notes."
        )));
    }

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

    if let Some(b) = body.body.as_deref() {
        check_max_len("Note body", b, MAX_NOTE_BODY)?;
    }
    if let Some(tags) = body.tags.as_deref() {
        check_count("Tags", tags, MAX_NOTE_TAGS)?;
        for tag in tags {
            check_max_len("Tag", tag, MAX_NOTE_TAG_LEN)?;
        }
    }

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

/// List all quotations for the authenticated user (books + articles, unified)
#[utoipa::path(
    get,
    path = "/api/quotations",
    params(UnifiedListQuery),
    responses(
        (status = 200, description = "User's quotations across books and articles", body = UnifiedQuotationListResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "quotations"
)]
pub async fn list_all_quotations(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<UnifiedListQuery>,
) -> Result<Json<UnifiedQuotationListResponse>, AppError> {
    user.require_permission(Permission::NotesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let mut quotations: Vec<UnifiedQuotationResponse> = Vec::new();

    let include_books = params.source_type.as_deref().map_or(true, |t| t == "book");
    let include_articles = params
        .source_type
        .as_deref()
        .map_or(true, |t| t == "article");

    if include_books {
        let book_quotations =
            db::quotations::list_all_quotations(&state.pool, user.id, params.book_slug.as_deref())
                .await?;

        for q in book_quotations {
            quotations.push(UnifiedQuotationResponse::Book {
                id: q.id,
                book_slug: q.book_slug,
                book_title: q.book_title,
                node_label: q.node_label,
                node_slug: q.node_slug,
                anchor_sentence_start_number: q.anchor_sentence_start_number,
                anchor_sentence_end_number: q.anchor_sentence_end_number,
                sentence_kind: q.sentence_kind,
                anchor_main_sentence_number: q.anchor_main_sentence_number,
                start_text_snippet: q.start_text_snippet,
                end_text_snippet: q.end_text_snippet,
                note_count: q.note_count,
                created_at: q.created_at,
            });
        }
    }

    if include_articles && params.book_slug.is_none() {
        let article_quotations =
            db::article_quotations::list_article_quotations_for_unified(&state.pool, user.id)
                .await?;

        for q in article_quotations {
            quotations.push(UnifiedQuotationResponse::Article {
                id: q.id.to_string(),
                article_id: q.article_id.map(|id| id.to_string()),
                article_title: q.article_title,
                author_display_name: q.author_display_name,
                text_snippet: truncate_snippet(&q.text, 80),
                note_count: q.note_count.unwrap_or(0),
                created_at: q
                    .created_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            });
        }
    }

    // Sort by created_at DESC
    quotations.sort_by(|a, b| {
        let a_time = match a {
            UnifiedQuotationResponse::Book { created_at, .. } => created_at,
            UnifiedQuotationResponse::Article { created_at, .. } => created_at,
        };
        let b_time = match b {
            UnifiedQuotationResponse::Book { created_at, .. } => created_at,
            UnifiedQuotationResponse::Article { created_at, .. } => created_at,
        };
        b_time.cmp(a_time)
    });

    let limits =
        db::quotations::get_quotation_limits_response(&state.pool, user.id, &user.roles).await?;

    Ok(Json(UnifiedQuotationListResponse { quotations, limits }))
}

fn truncate_snippet(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        let mut end = max_len;
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &text[..end])
    }
}

/// List all notes for the authenticated user (across all books)
#[utoipa::path(
    get,
    path = "/api/notes",
    params(GlobalListQuery),
    responses(
        (status = 200, description = "User's notes across books", body = NoteWithContextListResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "quotations"
)]
pub async fn list_all_notes(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<GlobalListQuery>,
) -> Result<Json<NoteWithContextListResponse>, AppError> {
    user.require_permission(Permission::NotesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let notes =
        db::quotations::list_all_notes(&state.pool, user.id, params.book_slug.as_deref()).await?;

    let limits =
        db::quotations::get_note_limits_response(&state.pool, user.id, &user.roles).await?;

    Ok(Json(NoteWithContextListResponse { notes, limits }))
}
