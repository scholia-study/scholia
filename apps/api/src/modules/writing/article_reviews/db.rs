use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::writing::article_reviews::models::{
    ArticleReviewCommentResponse, ArticleReviewMessageResponse, ArticleReviewQueueItem,
    ArticleReviewRequestResponse, ReviewParticipant,
};
use crate::system::error::AppError;

fn fmt_time(t: time::OffsetDateTime) -> String {
    t.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| t.to_string())
}

fn participant(
    user_id: Option<Uuid>,
    display_name: Option<String>,
    handle: Option<String>,
) -> Option<ReviewParticipant> {
    Some(ReviewParticipant {
        user_id: user_id?.to_string(),
        display_name: display_name?,
        handle,
    })
}

struct RequestRow {
    id: Uuid,
    article_id: Uuid,
    intent: String,
    status: String,
    submitted_at: time::OffsetDateTime,
    resolved_at: Option<time::OffsetDateTime>,
}

fn request_response(r: RequestRow) -> ArticleReviewRequestResponse {
    ArticleReviewRequestResponse {
        id: r.id.to_string(),
        article_id: r.article_id.to_string(),
        intent: r.intent,
        status: r.status,
        submitted_at: fmt_time(r.submitted_at),
        resolved_at: r.resolved_at.map(fmt_time),
    }
}

/// The article fields the submission handler needs, resolved by slug +
/// owner in one query.
pub struct SubmissionArticle {
    pub id: Uuid,
    pub status: String,
    pub markdown: String,
}

pub async fn get_article_for_submission(
    pool: &PgPool,
    slug: &str,
    user_id: Uuid,
) -> Result<Option<SubmissionArticle>, AppError> {
    let row = sqlx::query_as!(
        SubmissionArticle,
        r#"SELECT id, status::TEXT AS "status!", markdown
           FROM articles WHERE slug = $1 AND user_id = $2"#,
        slug,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn get_article_owner(pool: &PgPool, article_id: Uuid) -> Result<Option<Uuid>, AppError> {
    let row = sqlx::query_scalar!(r#"SELECT user_id FROM articles WHERE id = $1"#, article_id,)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub struct ReviewRequestCreate<'a> {
    pub article_id: Uuid,
    pub intent: &'a str,
    pub snapshot_markdown: &'a str,
    pub snapshot_html: &'a str,
}

pub async fn create_request(
    pool: &PgPool,
    entry: ReviewRequestCreate<'_>,
) -> Result<ArticleReviewRequestResponse, AppError> {
    let row = sqlx::query_as!(
        RequestRow,
        r#"INSERT INTO article_review_requests
               (article_id, intent, snapshot_markdown, snapshot_html)
           VALUES ($1, $2::article_review_intent, $3, $4)
           RETURNING id, article_id,
               intent::TEXT AS "intent!", status::TEXT AS "status!",
               submitted_at, resolved_at"#,
        entry.article_id,
        entry.intent as _,
        entry.snapshot_markdown,
        entry.snapshot_html,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            AppError::Conflict("A review request for this article is already pending".into())
        }
        _ => AppError::from(e),
    })?;

    Ok(request_response(row))
}

/// Review requests this user submitted in the last 24h (for rate limiting).
pub async fn count_recent_requests_by_user(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM article_review_requests arr
           JOIN articles a ON a.id = arr.article_id
           WHERE a.user_id = $1 AND arr.submitted_at > now() - INTERVAL '24 hours'"#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// Channel messages plus comments/replies this user wrote in the last
/// 24h (for the posting rate limit).
pub async fn count_recent_posts_by_user(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
    let count = sqlx::query_scalar!(
        r#"SELECT
               (SELECT COUNT(*) FROM article_review_messages
                WHERE sender_id = $1
                  AND created_at > now() - INTERVAL '24 hours')
             + (SELECT COUNT(*) FROM article_review_comments
                WHERE sender_id = $1
                  AND created_at > now() - INTERVAL '24 hours')
               AS "count!""#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// A request joined with the article fields access checks and the review
/// page need.
pub struct RequestWithArticle {
    pub id: Uuid,
    pub article_id: Uuid,
    pub intent: String,
    pub status: String,
    pub submitted_at: time::OffsetDateTime,
    pub resolved_at: Option<time::OffsetDateTime>,
    pub snapshot_html: String,
    pub article_title: String,
    pub article_slug: String,
    pub article_status: String,
    pub article_updated_at: time::OffsetDateTime,
    pub author_user_id: Uuid,
    pub author_display_name: String,
    pub author_handle: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub assignee_display_name: Option<String>,
    pub assignee_handle: Option<String>,
}

impl RequestWithArticle {
    pub fn assignee(&self) -> Option<ReviewParticipant> {
        participant(
            self.assigned_to,
            self.assignee_display_name.clone(),
            self.assignee_handle.clone(),
        )
    }

    pub fn to_request_response(&self) -> ArticleReviewRequestResponse {
        ArticleReviewRequestResponse {
            id: self.id.to_string(),
            article_id: self.article_id.to_string(),
            intent: self.intent.clone(),
            status: self.status.clone(),
            submitted_at: fmt_time(self.submitted_at),
            resolved_at: self.resolved_at.map(fmt_time),
        }
    }
}

pub async fn get_request(
    pool: &PgPool,
    request_id: Uuid,
) -> Result<Option<RequestWithArticle>, AppError> {
    let row = sqlx::query_as!(
        RequestWithArticle,
        r#"SELECT arr.id, arr.article_id,
               arr.intent::TEXT AS "intent!", arr.status::TEXT AS "status!",
               arr.submitted_at, arr.resolved_at, arr.snapshot_html,
               a.title AS article_title, a.slug AS article_slug,
               a.status::TEXT AS "article_status!",
               a.updated_at AS article_updated_at,
               a.user_id AS author_user_id,
               u.display_name AS author_display_name,
               u.handle AS author_handle,
               arr.assigned_to,
               au.display_name AS "assignee_display_name?",
               au.handle AS "assignee_handle?"
           FROM article_review_requests arr
           JOIN articles a ON a.id = arr.article_id
           JOIN users u ON u.id = a.user_id
           LEFT JOIN users au ON au.id = arr.assigned_to
           WHERE arr.id = $1"#,
        request_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// All rounds for an article, newest first. `include_withdrawn` is true
/// for the author's own view only.
pub async fn list_requests_for_article(
    pool: &PgPool,
    article_id: Uuid,
    include_withdrawn: bool,
) -> Result<Vec<ArticleReviewRequestResponse>, AppError> {
    let rows = sqlx::query_as!(
        RequestRow,
        r#"SELECT id, article_id,
               intent::TEXT AS "intent!", status::TEXT AS "status!",
               submitted_at, resolved_at
           FROM article_review_requests
           WHERE article_id = $1 AND ($2 OR status <> 'withdrawn')
           ORDER BY submitted_at DESC"#,
        article_id,
        include_withdrawn,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(request_response).collect())
}

/// Whether editors have access to this article's review surface: at
/// least one non-withdrawn request exists. Withdrawing the only request
/// un-shares the draft.
pub async fn editors_have_access(pool: &PgPool, article_id: Uuid) -> Result<bool, AppError> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(
               SELECT 1 FROM article_review_requests
               WHERE article_id = $1 AND status <> 'withdrawn'
           ) AS "exists!""#,
        article_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// Whether any request exists at all — the channel comes into existence
/// with the first submission.
pub async fn any_request_exists(pool: &PgPool, article_id: Uuid) -> Result<bool, AppError> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(
               SELECT 1 FROM article_review_requests WHERE article_id = $1
           ) AS "exists!""#,
        article_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// The owner-facing request pointers decorating article DTOs: the
/// pending request (if any) and the most recent request of any status.
pub async fn request_ids_for_article(
    pool: &PgPool,
    article_id: Uuid,
) -> Result<(Option<Uuid>, Option<Uuid>), AppError> {
    let row = sqlx::query!(
        r#"SELECT
               (SELECT id FROM article_review_requests
                WHERE article_id = $1 AND status = 'pending') AS pending,
               (SELECT id FROM article_review_requests
                WHERE article_id = $1
                ORDER BY submitted_at DESC LIMIT 1) AS latest"#,
        article_id,
    )
    .fetch_one(pool)
    .await?;
    Ok((row.pending, row.latest))
}

/// Pending request ids for a batch of articles (the author's dashboard
/// listing). Articles without a pending request are absent.
pub async fn pending_requests_for_articles(
    pool: &PgPool,
    article_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, Uuid>, AppError> {
    if article_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let rows = sqlx::query!(
        r#"SELECT article_id, id FROM article_review_requests
           WHERE article_id = ANY($1) AND status = 'pending'"#,
        article_ids,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| (r.article_id, r.id)).collect())
}

/// Author withdraws a pending request. Returns false when no pending
/// request with this id belongs to this author.
pub async fn withdraw_request(
    pool: &PgPool,
    request_id: Uuid,
    author_id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        r#"UPDATE article_review_requests arr
           SET status = 'withdrawn', resolved_at = now(), updated_at = now()
           FROM articles a
           WHERE arr.id = $1 AND arr.status = 'pending'
             AND a.id = arr.article_id AND a.user_id = $2"#,
        request_id,
        author_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Withdraw any pending request on this article (author archived it).
pub async fn withdraw_pending_for_article(pool: &PgPool, article_id: Uuid) -> Result<(), AppError> {
    sqlx::query!(
        r#"UPDATE article_review_requests
           SET status = 'withdrawn', resolved_at = now(), updated_at = now()
           WHERE article_id = $1 AND status = 'pending'"#,
        article_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Assign or unassign an editor on a pending request. Returns false
/// when the request is not pending (or doesn't exist).
pub async fn assign_request(
    pool: &PgPool,
    request_id: Uuid,
    patch: Option<Uuid>,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        r#"UPDATE article_review_requests
           SET assigned_to = $2, updated_at = now()
           WHERE id = $1 AND status = 'pending'"#,
        request_id,
        patch,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Users who hold a reviewer role (admin/editor), for the assignment
/// dropdown. Role names are the source of truth for the
/// `articles_review` permission (`Role::permissions`).
pub async fn list_reviewers(pool: &PgPool) -> Result<Vec<ReviewParticipant>, AppError> {
    struct ReviewerRow {
        id: Uuid,
        display_name: String,
        handle: Option<String>,
    }
    let rows = sqlx::query_as!(
        ReviewerRow,
        r#"SELECT DISTINCT u.id, u.display_name, u.handle
           FROM users u
           JOIN user_roles ur ON ur.user_id = u.id
           JOIN roles r ON r.id = ur.role_id
           WHERE r.name IN ('admin', 'editor')
           ORDER BY u.display_name"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ReviewParticipant {
            user_id: r.id.to_string(),
            display_name: r.display_name,
            handle: r.handle,
        })
        .collect())
}

/// Whether this user holds a reviewer role — guards assigning requests
/// to someone who couldn't act on them.
pub async fn user_is_reviewer(pool: &PgPool, user_id: Uuid) -> Result<bool, AppError> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(
               SELECT 1 FROM user_roles ur
               JOIN roles r ON r.id = ur.role_id
               WHERE ur.user_id = $1 AND r.name IN ('admin', 'editor')
           ) AS "exists!""#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

pub struct ReviewDecision<'a> {
    pub status: &'a str,
    pub reviewed_by: Uuid,
}

/// Editor decision on a pending request. Returns false when the request
/// is no longer pending (or doesn't exist).
pub async fn decide_request(
    pool: &PgPool,
    request_id: Uuid,
    patch: ReviewDecision<'_>,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        r#"UPDATE article_review_requests
           SET status = $2::article_review_request_status,
               reviewed_by = $3, resolved_at = now(), updated_at = now()
           WHERE id = $1 AND status = 'pending'"#,
        request_id,
        patch.status as _,
        patch.reviewed_by,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Assignee scoping for the queue: everyone, one reviewer, or only
/// unclaimed requests.
pub enum AssigneeFilter {
    Any,
    User(Uuid),
    Unassigned,
}

pub async fn list_queue(
    pool: &PgPool,
    statuses: &[String],
    assignee: &AssigneeFilter,
    page: i32,
    per_page: i32,
) -> Result<(Vec<ArticleReviewQueueItem>, i64), AppError> {
    let offset = i64::from((page - 1) * per_page);
    let limit = i64::from(per_page);
    // Two params encode the three-way filter: ($a IS NULL OR match) for
    // a specific reviewer, and a bool for unassigned-only.
    let (assignee_id, unassigned_only) = match assignee {
        AssigneeFilter::Any => (None, false),
        AssigneeFilter::User(id) => (Some(*id), false),
        AssigneeFilter::Unassigned => (None, true),
    };

    struct QueueRow {
        id: Uuid,
        article_id: Uuid,
        article_title: String,
        article_slug: String,
        article_status: String,
        author_user_id: Uuid,
        author_display_name: String,
        author_handle: Option<String>,
        intent: String,
        status: String,
        submitted_at: time::OffsetDateTime,
        resolved_at: Option<time::OffsetDateTime>,
        open_comment_count: i64,
        assigned_to: Option<Uuid>,
        assignee_display_name: Option<String>,
        assignee_handle: Option<String>,
    }

    let rows = sqlx::query_as!(
        QueueRow,
        r#"SELECT arr.id, arr.article_id,
               a.title AS article_title, a.slug AS article_slug,
               a.status::TEXT AS "article_status!",
               a.user_id AS author_user_id,
               u.display_name AS author_display_name,
               u.handle AS author_handle,
               arr.intent::TEXT AS "intent!", arr.status::TEXT AS "status!",
               arr.submitted_at, arr.resolved_at,
               (SELECT COUNT(*) FROM article_review_comments arc
                WHERE arc.request_id = arr.id
                  AND arc.parent_id IS NULL
                  AND arc.resolved_at IS NULL) AS "open_comment_count!",
               arr.assigned_to,
               au.display_name AS "assignee_display_name?",
               au.handle AS "assignee_handle?"
           FROM article_review_requests arr
           JOIN articles a ON a.id = arr.article_id
           JOIN users u ON u.id = a.user_id
           LEFT JOIN users au ON au.id = arr.assigned_to
           WHERE arr.status::TEXT = ANY($1)
             AND ($4::uuid IS NULL OR arr.assigned_to = $4)
             AND (NOT $5 OR arr.assigned_to IS NULL)
           ORDER BY arr.submitted_at DESC
           LIMIT $2 OFFSET $3"#,
        statuses,
        limit,
        offset,
        assignee_id,
        unassigned_only,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM article_review_requests
           WHERE status::TEXT = ANY($1)
             AND ($2::uuid IS NULL OR assigned_to = $2)
             AND (NOT $3 OR assigned_to IS NULL)"#,
        statuses,
        assignee_id,
        unassigned_only,
    )
    .fetch_one(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|r| ArticleReviewQueueItem {
            id: r.id.to_string(),
            article_id: r.article_id.to_string(),
            article_title: r.article_title,
            article_slug: r.article_slug,
            article_status: r.article_status,
            author_user_id: r.author_user_id.to_string(),
            author_display_name: r.author_display_name,
            author_handle: r.author_handle,
            intent: r.intent,
            status: r.status,
            submitted_at: fmt_time(r.submitted_at),
            resolved_at: r.resolved_at.map(fmt_time),
            open_comment_count: r.open_comment_count,
            assignee: participant(r.assigned_to, r.assignee_display_name, r.assignee_handle),
        })
        .collect();

    Ok((items, total))
}

pub struct ActivityStamp {
    pub messages: i64,
    pub comments: i64,
    pub resolved_comments: i64,
    pub requests_updated_at: time::OffsetDateTime,
}

pub async fn get_activity_stamp(
    pool: &PgPool,
    article_id: Uuid,
) -> Result<ActivityStamp, AppError> {
    let row = sqlx::query_as!(
        ActivityStamp,
        r#"SELECT
               (SELECT COUNT(*) FROM article_review_messages
                WHERE article_id = $1) AS "messages!",
               (SELECT COUNT(*) FROM article_review_comments c
                JOIN article_review_requests r ON r.id = c.request_id
                WHERE r.article_id = $1) AS "comments!",
               (SELECT COUNT(*) FROM article_review_comments c
                JOIN article_review_requests r ON r.id = c.request_id
                WHERE r.article_id = $1 AND c.resolved_at IS NOT NULL)
                   AS "resolved_comments!",
               (SELECT COALESCE(MAX(updated_at), 'epoch'::timestamptz)
                FROM article_review_requests
                WHERE article_id = $1) AS "requests_updated_at!""#,
        article_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

struct MessageRow {
    id: Uuid,
    sender_id: Option<Uuid>,
    sender_display_name: Option<String>,
    sender_handle: Option<String>,
    body: String,
    created_at: time::OffsetDateTime,
}

fn message_response(r: MessageRow) -> ArticleReviewMessageResponse {
    ArticleReviewMessageResponse {
        id: r.id.to_string(),
        sender: participant(r.sender_id, r.sender_display_name, r.sender_handle),
        body: r.body,
        created_at: fmt_time(r.created_at),
    }
}

pub async fn list_messages(
    pool: &PgPool,
    article_id: Uuid,
) -> Result<Vec<ArticleReviewMessageResponse>, AppError> {
    let rows = sqlx::query_as!(
        MessageRow,
        r#"SELECT m.id, m.sender_id,
               u.display_name AS "sender_display_name?",
               u.handle AS "sender_handle?",
               m.body, m.created_at
           FROM article_review_messages m
           LEFT JOIN users u ON u.id = m.sender_id
           WHERE m.article_id = $1
           ORDER BY m.created_at"#,
        article_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(message_response).collect())
}

pub struct MessageCreate<'a> {
    pub article_id: Uuid,
    pub sender_id: Uuid,
    pub body: &'a str,
}

pub async fn create_message(
    pool: &PgPool,
    entry: MessageCreate<'_>,
) -> Result<ArticleReviewMessageResponse, AppError> {
    let row = sqlx::query_as!(
        MessageRow,
        r#"WITH inserted AS (
               INSERT INTO article_review_messages (article_id, sender_id, body)
               VALUES ($1, $2, $3)
               RETURNING id, sender_id, body, created_at
           )
           SELECT i.id, i.sender_id,
               u.display_name AS "sender_display_name?",
               u.handle AS "sender_handle?",
               i.body, i.created_at
           FROM inserted i
           LEFT JOIN users u ON u.id = i.sender_id"#,
        entry.article_id,
        entry.sender_id,
        entry.body,
    )
    .fetch_one(pool)
    .await?;
    Ok(message_response(row))
}

struct CommentRow {
    id: Uuid,
    request_id: Uuid,
    parent_id: Option<Uuid>,
    sender_id: Option<Uuid>,
    sender_display_name: Option<String>,
    sender_handle: Option<String>,
    block_index: Option<i32>,
    sentence_start: Option<i32>,
    sentence_end: Option<i32>,
    quoted_text: Option<String>,
    body: String,
    resolved_at: Option<time::OffsetDateTime>,
    created_at: time::OffsetDateTime,
}

fn comment_response(r: CommentRow) -> ArticleReviewCommentResponse {
    ArticleReviewCommentResponse {
        id: r.id.to_string(),
        request_id: r.request_id.to_string(),
        parent_id: r.parent_id.map(|id| id.to_string()),
        sender: participant(r.sender_id, r.sender_display_name, r.sender_handle),
        block_index: r.block_index,
        sentence_start: r.sentence_start,
        sentence_end: r.sentence_end,
        quoted_text: r.quoted_text,
        body: r.body,
        resolved_at: r.resolved_at.map(fmt_time),
        created_at: fmt_time(r.created_at),
    }
}

pub async fn list_comments(
    pool: &PgPool,
    request_id: Uuid,
) -> Result<Vec<ArticleReviewCommentResponse>, AppError> {
    let rows = sqlx::query_as!(
        CommentRow,
        r#"SELECT c.id, c.request_id, c.parent_id, c.sender_id,
               u.display_name AS "sender_display_name?",
               u.handle AS "sender_handle?",
               c.block_index, c.sentence_start, c.sentence_end,
               c.quoted_text, c.body, c.resolved_at, c.created_at
           FROM article_review_comments c
           LEFT JOIN users u ON u.id = c.sender_id
           WHERE c.request_id = $1
           ORDER BY c.created_at"#,
        request_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(comment_response).collect())
}

pub struct CommentCreate<'a> {
    pub request_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub block_index: Option<i32>,
    pub sentence_start: Option<i32>,
    pub sentence_end: Option<i32>,
    pub quoted_text: Option<&'a str>,
    pub body: &'a str,
}

pub async fn create_comment(
    pool: &PgPool,
    entry: CommentCreate<'_>,
) -> Result<ArticleReviewCommentResponse, AppError> {
    let row = sqlx::query_as!(
        CommentRow,
        r#"WITH inserted AS (
               INSERT INTO article_review_comments
                   (request_id, parent_id, sender_id, block_index,
                    sentence_start, sentence_end, quoted_text, body)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING id, request_id, parent_id, sender_id, block_index,
                   sentence_start, sentence_end, quoted_text, body,
                   resolved_at, created_at
           )
           SELECT i.id, i.request_id, i.parent_id, i.sender_id,
               u.display_name AS "sender_display_name?",
               u.handle AS "sender_handle?",
               i.block_index, i.sentence_start, i.sentence_end,
               i.quoted_text, i.body, i.resolved_at, i.created_at
           FROM inserted i
           LEFT JOIN users u ON u.id = i.sender_id"#,
        entry.request_id,
        entry.parent_id,
        entry.sender_id,
        entry.block_index,
        entry.sentence_start,
        entry.sentence_end,
        entry.quoted_text,
        entry.body,
    )
    .fetch_one(pool)
    .await?;
    Ok(comment_response(row))
}

/// A comment row with enough context for access checks: the request it
/// belongs to (parent's request for replies) and the article's author.
pub struct CommentContext {
    pub id: Uuid,
    pub request_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub request_status: String,
    pub author_user_id: Uuid,
}

pub async fn get_comment_context(
    pool: &PgPool,
    comment_id: Uuid,
) -> Result<Option<CommentContext>, AppError> {
    let row = sqlx::query_as!(
        CommentContext,
        r#"SELECT c.id, c.request_id, c.parent_id,
               arr.status::TEXT AS "request_status!",
               a.user_id AS author_user_id
           FROM article_review_comments c
           JOIN article_review_requests arr ON arr.id = c.request_id
           JOIN articles a ON a.id = arr.article_id
           WHERE c.id = $1"#,
        comment_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub struct CommentResolvePatch {
    pub resolved: bool,
    pub resolved_by: Uuid,
}

pub async fn set_comment_resolved(
    pool: &PgPool,
    comment_id: Uuid,
    patch: CommentResolvePatch,
) -> Result<Option<ArticleReviewCommentResponse>, AppError> {
    let row = sqlx::query_as!(
        CommentRow,
        r#"WITH updated AS (
               UPDATE article_review_comments
               SET resolved_at = CASE WHEN $2 THEN now() ELSE NULL END,
                   resolved_by = CASE WHEN $2 THEN $3::uuid ELSE NULL END
               WHERE id = $1 AND parent_id IS NULL
               RETURNING id, request_id, parent_id, sender_id, block_index,
                   sentence_start, sentence_end, quoted_text, body,
                   resolved_at, created_at
           )
           SELECT up.id, up.request_id, up.parent_id, up.sender_id,
               u.display_name AS "sender_display_name?",
               u.handle AS "sender_handle?",
               up.block_index, up.sentence_start, up.sentence_end,
               up.quoted_text, up.body, up.resolved_at, up.created_at
           FROM updated up
           LEFT JOIN users u ON u.id = up.sender_id"#,
        comment_id,
        patch.resolved,
        patch.resolved_by,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(comment_response))
}
