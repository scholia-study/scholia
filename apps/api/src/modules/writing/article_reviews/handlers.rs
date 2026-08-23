use axum::Json;
use axum::extract::{Path, Query, State};
use uuid::Uuid;

use crate::modules::writing::article_reviews::db;
use crate::modules::writing::article_reviews::models::{
    ArticleReviewActivityResponse, ArticleReviewCommentListResponse, ArticleReviewCommentResponse,
    ArticleReviewDetailResponse, ArticleReviewMessageListResponse, ArticleReviewMessageResponse,
    ArticleReviewQueueResponse, ArticleReviewRequestResponse, AssignReviewRequest, ChannelQuery,
    CreateReviewCommentRequest, CreateReviewMessageRequest, CreateReviewReplyRequest,
    CreateReviewRequestRequest, ReviewArticleMeta, ReviewCollegiumMeta, ReviewDecision,
    ReviewDecisionRequest, ReviewIntent, ReviewQueueQuery, ReviewRequestStatus,
    ReviewerListResponse, UpdateReviewCommentRequest,
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

/// Whether `user` acts as a reviewer for a request's audience: for
/// editorial requests an `ArticlesReview` holder; for collegium requests a
/// live member — every member when the request is member-visible, collegium
/// admins only otherwise (the classroom mode). Editors have no standing
/// on collegium reviews, and vice versa.
async fn is_audience_reviewer(
    state: &AppState,
    user: &AuthUser,
    collegium_id: Option<Uuid>,
    member_visible: Option<bool>,
) -> Result<bool, AppError> {
    match collegium_id {
        Some(collegium_id) => Ok(
            match crate::modules::collegia::member_role(&state.pool, collegium_id, user.id).await? {
                None => false,
                Some(role) => {
                    member_visible.unwrap_or(true)
                        || role == crate::modules::collegia::CollegiumRole::Steward
                }
            },
        ),
        None => Ok(is_reviewer(user)),
    }
}

/// Access rule for a request's review surface: the article's author
/// always; the audience's reviewers unless the request was withdrawn
/// (withdrawing un-shares the draft). Failures are 404 so review URLs
/// don't leak the existence of someone's draft.
async fn require_request_access(
    state: &AppState,
    user: &AuthUser,
    req: &db::RequestWithArticle,
) -> Result<(), AppError> {
    if req.author_user_id == user.id
        || (req.status != ReviewRequestStatus::Withdrawn
            && is_audience_reviewer(state, user, req.collegium_id, req.member_visible).await?)
    {
        return Ok(());
    }
    Err(AppError::NotFound("Review request not found".into()))
}

fn parse_channel_collegium(channel: &ChannelQuery) -> Result<Option<Uuid>, AppError> {
    channel
        .collegium_id
        .as_deref()
        .filter(|id| !id.is_empty())
        .map(|id| parse_uuid(id, "collegium"))
        .transpose()
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
    let (collegium_id, member_visible) = match &body.collegium_id {
        Some(id) => {
            if body.intent != ReviewIntent::Feedback {
                return Err(AppError::BadRequest(
                    "Collegium reviews are feedback-only".into(),
                ));
            }
            let collegium_id = parse_uuid(id, "collegium")?;
            // Same 404 a non-member gets elsewhere — membership required,
            // and private collegia must not leak.
            crate::modules::collegia::member_role(&state.pool, collegium_id, user.id)
                .await?
                .ok_or_else(|| AppError::NotFound("Collegium not found".into()))?;
            // Snapshot the collegium's review visibility onto the request so
            // later setting flips never expose past submissions.
            let visibility = crate::modules::collegia::review_visibility(&state.pool, collegium_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Collegium not found".into()))?;
            (
                Some(collegium_id),
                Some(visibility == crate::modules::collegia::ReviewVisibility::Members),
            )
        }
        None => (None, None),
    };
    if let Some(message) = &body.message {
        check_max_len("Message", message, MAX_REVIEW_MESSAGE)?;
    }

    let article = db::get_article_for_submission(&state.pool, &slug, user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("Article not found".into()))?;
    if article.status == crate::modules::writing::articles::models::ArticleStatus::Archived {
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
            intent: body.intent,
            collegium_id,
            member_visible,
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
                collegium_id,
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

/// Close a pending collegium-review round. Who may close follows the
/// round's visibility snapshot: writing-collegium rounds (member-visible)
/// close by the author or a collegium steward; classroom rounds (stewards-only)
/// close by a collegium steward alone — the student can only withdraw.
/// Editorial rounds are decided by editors instead.
#[utoipa::path(
    post,
    path = "/api/user/review-requests/{id}/resolve",
    params(("id" = String, Path, description = "Review request ID")),
    responses(
        (status = 200, description = "Request resolved"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "No pending collegium review request found")
    ),
    tag = "article-reviews"
)]
pub async fn resolve_review_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<()>, AppError> {
    let request_id = parse_uuid(&id, "review request")?;
    let not_found = || AppError::NotFound("No pending collegium review request found".into());

    let req = db::get_request(&state.pool, request_id)
        .await?
        .ok_or_else(not_found)?;
    let collegium_id = req.collegium_id.ok_or_else(not_found)?;
    let is_author = req.author_user_id == user.id;
    let is_collegium_steward =
        crate::modules::collegia::member_role(&state.pool, collegium_id, user.id).await?
            == Some(crate::modules::collegia::CollegiumRole::Steward);
    let allowed = if req.member_visible.unwrap_or(true) {
        is_author || is_collegium_steward
    } else {
        is_collegium_steward
    };
    if !allowed {
        return Err(not_found());
    }

    let reviewed_by = is_collegium_steward.then_some(user.id);
    let resolved = db::resolve_collegium_request(&state.pool, request_id, reviewed_by).await?;
    if !resolved {
        return Err(not_found());
    }
    Ok(Json(()))
}

/// A collegium's workshop queue: pending review requests submitted to it.
/// Group members only (404 otherwise, so private collegia don't leak).
#[utoipa::path(
    get,
    path = "/api/collegia/{slug}/review-requests",
    params(("slug" = String, Path, description = "Collegium slug")),
    responses(
        (status = 200, description = "Pending collegium review requests", body = ArticleReviewQueueResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Collegium not found")
    ),
    tag = "article-reviews"
)]
pub async fn list_collegium_review_queue(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<ArticleReviewQueueResponse>, AppError> {
    let (collegium_id, role) =
        crate::modules::collegia::member_role_by_slug(&state.pool, &slug, user.id)
            .await?
            .ok_or_else(|| AppError::NotFound("Collegium not found".into()))?;
    let items = db::list_collegium_queue(
        &state.pool,
        collegium_id,
        user.id,
        role == crate::modules::collegia::CollegiumRole::Steward,
    )
    .await?;
    let total = items.len() as i64;
    Ok(Json(ArticleReviewQueueResponse { items, total }))
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
    require_request_access(&state, &user, &req).await?;

    // The author sees every round; audience reviewers only see rounds
    // addressed to their own audience — and non-admin collegium members
    // additionally only the member-visible ones.
    let is_owner = req.author_user_id == user.id;
    let audience = if is_owner {
        None
    } else {
        Some(req.collegium_id)
    };
    let collegium_role = match req.collegium_id {
        Some(collegium_id) => {
            crate::modules::collegia::member_role(&state.pool, collegium_id, user.id).await?
        }
        None => None,
    };
    let member_visible_only = !is_owner
        && req.collegium_id.is_some()
        && collegium_role != Some(crate::modules::collegia::CollegiumRole::Steward);
    let requests = db::list_requests_for_article(
        &state.pool,
        req.article_id,
        is_owner,
        audience,
        member_visible_only,
    )
    .await?;
    let labels = crate::modules::writing::articles::editorial_labels::list_for_article(
        &state.pool,
        req.article_id,
    )
    .await?;

    Ok(Json(ArticleReviewDetailResponse {
        request: req.to_request_response(),
        assignee: req.assignee(),
        snapshot_html: req.snapshot_html.clone(),
        article: ReviewArticleMeta {
            id: req.article_id.to_string(),
            title: req.article_title.clone(),
            slug: req.article_slug.clone(),
            status: req.article_status,
            author_user_id: req.author_user_id.to_string(),
            author_display_name: req.author_display_name.clone(),
            author_handle: req.author_handle.clone(),
            labels,
        },
        requests,
        draft_changed: req.article_updated_at > req.submitted_at,
        collegium: match (&req.collegium_id, &req.collegium_name, &req.collegium_slug) {
            (Some(id), Some(name), Some(slug)) => Some(ReviewCollegiumMeta {
                id: id.to_string(),
                name: name.clone(),
                slug: slug.clone(),
                member_visible: req.member_visible.unwrap_or(true),
                my_role: collegium_role,
            }),
            _ => None,
        },
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
    require_request_access(&state, &user, &req).await?;

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
    // Reviewers only — the author responds via replies, whatever the
    // audience.
    if req.author_user_id == user.id
        || req.status == ReviewRequestStatus::Withdrawn
        || !is_audience_reviewer(&state, &user, req.collegium_id, req.member_visible).await?
    {
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
    let can_access = ctx.author_user_id == user.id
        || (ctx.request_status != ReviewRequestStatus::Withdrawn
            && is_audience_reviewer(
                &state,
                &user,
                ctx.request_collegium_id,
                ctx.request_member_visible,
            )
            .await?);
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
    let comment_id = parse_uuid(&id, "comment")?;
    let ctx = db::get_comment_context(&state.pool, comment_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Comment not found".into()))?;
    // Audience reviewers resolve threads; the author never does, for
    // either audience. 404 like every other unauthorized review access.
    if ctx.author_user_id == user.id
        || !is_audience_reviewer(
            &state,
            &user,
            ctx.request_collegium_id,
            ctx.request_member_visible,
        )
        .await?
    {
        return Err(AppError::NotFound("Comment not found".into()));
    }

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

/// Access rule for an article's per-audience review channel: the
/// author, or one of the audience's reviewers while at least one
/// non-withdrawn request addressed to that audience exists. Non-admin
/// collegium members additionally need a member-visible request — an
/// stewards-only submission keeps its channel between the author and the
/// collegium's admins.
async fn require_channel_access(
    state: &AppState,
    user: &AuthUser,
    article_id: Uuid,
    collegium_id: Option<Uuid>,
) -> Result<(), AppError> {
    let owner = db::get_article_owner(&state.pool, article_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Article not found".into()))?;
    if owner == user.id {
        return Ok(());
    }
    let allowed = match collegium_id {
        Some(gid) => {
            match crate::modules::collegia::member_role(&state.pool, gid, user.id).await? {
                None => false,
                Some(role) => {
                    db::audience_has_access(
                        &state.pool,
                        article_id,
                        collegium_id,
                        role != crate::modules::collegia::CollegiumRole::Steward,
                    )
                    .await?
                }
            }
        }
        None => {
            is_reviewer(user)
                && db::audience_has_access(&state.pool, article_id, None, false).await?
        }
    };
    if allowed {
        return Ok(());
    }
    Err(AppError::NotFound("Article not found".into()))
}

/// The article's review channel, shared across review rounds.
#[utoipa::path(
    get,
    path = "/api/review/articles/{article_id}/messages",
    params(("article_id" = String, Path, description = "Article ID"), ChannelQuery),
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
    Query(channel): Query<ChannelQuery>,
) -> Result<Json<ArticleReviewMessageListResponse>, AppError> {
    let article_id = parse_uuid(&article_id, "article")?;
    let collegium_id = parse_channel_collegium(&channel)?;
    require_channel_access(&state, &user, article_id, collegium_id).await?;
    let messages = db::list_messages(&state.pool, article_id, collegium_id).await?;
    Ok(Json(ArticleReviewMessageListResponse { messages }))
}

/// Change-detection stamp for the review page's polling loop. The
/// client compares the whole payload and invalidates its request,
/// comment, and message queries when it changes.
#[utoipa::path(
    get,
    path = "/api/review/articles/{article_id}/activity",
    params(("article_id" = String, Path, description = "Article ID"), ChannelQuery),
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
    Query(channel): Query<ChannelQuery>,
) -> Result<Json<ArticleReviewActivityResponse>, AppError> {
    let article_id = parse_uuid(&article_id, "article")?;
    let collegium_id = parse_channel_collegium(&channel)?;
    require_channel_access(&state, &user, article_id, collegium_id).await?;
    let stamp = db::get_activity_stamp(&state.pool, article_id, collegium_id).await?;
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
    params(("article_id" = String, Path, description = "Article ID"), ChannelQuery),
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
    Query(channel): Query<ChannelQuery>,
    Json(body): Json<CreateReviewMessageRequest>,
) -> Result<Json<ArticleReviewMessageResponse>, AppError> {
    let article_id = parse_uuid(&article_id, "article")?;
    let collegium_id = parse_channel_collegium(&channel)?;
    require_channel_access(&state, &user, article_id, collegium_id).await?;
    if !db::any_request_exists(&state.pool, article_id, collegium_id).await? {
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
            collegium_id,
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
fn decision_is_valid(intent: ReviewIntent, decision: ReviewDecision) -> bool {
    matches!(
        (intent, decision),
        (ReviewIntent::Publication, ReviewDecision::Approved)
            | (ReviewIntent::Publication, ReviewDecision::Declined)
            | (ReviewIntent::Feedback, ReviewDecision::Resolved)
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
    let assignee = match params.assignee.as_deref() {
        None | Some("") => db::AssigneeFilter::Any,
        Some("unassigned") => db::AssigneeFilter::Unassigned,
        Some(id) => db::AssigneeFilter::User(parse_uuid(id, "assignee")?),
    };
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(25).clamp(1, 100);

    let (items, total) = db::list_queue(&state.pool, &statuses, &assignee, page, per_page).await?;
    Ok(Json(ArticleReviewQueueResponse { items, total }))
}

/// Users holding a reviewer role, for the assignment dropdown.
#[utoipa::path(
    get,
    path = "/api/admin/article-reviewers",
    responses(
        (status = 200, description = "Assignable reviewers", body = ReviewerListResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "article-reviews"
)]
pub async fn list_article_reviewers(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<ReviewerListResponse>, AppError> {
    user.require_permission(Permission::ArticlesReview)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;
    let reviewers = db::list_reviewers(&state.pool).await?;
    Ok(Json(ReviewerListResponse { reviewers }))
}

/// Assign an editor to a pending request (or unassign with a null
/// assignee). Any reviewer may assign themselves or a colleague.
#[utoipa::path(
    patch,
    path = "/api/admin/article-review-requests/{id}/assignee",
    params(("id" = String, Path, description = "Review request ID")),
    request_body = AssignReviewRequest,
    responses(
        (status = 200, description = "Assignee updated"),
        (status = 400, description = "Assignee is not a reviewer"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Review request not found"),
        (status = 409, description = "Request is no longer pending")
    ),
    tag = "article-reviews"
)]
pub async fn assign_article_review(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<AssignReviewRequest>,
) -> Result<Json<()>, AppError> {
    user.require_permission(Permission::ArticlesReview)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;
    let request_id = parse_uuid(&id, "review request")?;

    let assignee = match &body.assignee_id {
        Some(id) => {
            let assignee_id = parse_uuid(id, "assignee")?;
            if !db::user_is_reviewer(&state.pool, assignee_id).await? {
                return Err(AppError::BadRequest(
                    "Assignee does not hold a reviewer role".into(),
                ));
            }
            Some(assignee_id)
        }
        None => None,
    };

    let assigned = db::assign_request(&state.pool, request_id, assignee).await?;
    if !assigned {
        return Err(AppError::Conflict("Request is no longer pending".into()));
    }
    Ok(Json(()))
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
    // Collegium reviews are outside editorial authority; their requests
    // don't exist as far as the editor surface is concerned.
    if req.collegium_id.is_some() {
        return Err(AppError::NotFound("Review request not found".into()));
    }
    if req.status != ReviewRequestStatus::Pending {
        return Err(AppError::Conflict("Request is no longer pending".into()));
    }

    if !decision_is_valid(req.intent, body.status) {
        return Err(AppError::BadRequest(format!(
            "'{}' is not a valid decision for a {} request",
            body.status.as_str(),
            req.intent.as_str()
        )));
    }

    // Side effects before claiming the request: each is idempotent, so a
    // failure leaves the request pending and the decision retryable.
    let newly_published = body.status == ReviewDecision::Approved
        && req.article_status == crate::modules::writing::articles::models::ArticleStatus::Draft;
    if newly_published {
        crate::modules::writing::articles::db::publish_article_by_id(&state.pool, req.article_id)
            .await?;
    }
    if body.status == ReviewDecision::Approved {
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
        db::ReviewDecisionPatch {
            status: body.status,
            reviewed_by: user.id,
        },
    )
    .await?;
    if !decided {
        return Err(AppError::Conflict("Request is no longer pending".into()));
    }

    if body.status == ReviewDecision::Approved {
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
                collegium_id: None,
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
        use ReviewDecision as D;
        use ReviewIntent as I;
        assert!(decision_is_valid(I::Publication, D::Approved));
        assert!(decision_is_valid(I::Publication, D::Declined));
        assert!(decision_is_valid(I::Feedback, D::Resolved));

        assert!(!decision_is_valid(I::Feedback, D::Approved));
        assert!(!decision_is_valid(I::Feedback, D::Declined));
        assert!(!decision_is_valid(I::Publication, D::Resolved));
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
