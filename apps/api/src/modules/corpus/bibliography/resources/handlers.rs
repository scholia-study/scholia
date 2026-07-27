use axum::Json;
use axum::extract::{Path, Query, State};

use crate::modules::corpus::bibliography::models::{
    CreateResourceRequest, ResourceListResponse, ResourceQuery, ResourceReviewStatus,
    ResourceSubmissionListResponse, ResourceSubmissionResponse, ReviewSubmissionRequest,
    SubmissionListQuery, UpdateResourceRequest,
};
use crate::system::auth::middleware::AuthUser;
use crate::system::auth::permissions::Permission;
use crate::system::error::AppError;
use crate::system::state::AppState;
use crate::system::validation::{
    MAX_RESOURCE_ADMIN_NOTES, MAX_RESOURCE_EDITOR_NOTE, MAX_RESOURCE_QUOTED_TEXT,
    MAX_RESOURCE_REVIEW_NOTE, MAX_RESOURCE_SOURCE_LOCATION, MAX_RESOURCE_SOURCE_PAGE,
    MAX_RESOURCE_SUBMISSIONS_PER_DAY, MIN_RESOURCE_SOURCE_PAGE, check_int_range, check_max_len,
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
    maybe_user: Option<AuthUser>,
    Path(slug): Path<String>,
    Query(params): Query<ResourceQuery>,
) -> Result<Json<ResourceListResponse>, AppError> {
    let book_id =
        crate::modules::corpus::reading::books::db::get_book_id_by_slug(&state.pool, &slug).await?;

    // Editor-internal `admin_notes` is only returned to callers who can
    // manage resources. Authenticated editors bypass the CDN cache
    // (session cookie), so the cached anonymous response never carries it.
    // `viewer_id` lets a submitter see their own pending suggestion inline.
    let (viewer_id, include_admin_notes) = match &maybe_user {
        Some(u) => (Some(u.id), u.has_permission(Permission::ResourcesManage)),
        None => (None, false),
    };

    let resources = crate::modules::corpus::bibliography::resources::db::list_resources(
        &state.pool,
        book_id,
        params.start,
        params.end,
        &params.kind,
        viewer_id,
        include_admin_notes,
    )
    .await?;

    Ok(Json(ResourceListResponse { resources }))
}

/// Create a resource. Any authenticated user may submit; editors
/// (`ResourcesManage`) publish immediately, everyone else's suggestion enters
/// the review queue as `pending` until an editor approves it.
#[utoipa::path(
    post,
    path = "/api/books/{slug}/resources",
    params(("slug" = String, Path, description = "Book slug")),
    request_body = CreateResourceRequest,
    responses(
        (status = 200, description = "Resource created or submitted for review", body = ResourceListResponse),
        (status = 400, description = "Invalid input or rate limit reached"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Book not found")
    ),
    tag = "resources"
)]
pub async fn create_resource(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<CreateResourceRequest>,
) -> Result<Json<ResourceListResponse>, AppError> {
    let can_manage = user.has_permission(Permission::ResourcesManage);

    let book_id =
        crate::modules::corpus::reading::books::db::get_book_id_by_slug(&state.pool, &slug).await?;

    let source_id = body
        .source_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid source_id".into()))?;

    // Editor-only fields never originate from a community submission; ignore
    // them on that path rather than trusting the client.
    let (is_featured, admin_notes) = if can_manage {
        (
            body.is_featured.unwrap_or(false),
            body.admin_notes.as_deref(),
        )
    } else {
        (false, None)
    };

    validate_resource_fields(
        body.quoted_text.as_deref(),
        body.editor_note.as_deref(),
        admin_notes,
        body.source_location_freeform.as_deref(),
        body.source_page_start,
        body.source_page_end,
    )?;

    // Non-editors go through review; rate-limit to bound spam (auth is the real
    // gate). Their `submitted_by` also lets them see the pending row inline.
    let (review_status, submitted_by) = if can_manage {
        ("approved", None)
    } else {
        let recent =
            crate::modules::corpus::bibliography::resources::db::count_recent_submissions_by_user(
                &state.pool,
                user.id,
            )
            .await?;
        if recent >= MAX_RESOURCE_SUBMISSIONS_PER_DAY {
            return Err(AppError::BadRequest(format!(
                "Submission rate limit reached ({MAX_RESOURCE_SUBMISSIONS_PER_DAY} per 24h). Try again later."
            )));
        }
        ("pending", Some(user.id))
    };

    crate::modules::corpus::bibliography::resources::db::create_resource(
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
            is_featured,
            admin_notes,
            review_status,
            submitted_by,
            scope: body.scope.as_deref().unwrap_or("work"),
        },
    )
    .await?;

    // Re-fetch the range in context. A submitter sees their new pending row
    // (viewer_id == user.id); admin_notes are included only for editors.
    let resources = crate::modules::corpus::bibliography::resources::db::list_resources(
        &state.pool,
        book_id,
        body.sentence_start,
        body.sentence_end.unwrap_or(body.sentence_start),
        &body.sentence_kind,
        Some(user.id),
        can_manage,
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

    // source_id is nullable and clearable: preserve absent vs explicit-null.
    let source_id = match &body.source_id {
        None => None,
        Some(None) => Some(None),
        Some(Some(s)) => {
            Some(Some(uuid::Uuid::parse_str(s).map_err(|_| {
                AppError::BadRequest("Invalid source_id".into())
            })?))
        }
    };

    // Validate only the values being SET; a clear (Some(None)) has nothing to check.
    validate_resource_fields(
        body.quoted_text.as_ref().and_then(|o| o.as_deref()),
        body.editor_note.as_ref().and_then(|o| o.as_deref()),
        body.admin_notes.as_ref().and_then(|o| o.as_deref()),
        body.source_location_freeform
            .as_ref()
            .and_then(|o| o.as_deref()),
        body.source_page_start.flatten(),
        body.source_page_end.flatten(),
    )?;

    crate::modules::corpus::bibliography::resources::db::update_resource(
        &state.pool,
        resource_id,
        book_id,
        crate::modules::corpus::bibliography::resources::db::ResourceUpdate {
            resource_type: body.resource_type.as_deref(),
            verbatim_kind: body.verbatim_kind.as_ref().map(|o| o.as_deref()),
            sentence_start: body.sentence_start,
            sentence_end: body.sentence_end,
            sentence_kind: body.sentence_kind.as_deref(),
            source_id,
            source_page_start: body.source_page_start,
            source_page_end: body.source_page_end,
            source_location_freeform: body.source_location_freeform.as_ref().map(|o| o.as_deref()),
            quoted_text: body.quoted_text.as_ref().map(|o| o.as_deref()),
            editor_note: body.editor_note.as_ref().map(|o| o.as_deref()),
            is_featured: body.is_featured,
            admin_notes: body.admin_notes.as_ref().map(|o| o.as_deref()),
            scope: body.scope.as_deref(),
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
        (status = 400, description = "Invalid resource ID"),
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

/// List community resource submissions for review (editors + admins).
#[utoipa::path(
    get,
    path = "/api/admin/resource-submissions",
    params(SubmissionListQuery),
    responses(
        (status = 200, description = "Submission queue", body = ResourceSubmissionListResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "resources"
)]
pub async fn list_resource_submissions(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<SubmissionListQuery>,
) -> Result<Json<ResourceSubmissionListResponse>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let statuses = parse_submission_filter(params.filter.as_deref());
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(25).clamp(1, 100);

    let list = crate::modules::corpus::bibliography::resources::db::list_submissions(
        &state.pool,
        &statuses,
        page,
        per_page,
    )
    .await?;
    Ok(Json(list))
}

/// Approve or reject a community submission (editors + admins).
#[utoipa::path(
    patch,
    path = "/api/admin/resource-submissions/{id}",
    params(("id" = String, Path, description = "Submission (resource) ID")),
    request_body = ReviewSubmissionRequest,
    responses(
        (status = 200, description = "Submission reviewed", body = ResourceSubmissionResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Submission not found")
    ),
    tag = "resources"
)]
pub async fn review_resource_submission(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<ReviewSubmissionRequest>,
) -> Result<Json<ResourceSubmissionResponse>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    if !matches!(
        body.status,
        ResourceReviewStatus::Approved | ResourceReviewStatus::Rejected
    ) {
        return Err(AppError::BadRequest(
            "A review must set status to 'approved' or 'rejected'".into(),
        ));
    }

    if let Some(note) = body.review_note.as_deref() {
        check_max_len("Review note", note, MAX_RESOURCE_REVIEW_NOTE)?;
    }

    let submission_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid submission ID".into()))?;

    let reviewed = crate::modules::corpus::bibliography::resources::db::review_submission(
        &state.pool,
        submission_id,
        user.id,
        body.status,
        body.review_note.as_deref(),
    )
    .await?;
    Ok(Json(reviewed))
}

fn parse_submission_filter(filter: Option<&str>) -> Vec<ResourceReviewStatus> {
    match filter {
        None | Some("pending") => vec![ResourceReviewStatus::Pending],
        Some("approved") => vec![ResourceReviewStatus::Approved],
        Some("rejected") => vec![ResourceReviewStatus::Rejected],
        Some("all") => vec![
            ResourceReviewStatus::Pending,
            ResourceReviewStatus::Approved,
            ResourceReviewStatus::Rejected,
        ],
        Some(_) => vec![ResourceReviewStatus::Pending],
    }
}
