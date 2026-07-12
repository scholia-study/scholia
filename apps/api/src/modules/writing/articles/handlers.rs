use axum::Json;
use axum::extract::{Path, Query, State};

use crate::modules::writing::articles::models::{
    ApplyEditorialLabelRequest, ArticleDetailResponse, ArticleListQuery, ArticleListResponse,
    BatchSentencesRequest, BatchSentencesResponse, CreateArticleRequest,
    EditorialLabelListResponse, EditorialLabelResponse, PublicArticleListQuery,
    PublishedArticleListResponse, TopicListResponse, UpdateArticleRequest,
};
use crate::system::auth::middleware::AuthUser;
use crate::system::auth::permissions::Permission;
use crate::system::cache;
use crate::system::error::AppError;
use crate::system::state::AppState;

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
    let (current_active, _) =
        crate::modules::writing::articles::db::get_user_article_counts(&state.pool, user.id)
            .await?;
    let (max_active, _) = crate::modules::writing::articles::db::get_article_limits(&user.roles);
    if current_active >= max_active as i64 {
        return Err(AppError::BadRequest(format!(
            "Article limit reached ({max_active}). Archive an existing article to create a new one."
        )));
    }

    let article =
        crate::modules::writing::articles::db::create_article(&state.pool, user.id, &body.title)
            .await?;
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

    let articles = crate::modules::writing::articles::db::list_user_articles(
        &state.pool,
        user.id,
        params.status.as_deref(),
    )
    .await?;

    let limits = crate::modules::writing::articles::db::get_article_limits_response(
        &state.pool,
        user.id,
        &user.roles,
    )
    .await?;

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

    let article = crate::modules::writing::articles::db::get_user_article_by_slug(
        &state.pool,
        &slug,
        user.id,
    )
    .await?;
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

    let article = crate::modules::writing::articles::db::update_article(
        &state.pool,
        &state.config.frontend_url,
        &slug,
        user.id,
        &user.roles,
        crate::modules::writing::articles::db::ArticleUpdate {
            title: body.title.as_deref(),
            markdown: body.markdown.as_deref(),
            description: body.description.as_deref(),
            topic_ids: body.topic_ids.as_deref(),
        },
    )
    .await?;

    // Drafts have no public presence — nothing cached to purge. For a
    // published article, purge the old slug's paths (the URL caches
    // know) and, after a title-driven slug change, the new one too.
    if article.status == "published" {
        let mut paths = article_cache_paths(&slug);
        if article.slug != slug {
            paths.extend(article_cache_paths(&article.slug));
        }
        cache::invalidate(&state, paths);
    }

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

    crate::modules::writing::articles::db::publish_article(
        &state.pool,
        &slug,
        user.id,
        &user.roles,
    )
    .await?;
    let mut paths = article_cache_paths(&slug);
    paths.extend(sitemap_cache_paths());
    cache::invalidate(&state, paths);
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
    let (_, current_archive) =
        crate::modules::writing::articles::db::get_user_article_counts(&state.pool, user.id)
            .await?;
    let (_, max_archive) = crate::modules::writing::articles::db::get_article_limits(&user.roles);
    if current_archive >= max_archive as i64 {
        return Err(AppError::BadRequest(format!(
            "Archive limit reached ({max_archive}). Upgrade your plan to archive more articles."
        )));
    }

    crate::modules::writing::articles::db::archive_article(&state.pool, &slug, user.id).await?;
    let mut paths = article_cache_paths(&slug);
    paths.extend(sitemap_cache_paths());
    cache::invalidate(&state, paths);
    Ok(Json(()))
}

/// Cache paths to invalidate when a published article appears, changes,
/// or is removed. Listings get short-TTL'd already, but PURGEing them
/// makes the change visible immediately rather than after the TTL.
/// Draft-only changes purge nothing — no public URL serves a draft.
fn article_cache_paths(slug: &str) -> Vec<String> {
    vec![
        format!("/articles/{slug}"),
        "/articles".to_string(),
        format!("/api/articles/{slug}"),
        "/api/articles".to_string(),
    ]
}

/// Sitemap paths change only when the set of published articles (or
/// qualifying author profiles) changes — publish/archive, not content
/// edits, whose lastmod drift can ride out the 1h TTL.
fn sitemap_cache_paths() -> Vec<String> {
    vec!["/sitemap.xml".to_string(), "/sitemaps/site.xml".to_string()]
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

    let (articles, total) = crate::modules::writing::articles::db::list_published_articles(
        &state.pool,
        params.topic_slug.as_deref(),
        params.label_slug.as_deref(),
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
    let article =
        crate::modules::writing::articles::db::get_published_article_by_slug(&state.pool, &slug)
            .await?;
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
    let topics = crate::modules::writing::articles::db::list_topics(&state.pool).await?;
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

    let article =
        crate::modules::writing::articles::db::get_article_by_id(&state.pool, article_id).await?;
    Ok(Json(article))
}

// ── Editorial labels ──────────────────────────────────────

/// List all editorial labels. Public — readers see chip metadata so the
/// frontend can render names/slugs without a separate lookup, and editors
/// use the same endpoint to populate the manage-labels modal.
#[utoipa::path(
    get,
    path = "/api/editorial-labels",
    responses(
        (status = 200, description = "Editorial labels", body = EditorialLabelListResponse)
    ),
    tag = "articles"
)]
pub async fn list_editorial_labels(
    State(state): State<AppState>,
) -> Result<Json<EditorialLabelListResponse>, AppError> {
    let labels =
        crate::modules::writing::articles::editorial_labels::list_labels(&state.pool).await?;
    Ok(Json(EditorialLabelListResponse { labels }))
}

/// Apply an editorial label to a published article. Editor/admin only.
#[utoipa::path(
    post,
    path = "/api/admin/articles/{slug}/labels",
    params(("slug" = String, Path, description = "Article slug")),
    request_body = ApplyEditorialLabelRequest,
    responses(
        (status = 200, description = "Label applied", body = EditorialLabelResponse),
        (status = 400, description = "Article not eligible (e.g. not published)"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Article or label not found")
    ),
    tag = "articles"
)]
pub async fn apply_article_label(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<ApplyEditorialLabelRequest>,
) -> Result<Json<EditorialLabelResponse>, AppError> {
    user.require_permission(Permission::ArticleLabelsManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;
    let label = crate::modules::writing::articles::editorial_labels::apply_label(
        &state.pool,
        &slug,
        &body.label_slug,
        user.id,
    )
    .await?;
    Ok(Json(label))
}

/// Remove an editorial label from an article. Editor/admin only.
/// Idempotent — returns 200 even if the label wasn't applied.
#[utoipa::path(
    delete,
    path = "/api/admin/articles/{slug}/labels/{label_slug}",
    params(
        ("slug" = String, Path, description = "Article slug"),
        ("label_slug" = String, Path, description = "Label slug")
    ),
    responses(
        (status = 200, description = "Label removed (or wasn't applied)"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "articles"
)]
pub async fn remove_article_label(
    State(state): State<AppState>,
    user: AuthUser,
    Path((slug, label_slug)): Path<(String, String)>,
) -> Result<Json<()>, AppError> {
    user.require_permission(Permission::ArticleLabelsManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;
    crate::modules::writing::articles::editorial_labels::remove_label(
        &state.pool,
        &slug,
        &label_slug,
    )
    .await?;
    Ok(Json(()))
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
        let item = crate::modules::writing::articles::db::batch_get_sentences(
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
