use axum::Json;
use axum::extract::{Path, Query, State};
use uuid::Uuid;

use crate::modules::writing::article_reviews::db;
use crate::modules::writing::article_reviews::models::{
    ArticleReviewActivityResponse, ArticleReviewCommentListResponse, ArticleReviewCommentResponse,
    ArticleReviewDetailResponse, ArticleReviewMessageListResponse, ArticleReviewMessageResponse,
    ArticleReviewQueueResponse, ArticleReviewRequestResponse, CreateReviewCommentRequest,
    CreateReviewMessageRequest, CreateReviewReplyRequest, CreateReviewRequestRequest,
    ReviewArticleMeta, ReviewDecisionRequest, ReviewQueueQuery, UpdateReviewCommentRequest,
};
use crate::modules::writing::article_reviews::snapshot::annotate_snapshot_html;
use crate::system::auth::middleware::AuthUser;
use crate::system::auth::permissions::Permission;
use crate::system::cache;
use crate::system::error::AppError;
use crate::system::state::AppState;
use crate::system::validation::{
    MAX_REVIEW_COMMENT, MAX_REVIEW_MESSAGE, MAX_REVIEW_POSTS_PER_DAY, MAX_REVIEW_QUOTED_TEXT,
    MAX_REVIEW_REQUESTS_PER_DAY, check_max_len,
};

/// The label applied when a publication-intent request is approved.
const APPROVAL_LABEL_SLUG: &str = "imprimatur";

fn parse_uuid(id: &str, what: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(id).map_err(|_| AppError::BadRequest(format!("Invalid {what} ID")))
}

fn is_reviewer(user: &AuthUser) -> bool {
    user.has_permission(Permission::ArticlesReview)
}

/// Access rule for a request's review surface: the article's author
/// always; reviewers unless the request was withdrawn (withdrawing
/// un-shares the draft). Failures are 404 so review URLs don't leak the
/// existence of someone's draft.
fn require_request_access(user: &AuthUser, req: &db::RequestWithArticle) -> Result<(), AppError> {
    if req.author_user_id == user.id || (is_reviewer(user) && req.status != "withdrawn") {
        return Ok(());
    }
    Err(AppError::NotFound("Review request not found".into()))
}

fn check_body(field: &str, body: &str, max: usize) -> Result<(), AppError> {
    if body.trim().is_empty() {
        return Err(AppError::BadRequest(format!("{field} cannot be empty")));
    }
    check_max_len(field, body, max)
}

/// Shared guard for the message/comment/reply write paths.
async fn check_post_rate_limit(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    let recent = db::count_recent_posts_by_user(&state.pool, user_id).await?;
    if recent >= MAX_REVIEW_POSTS_PER_DAY {
        return Err(AppError::BadRequest(
            "Daily message limit reached. Try again tomorrow.".into(),
        ));
    }
    Ok(())
}

/// Submit an article for review (feedback or publication hand-off).
#[utoipa::path(
    post,
    path = "/api/user/articles/{slug}/review-requests",
    params(("slug" = String, Path, description = "Article slug")),
    request_body = CreateReviewRequestRequest,
    responses(
        (status = 200, description = "Review request created", body = ArticleReviewRequestResponse),
        (status = 400, description = "Invalid input or article not eligible"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Article not found"),
        (status = 409, description = "A review request is already pending")
    ),
    tag = "article-reviews"
)]
pub async fn create_review_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<CreateReviewRequestRequest>,
) -> Result<Json<ArticleReviewRequestResponse>, AppError> {
    if !matches!(body.intent.as_str(), "feedback" | "publication") {
        return Err(AppError::BadRequest(
            "Intent must be 'feedback' or 'publication'".into(),
        ));
    }
    if let Some(message) = &body.message {
        check_max_len("Message", message, MAX_REVIEW_MESSAGE)?;
    }

    let article = db::get_article_for_submission(&state.pool, &slug, user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("Article not found".into()))?;
    if article.status == "archived" {
        return Err(AppError::BadRequest(
            "Archived articles cannot be submitted for review".into(),
        ));
    }

    let recent = db::count_recent_requests_by_user(&state.pool, user.id).await?;
    if recent >= MAX_REVIEW_REQUESTS_PER_DAY {
        return Err(AppError::BadRequest(
            "Daily review request limit reached. Try again tomorrow.".into(),
        ));
    }

    let rendered = crate::modules::writing::articles::db::render_article_markdown(
        &state.pool,
        &state.config.frontend_url,
        &article.markdown,
    )
    .await;
    let snapshot_html = annotate_snapshot_html(&rendered);

    let request = db::create_request(
        &state.pool,
        db::ReviewRequestCreate {
            article_id: article.id,
            intent: &body.intent,
            snapshot_markdown: &article.markdown,
            snapshot_html: &snapshot_html,
        },
    )
    .await?;

    if let Some(message) = body
        .message
        .as_deref()
        .map(str::trim)
        .filter(|m| !m.is_empty())
    {
        db::create_message(
            &state.pool,
            db::MessageCreate {
                article_id: article.id,
                sender_id: user.id,
                body: message,
            },
        )
        .await?;
    }

    Ok(Json(request))
}

/// Withdraw a pending review request (author only). Withdrawing the
/// article's only request revokes editor access to the draft.
#[utoipa::path(
    post,
    path = "/api/user/review-requests/{id}/withdraw",
    params(("id" = String, Path, description = "Review request ID")),
    responses(
        (status = 200, description = "Request withdrawn"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "No pending request found")
    ),
    tag = "article-reviews"
)]
pub async fn withdraw_review_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<()>, AppError> {
    let request_id = parse_uuid(&id, "review request")?;
    let withdrawn = db::withdraw_request(&state.pool, request_id, user.id).await?;
    if !withdrawn {
        return Err(AppError::NotFound("No pending request found".into()));
    }
    Ok(Json(()))
}

/// Review page payload: the request, its frozen snapshot, article
/// metadata, and sibling rounds. Author or reviewer only (404 otherwise).
#[utoipa::path(
    get,
    path = "/api/review/requests/{id}",
    params(("id" = String, Path, description = "Review request ID")),
    responses(
        (status = 200, description = "Review detail", body = ArticleReviewDetailResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Review request not found")
    ),
    tag = "article-reviews"
)]
pub async fn get_review_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<ArticleReviewDetailResponse>, AppError> {
    let request_id = parse_uuid(&id, "review request")?;
    let req = db::get_request(&state.pool, request_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Review request not found".into()))?;
    require_request_access(&user, &req)?;

    let is_owner = req.author_user_id == user.id;
    let requests = db::list_requests_for_article(&state.pool, req.article_id, is_owner).await?;
    let labels = crate::modules::writing::articles::editorial_labels::list_for_article(
        &state.pool,
        req.article_id,
    )
    .await?;

    Ok(Json(ArticleReviewDetailResponse {
        request: req.to_request_response(),
        snapshot_html: req.snapshot_html.clone(),
        article: ReviewArticleMeta {
            id: req.article_id.to_string(),
            title: req.article_title.clone(),
            slug: req.article_slug.clone(),
            status: req.article_status.clone(),
            author_user_id: req.author_user_id.to_string(),
            author_display_name: req.author_display_name.clone(),
            author_handle: req.author_handle.clone(),
            labels,
        },
        requests,
        draft_changed: req.article_updated_at > req.submitted_at,
    }))
}

/// List the comment threads on a request's snapshot.
#[utoipa::path(
    get,
    path = "/api/review/requests/{id}/comments",
    params(("id" = String, Path, description = "Review request ID")),
    responses(
        (status = 200, description = "Comment threads", body = ArticleReviewCommentListResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Review request not found")
    ),
    tag = "article-reviews"
)]
pub async fn list_review_comments(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<ArticleReviewCommentListResponse>, AppError> {
    let request_id = parse_uuid(&id, "review request")?;
    let req = db::get_request(&state.pool, request_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Review request not found".into()))?;
    require_request_access(&user, &req)?;

    let comments = db::list_comments(&state.pool, request_id).await?;
    Ok(Json(ArticleReviewCommentListResponse { comments }))
}

/// Create an anchored comment on a request's snapshot (reviewers only).
#[utoipa::path(
    post,
    path = "/api/review/requests/{id}/comments",
    params(("id" = String, Path, description = "Review request ID")),
    request_body = CreateReviewCommentRequest,
    responses(
        (status = 200, description = "Comment created", body = ArticleReviewCommentResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Review request not found")
    ),
    tag = "article-reviews"
)]
pub async fn create_review_comment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<CreateReviewCommentRequest>,
) -> Result<Json<ArticleReviewCommentResponse>, AppError> {
    let request_id = parse_uuid(&id, "review request")?;
    let req = db::get_request(&state.pool, request_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Review request not found".into()))?;
    if !is_reviewer(&user) || req.status == "withdrawn" {
        return Err(AppError::NotFound("Review request not found".into()));
    }

    check_body("Comment", &body.body, MAX_REVIEW_COMMENT)?;
    check_post_rate_limit(&state, user.id).await?;
    if body.block_index < 0 {
        return Err(AppError::BadRequest("Invalid block index".into()));
    }
    match (body.sentence_start, body.sentence_end) {
        (None, None) => {}
        (Some(start), Some(end)) if start >= 0 && end >= start => {}
        _ => return Err(AppError::BadRequest("Invalid sentence range".into())),
    }
    if let Some(quoted) = &body.quoted_text {
        check_max_len("Quoted text", quoted, MAX_REVIEW_QUOTED_TEXT)?;
    }

    let comment = db::create_comment(
        &state.pool,
        db::CommentCreate {
            request_id,
            parent_id: None,
            sender_id: user.id,
            block_index: Some(body.block_index),
            sentence_start: body.sentence_start,
            sentence_end: body.sentence_end,
            quoted_text: body.quoted_text.as_deref(),
            body: body.body.trim(),
        },
    )
    .await?;
    Ok(Json(comment))
}

/// Reply within a comment thread (author or reviewer).
#[utoipa::path(
    post,
    path = "/api/review/comments/{id}/replies",
    params(("id" = String, Path, description = "Comment ID")),
    request_body = CreateReviewReplyRequest,
    responses(
        (status = 200, description = "Reply created", body = ArticleReviewCommentResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Comment not found")
    ),
    tag = "article-reviews"
)]
pub async fn create_review_reply(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<CreateReviewReplyRequest>,
) -> Result<Json<ArticleReviewCommentResponse>, AppError> {
    let comment_id = parse_uuid(&id, "comment")?;
    let ctx = db::get_comment_context(&state.pool, comment_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Comment not found".into()))?;
    if ctx.parent_id.is_some() {
        return Err(AppError::BadRequest("Replies cannot be nested".into()));
    }
    let can_access =
        ctx.author_user_id == user.id || (is_reviewer(&user) && ctx.request_status != "withdrawn");
    if !can_access {
        return Err(AppError::NotFound("Comment not found".into()));
    }
    check_body("Reply", &body.body, MAX_REVIEW_COMMENT)?;
    check_post_rate_limit(&state, user.id).await?;

    let reply = db::create_comment(
        &state.pool,
        db::CommentCreate {
            request_id: ctx.request_id,
            parent_id: Some(ctx.id),
            sender_id: user.id,
            block_index: None,
            sentence_start: None,
            sentence_end: None,
            quoted_text: None,
            body: body.body.trim(),
        },
    )
    .await?;
    Ok(Json(reply))
}

/// Resolve or reopen a comment thread (reviewers only).
#[utoipa::path(
    patch,
    path = "/api/review/comments/{id}",
    params(("id" = String, Path, description = "Comment ID")),
    request_body = UpdateReviewCommentRequest,
    responses(
        (status = 200, description = "Comment updated", body = ArticleReviewCommentResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Comment not found")
    ),
    tag = "article-reviews"
)]
pub async fn update_review_comment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<UpdateReviewCommentRequest>,
) -> Result<Json<ArticleReviewCommentResponse>, AppError> {
    user.require_permission(Permission::ArticlesReview)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;
    let comment_id = parse_uuid(&id, "comment")?;

    let updated = db::set_comment_resolved(
        &state.pool,
        comment_id,
        db::CommentResolvePatch {
            resolved: body.resolved,
            resolved_by: user.id,
        },
    )
    .await?
    .ok_or_else(|| AppError::NotFound("Comment not found".into()))?;
    Ok(Json(updated))
}

/// Access rule for an article's review channel: the author, or a
/// reviewer while at least one non-withdrawn request exists.
async fn require_channel_access(
    state: &AppState,
    user: &AuthUser,
    article_id: Uuid,
) -> Result<(), AppError> {
    let owner = db::get_article_owner(&state.pool, article_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Article not found".into()))?;
    if owner == user.id
        || (is_reviewer(user) && db::editors_have_access(&state.pool, article_id).await?)
    {
        return Ok(());
    }
    Err(AppError::NotFound("Article not found".into()))
}

/// The article's review channel, shared across review rounds.
#[utoipa::path(
    get,
    path = "/api/review/articles/{article_id}/messages",
    params(("article_id" = String, Path, description = "Article ID")),
    responses(
        (status = 200, description = "Channel messages", body = ArticleReviewMessageListResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Article not found")
    ),
    tag = "article-reviews"
)]
pub async fn list_review_messages(
    State(state): State<AppState>,
    user: AuthUser,
    Path(article_id): Path<String>,
) -> Result<Json<ArticleReviewMessageListResponse>, AppError> {
    let article_id = parse_uuid(&article_id, "article")?;
    require_channel_access(&state, &user, article_id).await?;
    let messages = db::list_messages(&state.pool, article_id).await?;
    Ok(Json(ArticleReviewMessageListResponse { messages }))
}

/// Change-detection stamp for the review page's polling loop. The
/// client compares the whole payload and invalidates its request,
/// comment, and message queries when it changes.
#[utoipa::path(
    get,
    path = "/api/review/articles/{article_id}/activity",
    params(("article_id" = String, Path, description = "Article ID")),
    responses(
        (status = 200, description = "Activity stamp", body = ArticleReviewActivityResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Article not found")
    ),
    tag = "article-reviews"
)]
pub async fn get_review_activity(
    State(state): State<AppState>,
    user: AuthUser,
    Path(article_id): Path<String>,
) -> Result<Json<ArticleReviewActivityResponse>, AppError> {
    let article_id = parse_uuid(&article_id, "article")?;
    require_channel_access(&state, &user, article_id).await?;
    let stamp = db::get_activity_stamp(&state.pool, article_id).await?;
    Ok(Json(ArticleReviewActivityResponse {
        messages: stamp.messages,
        comments: stamp.comments,
        resolved_comments: stamp.resolved_comments,
        requests_updated_at: stamp
            .requests_updated_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
    }))
}

/// Post to the article's review channel (author or reviewer). The
/// channel exists once the article has been submitted at least once.
#[utoipa::path(
    post,
    path = "/api/review/articles/{article_id}/messages",
    params(("article_id" = String, Path, description = "Article ID")),
    request_body = CreateReviewMessageRequest,
    responses(
        (status = 200, description = "Message posted", body = ArticleReviewMessageResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Article not found")
    ),
    tag = "article-reviews"
)]
pub async fn create_review_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path(article_id): Path<String>,
    Json(body): Json<CreateReviewMessageRequest>,
) -> Result<Json<ArticleReviewMessageResponse>, AppError> {
    let article_id = parse_uuid(&article_id, "article")?;
    require_channel_access(&state, &user, article_id).await?;
    if !db::any_request_exists(&state.pool, article_id).await? {
        return Err(AppError::BadRequest(
            "The review channel opens when the article is first submitted for review".into(),
        ));
    }
    check_body("Message", &body.body, MAX_REVIEW_MESSAGE)?;
    check_post_rate_limit(&state, user.id).await?;

    let message = db::create_message(
        &state.pool,
        db::MessageCreate {
            article_id,
            sender_id: user.id,
            body: body.body.trim(),
        },
    )
    .await?;
    Ok(Json(message))
}

/// The editor decision matrix: publication requests are approved or
/// declined; feedback requests close as resolved. Everything else
/// (including any attempt to set `pending` or `withdrawn`) is invalid.
fn decision_is_valid(intent: &str, status: &str) -> bool {
    matches!(
        (intent, status),
        ("publication", "approved") | ("publication", "declined") | ("feedback", "resolved")
    )
}

fn parse_queue_filter(filter: Option<&str>) -> Result<Vec<String>, AppError> {
    let statuses: &[&str] = match filter.unwrap_or("pending") {
        "pending" => &["pending"],
        "approved" => &["approved"],
        "declined" => &["declined"],
        "resolved" => &["resolved"],
        // Withdrawn rounds never appear in the editor queue.
        "all" => &["pending", "approved", "declined", "resolved"],
        _ => return Err(AppError::BadRequest("Invalid filter".into())),
    };
    Ok(statuses.iter().map(|s| (*s).to_string()).collect())
}

/// Editor queue of review requests.
#[utoipa::path(
    get,
    path = "/api/admin/article-review-requests",
    params(ReviewQueueQuery),
    responses(
        (status = 200, description = "Review queue", body = ArticleReviewQueueResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "article-reviews"
)]
pub async fn list_article_review_queue(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<ReviewQueueQuery>,
) -> Result<Json<ArticleReviewQueueResponse>, AppError> {
    user.require_permission(Permission::ArticlesReview)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let statuses = parse_queue_filter(params.filter.as_deref())?;
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(25).clamp(1, 100);

    let (items, total) = db::list_queue(&state.pool, &statuses, page, per_page).await?;
    Ok(Json(ArticleReviewQueueResponse { items, total }))
}

/// Decide a pending request. Approving a publication-intent request
/// publishes the article (if still a draft) and applies the Imprimatur
/// label; feedback-intent requests close as `resolved`.
#[utoipa::path(
    patch,
    path = "/api/admin/article-review-requests/{id}",
    params(("id" = String, Path, description = "Review request ID")),
    request_body = ReviewDecisionRequest,
    responses(
        (status = 200, description = "Request decided", body = ArticleReviewRequestResponse),
        (status = 400, description = "Invalid decision for this request"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Review request not found"),
        (status = 409, description = "Request is no longer pending")
    ),
    tag = "article-reviews"
)]
pub async fn decide_article_review(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<ReviewDecisionRequest>,
) -> Result<Json<ArticleReviewRequestResponse>, AppError> {
    user.require_permission(Permission::ArticlesReview)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;
    let request_id = parse_uuid(&id, "review request")?;
    if let Some(message) = &body.message {
        check_max_len("Message", message, MAX_REVIEW_MESSAGE)?;
    }

    let req = db::get_request(&state.pool, request_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Review request not found".into()))?;
    if req.status != "pending" {
        return Err(AppError::Conflict("Request is no longer pending".into()));
    }

    if !decision_is_valid(&req.intent, &body.status) {
        return Err(AppError::BadRequest(format!(
            "'{}' is not a valid decision for a {} request",
            body.status, req.intent
        )));
    }

    // Side effects before claiming the request: each is idempotent, so a
    // failure leaves the request pending and the decision retryable.
    let newly_published = body.status == "approved" && req.article_status == "draft";
    if newly_published {
        crate::modules::writing::articles::db::publish_article_by_id(&state.pool, req.article_id)
            .await?;
    }
    if body.status == "approved" {
        crate::modules::writing::articles::editorial_labels::apply_label(
            &state.pool,
            &req.article_slug,
            APPROVAL_LABEL_SLUG,
            user.id,
        )
        .await?;
    }

    let decided = db::decide_request(
        &state.pool,
        request_id,
        db::ReviewDecision {
            status: &body.status,
            reviewed_by: user.id,
        },
    )
    .await?;
    if !decided {
        return Err(AppError::Conflict("Request is no longer pending".into()));
    }

    if body.status == "approved" {
        let mut paths =
            crate::modules::writing::articles::handlers::article_cache_paths(&req.article_slug);
        if newly_published {
            paths.extend(crate::modules::writing::articles::handlers::sitemap_cache_paths());
        }
        cache::invalidate(&state, paths);
    }

    if let Some(message) = body
        .message
        .as_deref()
        .map(str::trim)
        .filter(|m| !m.is_empty())
    {
        db::create_message(
            &state.pool,
            db::MessageCreate {
                article_id: req.article_id,
                sender_id: user.id,
                body: message,
            },
        )
        .await?;
    }

    let refreshed = db::get_request(&state.pool, request_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Review request not found".into()))?;
    Ok(Json(refreshed.to_request_response()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_matrix() {
        assert!(decision_is_valid("publication", "approved"));
        assert!(decision_is_valid("publication", "declined"));
        assert!(decision_is_valid("feedback", "resolved"));

        assert!(!decision_is_valid("feedback", "approved"));
        assert!(!decision_is_valid("feedback", "declined"));
        assert!(!decision_is_valid("publication", "resolved"));
        assert!(!decision_is_valid("publication", "pending"));
        assert!(!decision_is_valid("publication", "withdrawn"));
        assert!(!decision_is_valid("feedback", "withdrawn"));
    }

    #[test]
    fn queue_filter_excludes_withdrawn() {
        let all = parse_queue_filter(Some("all")).unwrap();
        assert!(!all.contains(&"withdrawn".to_string()));
        assert_eq!(all.len(), 4);
        assert_eq!(parse_queue_filter(None).unwrap(), vec!["pending"]);
        assert!(parse_queue_filter(Some("bogus")).is_err());
    }
}
