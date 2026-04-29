use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::article_quotation::ArticleQuotationResponse;

// ── Row types ──────────────────────────────────────────────

struct ArticleQuotationRow {
    id: Uuid,
    article_id: Option<Uuid>,
    article_title: String,
    author_display_name: String,
    text: String,
    html: String,
    note_count: Option<i64>,
    created_at: time::OffsetDateTime,
}

// ── Helpers ────────────────────────────────────────────────

fn fmt_time(t: time::OffsetDateTime) -> String {
    t.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

fn article_quotation_from_row(r: ArticleQuotationRow) -> ArticleQuotationResponse {
    ArticleQuotationResponse {
        id: r.id.to_string(),
        article_id: r.article_id.map(|id| id.to_string()),
        article_title: r.article_title,
        author_display_name: r.author_display_name,
        text: r.text,
        html: r.html,
        note_count: r.note_count.unwrap_or(0),
        created_at: fmt_time(r.created_at),
    }
}

// ── Queries ────────────────────────────────────────────────

pub async fn create_article_quotation(
    pool: &PgPool,
    user_id: Uuid,
    article_id: Uuid,
    text: &str,
    html: &str,
) -> Result<(ArticleQuotationResponse, bool), AppError> {
    // App-level dedup: check if same user already saved same text from same article
    let existing = sqlx::query_scalar!(
        r#"SELECT id FROM article_quotations
           WHERE user_id = $1 AND article_id = $2 AND text = $3"#,
        user_id,
        article_id,
        text,
    )
    .fetch_optional(pool)
    .await?;

    if let Some(existing_id) = existing {
        let row = fetch_article_quotation_row(pool, existing_id).await?;
        return Ok((article_quotation_from_row(row), false));
    }

    // Fetch article metadata for snapshot
    struct ArticleMeta {
        title: String,
        author_display_name: String,
    }
    let meta = sqlx::query_as!(
        ArticleMeta,
        r#"SELECT a.title, u.display_name AS "author_display_name!"
           FROM articles a
           JOIN users u ON u.id = a.user_id
           WHERE a.id = $1 AND a.status IN ('published', 'archived')"#,
        article_id,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Article not found or not published".into()))?;

    let new_id = sqlx::query_scalar!(
        r#"INSERT INTO article_quotations (user_id, article_id, article_title, author_display_name, text, html)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id"#,
        user_id,
        article_id,
        meta.title,
        meta.author_display_name,
        text,
        html,
    )
    .fetch_one(pool)
    .await?;

    let row = fetch_article_quotation_row(pool, new_id).await?;
    Ok((article_quotation_from_row(row), true))
}

pub async fn list_article_quotations(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ArticleQuotationResponse>, AppError> {
    let rows = sqlx::query_as!(
        ArticleQuotationRow,
        r#"SELECT aq.id, aq.article_id, aq.article_title,
                  aq.author_display_name, aq.text, aq.html,
                  COUNT(qn.id) AS "note_count?",
                  aq.created_at
           FROM article_quotations aq
           LEFT JOIN quotation_notes qn ON qn.article_quotation_id = aq.id
           WHERE aq.user_id = $1
           GROUP BY aq.id
           ORDER BY aq.created_at DESC"#,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(article_quotation_from_row).collect())
}

pub async fn get_article_quotation(
    pool: &PgPool,
    id: Uuid,
) -> Result<ArticleQuotationResponse, AppError> {
    let row = fetch_article_quotation_row(pool, id).await?;
    Ok(article_quotation_from_row(row))
}

pub async fn get_article_quotation_owner(pool: &PgPool, id: Uuid) -> Result<Uuid, AppError> {
    sqlx::query_scalar!(
        r#"SELECT user_id FROM article_quotations WHERE id = $1"#,
        id,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Article quotation not found".into()))
}

pub async fn delete_article_quotation(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"DELETE FROM article_quotations WHERE id = $1 AND user_id = $2"#,
        id,
        user_id,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Article quotation not found".into()));
    }
    Ok(())
}

// ── Unified listing helpers ────────────────────────────────

pub struct UnifiedArticleQuotationRow {
    pub id: Uuid,
    pub article_id: Option<Uuid>,
    pub article_title: String,
    pub author_display_name: String,
    pub text: String,
    pub note_count: Option<i64>,
    pub created_at: time::OffsetDateTime,
}

pub async fn list_article_quotations_for_unified(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<UnifiedArticleQuotationRow>, AppError> {
    let rows = sqlx::query_as!(
        UnifiedArticleQuotationRow,
        r#"SELECT aq.id, aq.article_id, aq.article_title,
                  aq.author_display_name, aq.text,
                  COUNT(qn.id) AS "note_count?",
                  aq.created_at
           FROM article_quotations aq
           LEFT JOIN quotation_notes qn ON qn.article_quotation_id = aq.id
           WHERE aq.user_id = $1
           GROUP BY aq.id
           ORDER BY aq.created_at DESC"#,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

// ── Internal helpers ───────────────────────────────────────

async fn fetch_article_quotation_row(
    pool: &PgPool,
    id: Uuid,
) -> Result<ArticleQuotationRow, AppError> {
    sqlx::query_as!(
        ArticleQuotationRow,
        r#"SELECT aq.id, aq.article_id, aq.article_title,
                  aq.author_display_name, aq.text, aq.html,
                  COUNT(qn.id) AS "note_count?",
                  aq.created_at
           FROM article_quotations aq
           LEFT JOIN quotation_notes qn ON qn.article_quotation_id = aq.id
           WHERE aq.id = $1
           GROUP BY aq.id"#,
        id,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Article quotation not found".into()))
}
