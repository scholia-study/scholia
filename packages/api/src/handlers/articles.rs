use axum::Json;
use axum::extract::{Path, Query, State};

use crate::auth::middleware::AuthUser;
use crate::auth::permissions::Permission;
use crate::db;
use crate::error::AppError;
use crate::models::article::{
    ArticleDetailResponse, ArticleListQuery, ArticleListResponse, BatchSentencesRequest,
    BatchSentencesResponse, CreateArticleRequest, PublicArticleListQuery,
    PublishedArticleListResponse, TopicListResponse, UpdateArticleRequest,
};
use crate::state::AppState;

// ── User article endpoints (authenticated) ────────────────

/// Create a new article
#[utoipa::path(
    post,
    path = "/api/articles",
    request_body = CreateArticleRequest,
    responses(
        (status = 200, description = "Article created", body = ArticleDetailResponse),
        (status = 400, description = "Invalid input or limit reached"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "articles"
)]
pub async fn create_article(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateArticleRequest>,
) -> Result<Json<ArticleDetailResponse>, AppError> {
    user.require_permission(Permission::ArticlesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    // Check active article limit
    let (current_active, _) = db::articles::get_user_article_counts(&state.pool, user.id).await?;
    let (max_active, _) = db::articles::get_article_limits(&user.roles);
    if current_active >= max_active as i64 {
        return Err(AppError::BadRequest(format!(
            "Article limit reached ({max_active}). Archive an existing article to create a new one."
        )));
    }

    let article = db::articles::create_article(&state.pool, user.id, &body.title).await?;
    Ok(Json(article))
}

/// List current user's articles
#[utoipa::path(
    get,
    path = "/api/user/articles",
    params(ArticleListQuery),
    responses(
        (status = 200, description = "User's articles", body = ArticleListResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "articles"
)]
pub async fn list_user_articles(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<ArticleListQuery>,
) -> Result<Json<ArticleListResponse>, AppError> {
    user.require_permission(Permission::ArticlesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let articles =
        db::articles::list_user_articles(&state.pool, user.id, params.status.as_deref()).await?;

    let limits =
        db::articles::get_article_limits_response(&state.pool, user.id, &user.roles).await?;

    Ok(Json(ArticleListResponse { articles, limits }))
}

/// Get a specific article for editing (owner only)
#[utoipa::path(
    get,
    path = "/api/user/articles/{slug}",
    params(("slug" = String, Path, description = "Article slug")),
    responses(
        (status = 200, description = "Article detail", body = ArticleDetailResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Article not found")
    ),
    tag = "articles"
)]
pub async fn get_user_article(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<ArticleDetailResponse>, AppError> {
    user.require_permission(Permission::ArticlesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let article = db::articles::get_user_article_by_slug(&state.pool, &slug, user.id).await?;
    Ok(Json(article))
}

/// Update an article (owner only)
#[utoipa::path(
    put,
    path = "/api/user/articles/{slug}",
    params(("slug" = String, Path, description = "Article slug")),
    request_body = UpdateArticleRequest,
    responses(
        (status = 200, description = "Article updated", body = ArticleDetailResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Article not found")
    ),
    tag = "articles"
)]
pub async fn update_article(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<UpdateArticleRequest>,
) -> Result<Json<ArticleDetailResponse>, AppError> {
    user.require_permission(Permission::ArticlesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let article = db::articles::update_article(
        &state.pool,
        &slug,
        user.id,
        body.title.as_deref(),
        body.markdown.as_deref(),
        body.description.as_deref(),
        body.topic_ids.as_deref(),
    )
    .await?;

    Ok(Json(article))
}

/// Publish an article
#[utoipa::path(
    post,
    path = "/api/user/articles/{slug}/publish",
    params(("slug" = String, Path, description = "Article slug")),
    responses(
        (status = 200, description = "Article published"),
        (status = 400, description = "Publish limit reached"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Article not found")
    ),
    tag = "articles"
)]
pub async fn publish_article(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<()>, AppError> {
    user.require_permission(Permission::ArticlesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    db::articles::publish_article(&state.pool, &slug, user.id).await?;
    Ok(Json(()))
}

/// Archive an article (one-way from published)
#[utoipa::path(
    post,
    path = "/api/user/articles/{slug}/archive",
    params(("slug" = String, Path, description = "Article slug")),
    responses(
        (status = 200, description = "Article archived"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Article not found")
    ),
    tag = "articles"
)]
pub async fn archive_article(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<()>, AppError> {
    user.require_permission(Permission::ArticlesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    // Check archive limit
    let (_, current_archive) = db::articles::get_user_article_counts(&state.pool, user.id).await?;
    let (_, max_archive) = db::articles::get_article_limits(&user.roles);
    if current_archive >= max_archive as i64 {
        return Err(AppError::BadRequest(format!(
            "Archive limit reached ({max_archive}). Upgrade your plan to archive more articles."
        )));
    }

    db::articles::archive_article(&state.pool, &slug, user.id).await?;
    Ok(Json(()))
}

// ── Public article endpoints ──────────────────────────────

/// List published articles
#[utoipa::path(
    get,
    path = "/api/articles",
    params(PublicArticleListQuery),
    responses(
        (status = 200, description = "Published articles", body = PublishedArticleListResponse)
    ),
    tag = "articles"
)]
pub async fn list_published_articles(
    State(state): State<AppState>,
    Query(params): Query<PublicArticleListQuery>,
) -> Result<Json<PublishedArticleListResponse>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let (articles, total) = db::articles::list_published_articles(
        &state.pool,
        params.topic_slug.as_deref(),
        page,
        per_page,
    )
    .await?;

    Ok(Json(PublishedArticleListResponse { articles, total }))
}

/// Get a published article by slug
#[utoipa::path(
    get,
    path = "/api/articles/{slug}",
    params(("slug" = String, Path, description = "Article slug")),
    responses(
        (status = 200, description = "Published article", body = ArticleDetailResponse),
        (status = 404, description = "Article not found")
    ),
    tag = "articles"
)]
pub async fn get_published_article(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ArticleDetailResponse>, AppError> {
    let article = db::articles::get_published_article_by_slug(&state.pool, &slug).await?;
    Ok(Json(article))
}

// ── Topic endpoints ───────────────────────────────────────

/// List all topics
#[utoipa::path(
    get,
    path = "/api/topics",
    responses(
        (status = 200, description = "All topics", body = TopicListResponse)
    ),
    tag = "topics"
)]
pub async fn list_topics(
    State(state): State<AppState>,
) -> Result<Json<TopicListResponse>, AppError> {
    let topics = db::articles::list_topics(&state.pool).await?;
    Ok(Json(TopicListResponse { topics }))
}

/// Get a published/archived article by UUID (stable URL)
#[utoipa::path(
    get,
    path = "/api/articles/by-id/{id}",
    params(("id" = String, Path, description = "Article UUID")),
    responses(
        (status = 200, description = "Article detail", body = ArticleDetailResponse),
        (status = 404, description = "Article not found")
    ),
    tag = "articles"
)]
pub async fn get_article_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ArticleDetailResponse>, AppError> {
    let article_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid article ID".into()))?;

    let article = db::articles::get_article_by_id(&state.pool, article_id).await?;
    Ok(Json(article))
}

// ── Batch sentence endpoint ───────────────────────────────

/// Batch fetch sentences for quotation card hydration
#[utoipa::path(
    post,
    path = "/api/sentences/batch",
    request_body = BatchSentencesRequest,
    responses(
        (status = 200, description = "Sentence data", body = BatchSentencesResponse)
    ),
    tag = "sentences"
)]
pub async fn batch_sentences(
    State(state): State<AppState>,
    Json(body): Json<BatchSentencesRequest>,
) -> Result<Json<BatchSentencesResponse>, AppError> {
    let mut items = Vec::with_capacity(body.items.len());

    for req in &body.items {
        let item = db::articles::batch_get_sentences(
            &state.pool,
            &req.book_slug,
            &req.node_slug,
            req.start_number,
            req.end_number,
            &req.kind,
        )
        .await?;
        items.push(item);
    }

    Ok(Json(BatchSentencesResponse { items }))
}
