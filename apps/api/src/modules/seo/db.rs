use sqlx::PgPool;
use time::OffsetDateTime;

use crate::system::error::AppError;

pub struct SitemapEntry {
    pub slug: String,
    pub lastmod: OffsetDateTime,
}

/// One entry per book: slug + the latest change across the book row and
/// its TOC nodes. Drives the sitemap index.
pub async fn book_entries(pool: &PgPool) -> Result<Vec<SitemapEntry>, AppError> {
    let rows = sqlx::query_as!(
        SitemapEntry,
        r#"SELECT
               b.slug,
               GREATEST(b.updated_at, MAX(tn.updated_at)) AS "lastmod!"
           FROM books b
           LEFT JOIN toc_nodes tn ON tn.book_id = b.id
           GROUP BY b.id, b.slug, b.updated_at
           ORDER BY b.slug"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn book_lastmod(
    pool: &PgPool,
    book_slug: &str,
) -> Result<Option<OffsetDateTime>, AppError> {
    let row = sqlx::query_scalar!(
        r#"SELECT
               GREATEST(b.updated_at, MAX(tn.updated_at)) AS "lastmod!"
           FROM books b
           LEFT JOIN toc_nodes tn ON tn.book_id = b.id
           WHERE b.slug = $1
           GROUP BY b.id, b.updated_at"#,
        book_slug,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Content-bearing TOC nodes of a book, in reading order. Mirrors the
/// `has_content` predicate of the TOC endpoint: a node is a page only
/// if it has content blocks.
pub async fn content_node_entries(
    pool: &PgPool,
    book_slug: &str,
) -> Result<Vec<SitemapEntry>, AppError> {
    let rows = sqlx::query_as!(
        SitemapEntry,
        r#"SELECT tn.slug, tn.updated_at AS "lastmod"
           FROM toc_nodes tn
           JOIN books b ON b.id = tn.book_id
           WHERE b.slug = $1
             AND EXISTS (SELECT 1 FROM content_blocks cb WHERE cb.node_id = tn.id)
           ORDER BY tn.sort_order"#,
        book_slug,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn published_article_entries(pool: &PgPool) -> Result<Vec<SitemapEntry>, AppError> {
    let rows = sqlx::query_as!(
        SitemapEntry,
        r#"SELECT slug, GREATEST(updated_at, published_at) AS "lastmod!"
           FROM articles
           WHERE status = 'published'
           ORDER BY published_at DESC"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Profiles worth indexing: users with a handle and at least one
/// published article (the profile route noindexes everyone else).
pub async fn author_entries(pool: &PgPool) -> Result<Vec<SitemapEntry>, AppError> {
    let rows = sqlx::query_as!(
        SitemapEntry,
        r#"SELECT u.handle AS "slug!", u.updated_at AS "lastmod"
           FROM users u
           WHERE u.handle IS NOT NULL
             AND EXISTS (
                 SELECT 1 FROM articles a
                 WHERE a.user_id = u.id AND a.status = 'published'
             )
           ORDER BY u.handle"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
