use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::feedback::models::{
    FeedbackHandler, FeedbackListResponse, FeedbackResponse, FeedbackStatus, FeedbackSubmitter,
};
use crate::system::error::{AppError, SqlxResultExt};

fn fmt_time(t: time::OffsetDateTime) -> String {
    t.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

struct FeedbackRow {
    id: Uuid,
    user_id: Option<Uuid>,
    submitter_display_name: Option<String>,
    submitter_email: Option<String>,
    body: String,
    url: Option<String>,
    user_agent: Option<String>,
    viewport_w: Option<i32>,
    viewport_h: Option<i32>,
    status: FeedbackStatus,
    admin_notes: Option<String>,
    handled_by: Option<Uuid>,
    handler_display_name: Option<String>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

fn feedback_from_row(r: FeedbackRow) -> FeedbackResponse {
    let submitter = match (r.user_id, r.submitter_display_name, r.submitter_email) {
        (Some(id), Some(dn), Some(em)) => Some(FeedbackSubmitter {
            id: id.to_string(),
            display_name: dn,
            email: em,
        }),
        _ => None,
    };
    let handled_by = match (r.handled_by, r.handler_display_name) {
        (Some(id), Some(dn)) => Some(FeedbackHandler {
            id: id.to_string(),
            display_name: dn,
        }),
        _ => None,
    };
    FeedbackResponse {
        id: r.id.to_string(),
        submitter,
        body: r.body,
        url: r.url,
        user_agent: r.user_agent,
        viewport_w: r.viewport_w,
        viewport_h: r.viewport_h,
        status: r.status,
        admin_notes: r.admin_notes,
        handled_by,
        created_at: fmt_time(r.created_at),
        updated_at: fmt_time(r.updated_at),
    }
}

/// Count feedback rows submitted by `user_id` in the last 24 hours.
/// Used by the rate-limit gate at the create endpoint.
pub async fn count_feedback_last_24h(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
    let count: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM feedback
           WHERE user_id = $1
             AND created_at > now() - INTERVAL '24 hours'"#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn create_feedback(
    pool: &PgPool,
    user_id: Uuid,
    body: &str,
    url: Option<&str>,
    user_agent: Option<&str>,
    viewport_w: Option<i32>,
    viewport_h: Option<i32>,
) -> Result<FeedbackResponse, AppError> {
    let id = sqlx::query_scalar!(
        r#"INSERT INTO feedback (user_id, body, url, user_agent, viewport_w, viewport_h)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id"#,
        user_id,
        body,
        url,
        user_agent,
        viewport_w,
        viewport_h,
    )
    .fetch_one(pool)
    .await?;
    get_feedback(pool, id).await
}

pub async fn get_feedback(pool: &PgPool, id: Uuid) -> Result<FeedbackResponse, AppError> {
    let row = sqlx::query_as!(
        FeedbackRow,
        r#"SELECT f.id,
                  f.user_id,
                  u.display_name      AS "submitter_display_name?",
                  u.email             AS "submitter_email?",
                  f.body,
                  f.url,
                  f.user_agent,
                  f.viewport_w,
                  f.viewport_h,
                  f.status            AS "status!: FeedbackStatus",
                  f.admin_notes,
                  f.handled_by,
                  h.display_name      AS "handler_display_name?",
                  f.created_at,
                  f.updated_at
           FROM feedback f
           LEFT JOIN users u ON u.id = f.user_id
           LEFT JOIN users h ON h.id = f.handled_by
           WHERE f.id = $1"#,
        id,
    )
    .fetch_one(pool)
    .await
    .on_missing(|| AppError::NotFound("Feedback not found".into()))?;
    Ok(feedback_from_row(row))
}

pub async fn list_feedback(
    pool: &PgPool,
    statuses: &[FeedbackStatus],
    page: u32,
    per_page: u32,
) -> Result<FeedbackListResponse, AppError> {
    let offset = ((page.saturating_sub(1)) as i64) * per_page as i64;
    let limit = per_page as i64;

    // sqlx doesn't bind enum arrays cleanly via the macro path; cast in SQL.
    let status_strs: Vec<String> = statuses
        .iter()
        .map(|s| status_str(*s).to_string())
        .collect();

    let rows = sqlx::query_as!(
        FeedbackRow,
        r#"SELECT f.id,
                  f.user_id,
                  u.display_name      AS "submitter_display_name?",
                  u.email             AS "submitter_email?",
                  f.body,
                  f.url,
                  f.user_agent,
                  f.viewport_w,
                  f.viewport_h,
                  f.status            AS "status!: FeedbackStatus",
                  f.admin_notes,
                  f.handled_by,
                  h.display_name      AS "handler_display_name?",
                  f.created_at,
                  f.updated_at
           FROM feedback f
           LEFT JOIN users u ON u.id = f.user_id
           LEFT JOIN users h ON h.id = f.handled_by
           WHERE f.status::TEXT = ANY($1)
           ORDER BY f.created_at DESC
           LIMIT $2 OFFSET $3"#,
        &status_strs,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM feedback
           WHERE status::TEXT = ANY($1)"#,
        &status_strs,
    )
    .fetch_one(pool)
    .await?;

    Ok(FeedbackListResponse {
        feedback: rows.into_iter().map(feedback_from_row).collect(),
        total,
        page,
        per_page,
    })
}

pub async fn update_feedback(
    pool: &PgPool,
    id: Uuid,
    admin_id: Uuid,
    status: Option<FeedbackStatus>,
    admin_notes: Option<&str>,
) -> Result<FeedbackResponse, AppError> {
    // No-op: skip the write if neither field changed.
    if status.is_none() && admin_notes.is_none() {
        return get_feedback(pool, id).await;
    }

    let status_str_opt = status.map(status_str);

    let result = sqlx::query!(
        r#"UPDATE feedback
           SET status      = COALESCE($2::feedback_status, status),
               admin_notes = COALESCE($3, admin_notes),
               handled_by  = $4,
               updated_at  = now()
           WHERE id = $1"#,
        id,
        status_str_opt as Option<&str>,
        admin_notes,
        admin_id,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Feedback not found".into()));
    }

    get_feedback(pool, id).await
}

fn status_str(s: FeedbackStatus) -> &'static str {
    match s {
        FeedbackStatus::Todo => "todo",
        FeedbackStatus::InProgress => "in_progress",
        FeedbackStatus::Done => "done",
        FeedbackStatus::Cancelled => "cancelled",
    }
}
