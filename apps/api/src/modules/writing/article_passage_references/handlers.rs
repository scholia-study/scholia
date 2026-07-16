use axum::Json;
use axum::extract::{Path, Query, State};

use crate::modules::writing::article_passage_references::db;
use crate::modules::writing::article_passage_references::models::{
    ArticleReferenceQuery, PassageArticleListResponse,
};
use crate::system::error::AppError;
use crate::system::state::AppState;

const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 50;

/// List published articles quoting a sentence range (public)
#[utoipa::path(
    get,
    path = "/api/books/{slug}/article-references",
    params(
        ("slug" = String, Path, description = "Book slug"),
        ArticleReferenceQuery,
    ),
    responses(
        (status = 200, description = "Articles quoting the passage, across translations of the same work", body = PassageArticleListResponse),
        (status = 404, description = "Book not found")
    ),
    tag = "articles"
)]
pub async fn list_article_references(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<ArticleReferenceQuery>,
) -> Result<Json<PassageArticleListResponse>, AppError> {
    let book_id = crate::modules::corpus::get_book_id_by_slug(&state.pool, &slug).await?;

    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);

    let (articles, total) = db::list_article_references(
        &state.pool,
        book_id,
        params.start,
        params.end,
        &params.kind,
        limit,
        offset,
    )
    .await?;

    Ok(Json(PassageArticleListResponse { articles, total }))
}
