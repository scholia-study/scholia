use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::writing::articles::db::fmt_time;
use crate::modules::writing::series::models::{
    ArticleSeriesContext, SeriesAdminResponse, SeriesDetailResponse, SeriesMemberResponse,
    SeriesNeighbor, SeriesResponse,
};
use crate::system::error::{AppError, SqlxResultExt};

pub struct SeriesCreate<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
}

pub struct SeriesUpdate<'a> {
    pub name: Option<&'a str>,
    /// `Some("")` clears the description.
    pub description: Option<&'a str>,
    pub pinned: Option<bool>,
    pub sort_order: Option<i32>,
}

/// Reorder input must be an exact permutation of the current members: a
/// stale drawer (member added/removed elsewhere) must fail loudly, not
/// silently drop or resurrect memberships.
fn validate_reorder(current: &[Uuid], requested: &[Uuid]) -> Result<(), AppError> {
    use std::collections::HashSet;
    let requested_set: HashSet<&Uuid> = requested.iter().collect();
    if requested_set.len() != requested.len() {
        return Err(AppError::BadRequest(
            "Reorder list contains a duplicate article".into(),
        ));
    }
    let current_set: HashSet<&Uuid> = current.iter().collect();
    if requested_set != current_set {
        return Err(AppError::BadRequest(
            "Reorder list must contain exactly the series' current articles".into(),
        ));
    }
    Ok(())
}

/// All series, pinned first (each block in editor-set order) — the
/// order both the series index page and the landing sidebar show.
pub async fn list_series(
    pool: &PgPool,
    pinned: Option<bool>,
) -> Result<Vec<SeriesResponse>, AppError> {
    struct Row {
        id: Uuid,
        name: String,
        slug: String,
        description: Option<String>,
        pinned: bool,
        article_count: i64,
    }
    let rows = sqlx::query_as!(
        Row,
        r#"SELECT s.id, s.name, s.slug, s.description, s.pinned,
                  COUNT(a.id) FILTER (WHERE a.status = 'published') AS "article_count!"
           FROM series s
           LEFT JOIN series_articles sa ON sa.series_id = s.id
           LEFT JOIN articles a ON a.id = sa.article_id
           WHERE $1::BOOLEAN IS NULL OR s.pinned = $1
           GROUP BY s.id
           ORDER BY s.pinned DESC, s.sort_order, s.name"#,
        pinned,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SeriesResponse {
            id: r.id.to_string(),
            name: r.name,
            slug: r.slug,
            description: r.description,
            pinned: r.pinned,
            article_count: r.article_count,
        })
        .collect())
}

pub async fn get_series_detail_by_slug(
    pool: &PgPool,
    slug: &str,
    page: i32,
    per_page: i32,
) -> Result<SeriesDetailResponse, AppError> {
    let row = sqlx::query!(
        r#"SELECT id, name, slug, description FROM series WHERE slug = $1"#,
        slug,
    )
    .fetch_one(pool)
    .await
    .on_missing(|| AppError::NotFound("Series not found".into()))?;

    let (articles, total) =
        crate::modules::writing::articles::db::list_published_articles_in_series(
            pool, row.id, page, per_page,
        )
        .await?;

    Ok(SeriesDetailResponse {
        id: row.id.to_string(),
        name: row.name,
        slug: row.slug,
        description: row.description,
        articles,
        total,
    })
}

async fn admin_get_series(pool: &PgPool, series_id: Uuid) -> Result<SeriesAdminResponse, AppError> {
    let r = sqlx::query!(
        r#"SELECT s.id, s.name, s.slug, s.description, s.pinned, s.sort_order,
                  s.created_at, s.updated_at,
                  COUNT(a.id) AS "article_count!",
                  COUNT(a.id) FILTER (WHERE a.status = 'published') AS "published_count!"
           FROM series s
           LEFT JOIN series_articles sa ON sa.series_id = s.id
           LEFT JOIN articles a ON a.id = sa.article_id
           WHERE s.id = $1
           GROUP BY s.id"#,
        series_id,
    )
    .fetch_one(pool)
    .await
    .on_missing(|| AppError::NotFound("Series not found".into()))?;

    Ok(SeriesAdminResponse {
        id: r.id.to_string(),
        name: r.name,
        slug: r.slug,
        description: r.description,
        pinned: r.pinned,
        sort_order: r.sort_order,
        article_count: r.article_count,
        published_count: r.published_count,
        created_at: fmt_time(r.created_at),
        updated_at: fmt_time(r.updated_at),
    })
}

pub async fn admin_list_series(pool: &PgPool) -> Result<Vec<SeriesAdminResponse>, AppError> {
    struct Row {
        id: Uuid,
        name: String,
        slug: String,
        description: Option<String>,
        pinned: bool,
        sort_order: i32,
        created_at: time::OffsetDateTime,
        updated_at: time::OffsetDateTime,
        article_count: i64,
        published_count: i64,
    }
    let rows = sqlx::query_as!(
        Row,
        r#"SELECT s.id, s.name, s.slug, s.description, s.pinned, s.sort_order,
                  s.created_at, s.updated_at,
                  COUNT(a.id) AS "article_count!",
                  COUNT(a.id) FILTER (WHERE a.status = 'published') AS "published_count!"
           FROM series s
           LEFT JOIN series_articles sa ON sa.series_id = s.id
           LEFT JOIN articles a ON a.id = sa.article_id
           GROUP BY s.id
           ORDER BY s.sort_order, s.name"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SeriesAdminResponse {
            id: r.id.to_string(),
            name: r.name,
            slug: r.slug,
            description: r.description,
            pinned: r.pinned,
            sort_order: r.sort_order,
            article_count: r.article_count,
            published_count: r.published_count,
            created_at: fmt_time(r.created_at),
            updated_at: fmt_time(r.updated_at),
        })
        .collect())
}

pub async fn create_series(
    pool: &PgPool,
    entry: SeriesCreate<'_>,
) -> Result<SeriesAdminResponse, AppError> {
    let slug = slug::slugify(entry.name);
    if slug.is_empty() {
        return Err(AppError::BadRequest(
            "Series name must contain letters or numbers".into(),
        ));
    }

    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM series WHERE name = $1 OR slug = $2) AS "exists!""#,
        entry.name,
        slug,
    )
    .fetch_one(pool)
    .await?;
    if exists {
        return Err(AppError::BadRequest(
            "A series with this name already exists".into(),
        ));
    }

    let id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO series (name, slug, description) VALUES ($1, $2, $3) RETURNING id"#,
        entry.name,
        slug,
        entry.description,
    )
    .fetch_one(pool)
    .await?;

    admin_get_series(pool, id).await
}

/// Renames/edits series metadata; the slug is immutable (it lives in
/// shareable URLs and the hardcoded blog link).
pub async fn update_series(
    pool: &PgPool,
    series_id: Uuid,
    patch: SeriesUpdate<'_>,
) -> Result<SeriesAdminResponse, AppError> {
    if let Some(name) = patch.name {
        let taken = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM series WHERE name = $1 AND id <> $2) AS "taken!""#,
            name,
            series_id,
        )
        .fetch_one(pool)
        .await?;
        if taken {
            return Err(AppError::BadRequest(
                "A series with this name already exists".into(),
            ));
        }
    }

    let updated = sqlx::query!(
        r#"UPDATE series SET
               name = COALESCE($2, name),
               description = CASE
                   WHEN $3::TEXT IS NULL THEN description
                   WHEN $3 = '' THEN NULL
                   ELSE $3
               END,
               pinned = COALESCE($4, pinned),
               sort_order = COALESCE($5, sort_order),
               updated_at = now()
           WHERE id = $1"#,
        series_id,
        patch.name,
        patch.description,
        patch.pinned,
        patch.sort_order,
    )
    .execute(pool)
    .await?
    .rows_affected();
    if updated == 0 {
        return Err(AppError::NotFound("Series not found".into()));
    }

    admin_get_series(pool, series_id).await
}

/// Deletes a series; refused while any article is still attached.
/// Returns the slug for cache purging.
pub async fn delete_series(pool: &PgPool, series_id: Uuid) -> Result<String, AppError> {
    let slug = sqlx::query_scalar!(r#"SELECT slug FROM series WHERE id = $1"#, series_id)
        .fetch_one(pool)
        .await
        .on_missing(|| AppError::NotFound("Series not found".into()))?;

    let in_use = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM series_articles WHERE series_id = $1"#,
        series_id,
    )
    .fetch_one(pool)
    .await?;
    if in_use > 0 {
        return Err(AppError::BadRequest(format!(
            "Series contains {in_use} article(s); detach them first"
        )));
    }

    sqlx::query!(r#"DELETE FROM series WHERE id = $1"#, series_id)
        .execute(pool)
        .await?;
    Ok(slug)
}

pub async fn list_series_members(
    pool: &PgPool,
    series_id: Uuid,
) -> Result<Vec<SeriesMemberResponse>, AppError> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM series WHERE id = $1) AS "exists!""#,
        series_id,
    )
    .fetch_one(pool)
    .await?;
    if !exists {
        return Err(AppError::NotFound("Series not found".into()));
    }

    struct Row {
        article_id: Uuid,
        title: String,
        slug: String,
        status: String,
        author_display_name: String,
        position: i32,
    }
    let rows = sqlx::query_as!(
        Row,
        r#"SELECT sa.article_id, a.title, a.slug,
                  a.status::TEXT AS "status!",
                  u.display_name AS "author_display_name!",
                  sa.position
           FROM series_articles sa
           JOIN articles a ON a.id = sa.article_id
           JOIN users u ON u.id = a.user_id
           WHERE sa.series_id = $1
           ORDER BY sa.position"#,
        series_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SeriesMemberResponse {
            article_id: r.article_id.to_string(),
            title: r.title,
            slug: r.slug,
            status: r.status,
            author_display_name: r.author_display_name,
            position: r.position,
        })
        .collect())
}

/// Attaches a published article at position 1 (prepend), shifting the
/// rest down. Returns `(series_slug, article_slug)` for cache purging.
pub async fn add_series_article(
    pool: &PgPool,
    series_id: Uuid,
    article_id: Uuid,
) -> Result<(String, String), AppError> {
    let series_slug = sqlx::query_scalar!(r#"SELECT slug FROM series WHERE id = $1"#, series_id)
        .fetch_one(pool)
        .await
        .on_missing(|| AppError::NotFound("Series not found".into()))?;

    let article = sqlx::query!(
        r#"SELECT slug, status::TEXT AS "status!" FROM articles WHERE id = $1"#,
        article_id,
    )
    .fetch_one(pool)
    .await
    .on_missing(|| AppError::NotFound("Article not found".into()))?;
    if article.status != "published" {
        return Err(AppError::BadRequest(
            "Only published articles can be added to a series".into(),
        ));
    }

    let already = sqlx::query_scalar!(
        r#"SELECT EXISTS(
               SELECT 1 FROM series_articles WHERE series_id = $1 AND article_id = $2
           ) AS "exists!""#,
        series_id,
        article_id,
    )
    .fetch_one(pool)
    .await?;
    if already {
        return Err(AppError::BadRequest(
            "Article is already in this series".into(),
        ));
    }

    let mut tx = pool.begin().await?;
    sqlx::query!(
        r#"UPDATE series_articles SET position = position + 1 WHERE series_id = $1"#,
        series_id,
    )
    .execute(&mut *tx)
    .await?;
    // The PK backstops the pre-check against concurrent attaches (23505 → 409).
    sqlx::query!(
        r#"INSERT INTO series_articles (series_id, article_id, position) VALUES ($1, $2, 1)"#,
        series_id,
        article_id,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        r#"UPDATE series SET updated_at = now() WHERE id = $1"#,
        series_id,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok((series_slug, article.slug))
}

/// Detaches an article and renumbers the remaining members densely.
/// Returns `(series_slug, article_slug)` for cache purging.
pub async fn remove_series_article(
    pool: &PgPool,
    series_id: Uuid,
    article_id: Uuid,
) -> Result<(String, String), AppError> {
    let series_slug = sqlx::query_scalar!(r#"SELECT slug FROM series WHERE id = $1"#, series_id)
        .fetch_one(pool)
        .await
        .on_missing(|| AppError::NotFound("Series not found".into()))?;
    let article_slug =
        sqlx::query_scalar!(r#"SELECT slug FROM articles WHERE id = $1"#, article_id)
            .fetch_one(pool)
            .await
            .on_missing(|| AppError::NotFound("Article not found".into()))?;

    let mut tx = pool.begin().await?;
    let deleted = sqlx::query!(
        r#"DELETE FROM series_articles WHERE series_id = $1 AND article_id = $2"#,
        series_id,
        article_id,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if deleted == 0 {
        return Err(AppError::NotFound("Article is not in this series".into()));
    }
    sqlx::query!(
        r#"WITH ranked AS (
               SELECT article_id, ROW_NUMBER() OVER (ORDER BY position) AS rn
               FROM series_articles WHERE series_id = $1
           )
           UPDATE series_articles sa
           SET position = ranked.rn::INT
           FROM ranked
           WHERE sa.series_id = $1 AND sa.article_id = ranked.article_id"#,
        series_id,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        r#"UPDATE series SET updated_at = now() WHERE id = $1"#,
        series_id,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok((series_slug, article_slug))
}

/// Rewrites all member positions to the requested order (1..n). Returns
/// the series slug for cache purging.
pub async fn reorder_series_articles(
    pool: &PgPool,
    series_id: Uuid,
    article_ids: &[Uuid],
) -> Result<String, AppError> {
    let series_slug = sqlx::query_scalar!(r#"SELECT slug FROM series WHERE id = $1"#, series_id)
        .fetch_one(pool)
        .await
        .on_missing(|| AppError::NotFound("Series not found".into()))?;

    let current: Vec<Uuid> = sqlx::query_scalar!(
        r#"SELECT article_id FROM series_articles WHERE series_id = $1"#,
        series_id,
    )
    .fetch_all(pool)
    .await?;
    validate_reorder(&current, article_ids)?;

    let mut tx = pool.begin().await?;
    for (i, article_id) in article_ids.iter().enumerate() {
        sqlx::query!(
            r#"UPDATE series_articles SET position = $3
               WHERE series_id = $1 AND article_id = $2"#,
            series_id,
            article_id,
            (i + 1) as i32,
        )
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query!(
        r#"UPDATE series SET updated_at = now() WHERE id = $1"#,
        series_id,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(series_slug)
}

/// Series contexts for the article page strip: every series the article
/// belongs to, with its published prev/next neighbors by position. An
/// article that is itself no longer published gets no contexts — it is
/// invisible on public series surfaces.
pub(in crate::modules::writing) async fn list_article_series_contexts(
    pool: &PgPool,
    article_id: Uuid,
) -> Result<Vec<ArticleSeriesContext>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT s.name, s.slug,
                  pm.prev_title AS "prev_title?", pm.prev_slug AS "prev_slug?",
                  pm.next_title AS "next_title?", pm.next_slug AS "next_slug?"
           FROM (
               SELECT sa.series_id, sa.article_id,
                      LAG(a.title)  OVER w AS prev_title,
                      LAG(a.slug)   OVER w AS prev_slug,
                      LEAD(a.title) OVER w AS next_title,
                      LEAD(a.slug)  OVER w AS next_slug
               FROM series_articles sa
               JOIN articles a ON a.id = sa.article_id
               WHERE a.status = 'published'
                 AND sa.series_id IN (
                     SELECT series_id FROM series_articles WHERE article_id = $1
                 )
               WINDOW w AS (PARTITION BY sa.series_id ORDER BY sa.position)
           ) pm
           JOIN series s ON s.id = pm.series_id
           WHERE pm.article_id = $1
           ORDER BY s.name"#,
        article_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ArticleSeriesContext {
            name: r.name,
            slug: r.slug,
            prev: match (r.prev_title, r.prev_slug) {
                (Some(title), Some(slug)) => Some(SeriesNeighbor { title, slug }),
                _ => None,
            },
            next: match (r.next_title, r.next_slug) {
                (Some(title), Some(slug)) => Some(SeriesNeighbor { title, slug }),
                _ => None,
            },
        })
        .collect())
}

/// Slugs of every series containing the article — publish/archive
/// extend their cache purges with these series' pages.
pub(in crate::modules::writing) async fn series_slugs_for_article_slug(
    pool: &PgPool,
    article_slug: &str,
) -> Result<Vec<String>, AppError> {
    let slugs = sqlx::query_scalar!(
        r#"SELECT s.slug
           FROM series s
           JOIN series_articles sa ON sa.series_id = s.id
           JOIN articles a ON a.id = sa.article_id
           WHERE a.slug = $1"#,
        article_slug,
    )
    .fetch_all(pool)
    .await?;
    Ok(slugs)
}

#[cfg(test)]
mod tests {
    use super::validate_reorder;
    use uuid::Uuid;

    fn ids(n: usize) -> Vec<Uuid> {
        (0..n).map(|_| Uuid::new_v4()).collect()
    }

    #[test]
    fn accepts_permutations() {
        let current = ids(3);
        let mut requested = current.clone();
        requested.reverse();
        assert!(validate_reorder(&current, &requested).is_ok());
        assert!(validate_reorder(&current, &current).is_ok());
        assert!(validate_reorder(&[], &[]).is_ok());
    }

    #[test]
    fn rejects_duplicates() {
        let current = ids(2);
        let requested = vec![current[0], current[0]];
        assert!(validate_reorder(&current, &requested).is_err());
    }

    #[test]
    fn rejects_missing_and_extra_members() {
        let current = ids(3);
        assert!(validate_reorder(&current, &current[..2]).is_err());
        let mut extra = current.clone();
        extra.push(Uuid::new_v4());
        assert!(validate_reorder(&current, &extra).is_err());
        let swapped = vec![current[0], current[1], Uuid::new_v4()];
        assert!(validate_reorder(&current, &swapped).is_err());
    }
}
