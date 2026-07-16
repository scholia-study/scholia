use axum::Json;
use axum::extract::{Path, Query, State};

use crate::modules::feedback::models::{
    CreateFeedbackRequest, FeedbackListQuery, FeedbackListResponse, FeedbackResponse,
    FeedbackStatus, UpdateFeedbackRequest,
};
use crate::system::auth::middleware::AuthUser;
use crate::system::auth::permissions::Permission;
use crate::system::error::AppError;
use crate::system::state::AppState;
use crate::system::validation::{
    MAX_FEEDBACK_ADMIN_NOTES, MAX_FEEDBACK_BODY, MAX_FEEDBACK_PER_DAY, MAX_FEEDBACK_URL,
    MAX_FEEDBACK_USER_AGENT, MIN_FEEDBACK_BODY, check_max_len,
};

/// Submit feedback to admins. Auth-required; rate-limited per user.
#[utoipa::path(
    post,
    path = "/api/feedback",
    request_body = CreateFeedbackRequest,
    responses(
        (status = 200, description = "Feedback submitted", body = FeedbackResponse),
        (status = 400, description = "Invalid input or rate limit reached"),
        (status = 401, description = "Not authenticated")
    ),
    tag = "feedback"
)]
pub async fn create_feedback(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateFeedbackRequest>,
) -> Result<Json<FeedbackResponse>, AppError> {
    let trimmed = body.body.trim();
    if trimmed.chars().count() < MIN_FEEDBACK_BODY {
        return Err(AppError::BadRequest(format!(
            "Feedback must be at least {MIN_FEEDBACK_BODY} characters."
        )));
    }
    check_max_len("Feedback body", trimmed, MAX_FEEDBACK_BODY)?;
    if let Some(u) = body.url.as_deref() {
        check_max_len("URL", u, MAX_FEEDBACK_URL)?;
    }
    if let Some(ua) = body.user_agent.as_deref() {
        check_max_len("User agent", ua, MAX_FEEDBACK_USER_AGENT)?;
    }

    // Rate limit: bound runaway clients / angry users. Not a real abuse
    // defence — auth is.
    let recent =
        crate::modules::feedback::db::count_feedback_last_24h(&state.pool, user.id).await?;
    if recent >= MAX_FEEDBACK_PER_DAY {
        return Err(AppError::BadRequest(format!(
            "Feedback rate limit reached ({MAX_FEEDBACK_PER_DAY} per 24h). Try again later."
        )));
    }

    let feedback = crate::modules::feedback::db::create_feedback(
        &state.pool,
        user.id,
        trimmed,
        body.url.as_deref(),
        body.user_agent.as_deref(),
        body.viewport_w,
        body.viewport_h,
    )
    .await?;
    Ok(Json(feedback))
}

/// List feedback (admin only). Default filter is "active" (todo + in_progress).
#[utoipa::path(
    get,
    path = "/api/admin/feedback",
    params(FeedbackListQuery),
    responses(
        (status = 200, description = "Feedback list", body = FeedbackListResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Not an admin")
    ),
    tag = "feedback"
)]
pub async fn list_feedback(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<FeedbackListQuery>,
) -> Result<Json<FeedbackListResponse>, AppError> {
    require_admin(&user)?;

    let statuses = parse_filter(params.filter.as_deref());
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(25).clamp(1, 100);

    let list =
        crate::modules::feedback::db::list_feedback(&state.pool, &statuses, page, per_page).await?;
    Ok(Json(list))
}

/// Get a single feedback row (admin only).
#[utoipa::path(
    get,
    path = "/api/admin/feedback/{id}",
    params(("id" = String, Path, description = "Feedback ID")),
    responses(
        (status = 200, description = "Feedback detail", body = FeedbackResponse),
        (status = 400, description = "Invalid feedback ID"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Not found / not an admin")
    ),
    tag = "feedback"
)]
pub async fn get_feedback(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<FeedbackResponse>, AppError> {
    require_admin(&user)?;
    let id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid feedback ID".into()))?;
    let f = crate::modules::feedback::db::get_feedback(&state.pool, id).await?;
    Ok(Json(f))
}

/// Update status and/or admin notes (admin only).
#[utoipa::path(
    patch,
    path = "/api/admin/feedback/{id}",
    params(("id" = String, Path, description = "Feedback ID")),
    request_body = UpdateFeedbackRequest,
    responses(
        (status = 200, description = "Feedback updated", body = FeedbackResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Not found / not an admin")
    ),
    tag = "feedback"
)]
pub async fn update_feedback(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<UpdateFeedbackRequest>,
) -> Result<Json<FeedbackResponse>, AppError> {
    require_admin(&user)?;
    let id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid feedback ID".into()))?;

    if let Some(notes) = body.admin_notes.as_deref() {
        check_max_len("Admin notes", notes, MAX_FEEDBACK_ADMIN_NOTES)?;
    }

    let f = crate::modules::feedback::db::update_feedback(
        &state.pool,
        id,
        user.id,
        body.status,
        body.admin_notes.as_deref(),
    )
    .await?;
    Ok(Json(f))
}

/// Reject non-admins as if the route doesn't exist (don't signal that
/// `/api/admin/*` endpoints are real to non-admin clients).
fn require_admin(user: &AuthUser) -> Result<(), AppError> {
    user.require_permission(Permission::AdminPanel)
        .map_err(|_| AppError::NotFound("Not found".into()))
}

fn parse_filter(filter: Option<&str>) -> Vec<FeedbackStatus> {
    match filter {
        None | Some("active") => vec![FeedbackStatus::Todo, FeedbackStatus::InProgress],
        Some("all") => vec![
            FeedbackStatus::Todo,
            FeedbackStatus::InProgress,
            FeedbackStatus::Done,
            FeedbackStatus::Cancelled,
        ],
        Some("todo") => vec![FeedbackStatus::Todo],
        Some("in_progress") => vec![FeedbackStatus::InProgress],
        Some("done") => vec![FeedbackStatus::Done],
        Some("cancelled") => vec![FeedbackStatus::Cancelled],
        Some(_) => vec![FeedbackStatus::Todo, FeedbackStatus::InProgress],
    }
}
