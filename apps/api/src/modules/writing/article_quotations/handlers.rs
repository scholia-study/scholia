use axum::Json;
use axum::extract::{Path, State};

use crate::modules::writing::article_quotations::models::{
    ArticleQuotationListResponse, ArticleQuotationResponse, CreateArticleQuotationRequest,
    CreateArticleQuotationResponse,
};
use crate::system::auth::middleware::AuthUser;
use crate::system::auth::permissions::Permission;
use crate::system::error::AppError;
use crate::system::state::AppState;
use crate::system::validation::{
    MAX_ARTICLE_QUOTATION_HTML, MAX_ARTICLE_QUOTATION_TEXT, check_max_len,
};

/// Save a quotation from an article (returns existing if duplicate)
#[utoipa::path(
    post,
    path = "/api/article-quotations",
    request_body = CreateArticleQuotationRequest,
    responses(
        (status = 200, description = "Article quotation saved", body = CreateArticleQuotationResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated")
    ),
    tag = "article-quotations"
)]
pub async fn create_article_quotation(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateArticleQuotationRequest>,
) -> Result<Json<CreateArticleQuotationResponse>, AppError> {
    user.require_permission(Permission::NotesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let current =
        crate::modules::writing::quotations::db::get_user_quotation_count(&state.pool, user.id)
            .await?;
    let max = crate::modules::writing::quotations::db::get_quotation_limit(&user.roles);
    if current >= max as i64 {
        return Err(AppError::BadRequest(format!(
            "Quotation limit reached ({max}). Upgrade your plan to save more quotations."
        )));
    }

    let article_id = uuid::Uuid::parse_str(&body.article_id)
        .map_err(|_| AppError::BadRequest("Invalid article_id".into()))?;

    check_max_len("Quotation text", &body.text, MAX_ARTICLE_QUOTATION_TEXT)?;
    check_max_len("Quotation html", &body.html, MAX_ARTICLE_QUOTATION_HTML)?;

    let (article_quotation, created) =
        crate::modules::writing::article_quotations::db::create_article_quotation(
            &state.pool,
            user.id,
            article_id,
            &body.text,
            &body.html,
        )
        .await?;

    Ok(Json(CreateArticleQuotationResponse {
        article_quotation,
        created,
    }))
}

/// List current user's article quotations
#[utoipa::path(
    get,
    path = "/api/article-quotations",
    responses(
        (status = 200, description = "User's article quotations", body = ArticleQuotationListResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "article-quotations"
)]
pub async fn list_article_quotations(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<ArticleQuotationListResponse>, AppError> {
    user.require_permission(Permission::NotesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let article_quotations =
        crate::modules::writing::article_quotations::db::list_article_quotations(
            &state.pool,
            user.id,
        )
        .await?;

    Ok(Json(ArticleQuotationListResponse { article_quotations }))
}

/// Get a single article quotation (for directive hydration)
#[utoipa::path(
    get,
    path = "/api/article-quotations/{id}",
    params(("id" = String, Path, description = "Article quotation ID")),
    responses(
        (status = 200, description = "Article quotation detail", body = ArticleQuotationResponse),
        (status = 404, description = "Not found")
    ),
    tag = "article-quotations"
)]
pub async fn get_article_quotation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ArticleQuotationResponse>, AppError> {
    let quotation_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid quotation ID".into()))?;

    let article_quotation = crate::modules::writing::article_quotations::db::get_article_quotation(
        &state.pool,
        quotation_id,
    )
    .await?;

    Ok(Json(article_quotation))
}

/// Delete an article quotation
#[utoipa::path(
    delete,
    path = "/api/article-quotations/{id}",
    params(("id" = String, Path, description = "Article quotation ID")),
    responses(
        (status = 200, description = "Article quotation deleted"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Not found")
    ),
    tag = "article-quotations"
)]
pub async fn delete_article_quotation(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<()>, AppError> {
    user.require_permission(Permission::NotesDelete)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let quotation_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid quotation ID".into()))?;

    crate::modules::writing::article_quotations::db::delete_article_quotation(
        &state.pool,
        quotation_id,
        user.id,
    )
    .await?;

    Ok(Json(()))
}
