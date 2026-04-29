use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::resource::{
    ParentSourceResponse, ReferenceCheckResponse, ReferencedArticle, ReferencedArticles,
    ReferencedChildSource, ReferencedChildSources, ReferencedResources, SourcePersonResponse,
    SourceResponse, SourceSearchResponse,
};

// ── Row types ──────────────────────────────────────────────

struct SourceRow {
    id: Uuid,
    source_type: String,
    title: String,
    title_display: Option<String>,
    publication_year: Option<i16>,
    publisher: Option<String>,
    isbn: Option<Vec<String>>,
    doi: Option<String>,
    edition: Option<String>,
    volume: Option<String>,
    journal_name: Option<String>,
    url: Option<String>,
    page_start: Option<i32>,
    page_end: Option<i32>,
    parent_source_id: Option<Uuid>,
    translation_of_id: Option<Uuid>,
    created_by: Uuid,
    protected: bool,
}

struct SourcePersonRow {
    source_id: Uuid,
    person_id: Uuid,
    name: String,
    sort_name: Option<String>,
    role: String,
    position: i16,
    person_created_by: Uuid,
    person_protected: bool,
}

// ── Queries ────────────────────────────────────────────────

pub async fn search_sources(
    pool: &PgPool,
    query: &str,
) -> Result<Vec<SourceSearchResponse>, AppError> {
    let pattern = format!("%{query}%");

    let rows = sqlx::query_as!(
        SourceRow,
        r#"SELECT DISTINCT s.id, s.source_type::TEXT AS "source_type!", s.title, s.title_display,
                  s.publication_year, s.publisher, s.isbn, s.doi, s.edition, s.volume,
                  s.journal_name, s.url, s.page_start, s.page_end,
                  s.parent_source_id, s.translation_of_id,
                  s.created_by AS "created_by!", s.protected
           FROM sources s
           LEFT JOIN source_persons sp ON sp.source_id = s.id
           LEFT JOIN persons p ON p.id = sp.person_id
           WHERE s.title ILIKE $1 OR p.name ILIKE $1
           ORDER BY s.title
           LIMIT 20"#,
        pattern,
    )
    .fetch_all(pool)
    .await?;

    let source_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
    let persons = fetch_source_persons(pool, &source_ids).await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let sp = persons.get(&r.id).cloned().unwrap_or_default();
            SourceSearchResponse {
                id: r.id.to_string(),
                source_type: r.source_type,
                title: r.title,
                publication_year: r.publication_year,
                created_by: r.created_by.to_string(),
                protected: r.protected,
                persons: sp,
            }
        })
        .collect())
}

pub async fn get_source(pool: &PgPool, source_id: Uuid) -> Result<SourceResponse, AppError> {
    let row = sqlx::query_as!(
        SourceRow,
        r#"SELECT id, source_type::TEXT AS "source_type!", title, title_display, publication_year,
                  publisher, isbn, doi, edition, volume, journal_name, url,
                  page_start, page_end, parent_source_id, translation_of_id,
                  created_by AS "created_by!", protected
           FROM sources
           WHERE id = $1"#,
        source_id,
    )
    .fetch_one(pool)
    .await?;

    let persons_map = fetch_source_persons(pool, &[source_id]).await?;
    let persons = persons_map.get(&source_id).cloned().unwrap_or_default();

    let parent = if let Some(parent_id) = row.parent_source_id {
        Some(Box::new(fetch_parent_source(pool, parent_id).await?))
    } else {
        None
    };

    Ok(build_source_response(row, persons, parent))
}

pub async fn create_source(
    pool: &PgPool,
    source_type: &str,
    title: &str,
    title_display: Option<&str>,
    publication_year: Option<i16>,
    publisher: Option<&str>,
    isbn: Option<&[String]>,
    doi: Option<&str>,
    edition: Option<&str>,
    volume: Option<&str>,
    journal_name: Option<&str>,
    url: Option<&str>,
    page_start: Option<i32>,
    page_end: Option<i32>,
    parent_source_id: Option<Uuid>,
    translation_of_id: Option<Uuid>,
    created_by: Uuid,
) -> Result<SourceResponse, AppError> {
    let id = sqlx::query_scalar!(
        r#"INSERT INTO sources (source_type, title, title_display, publication_year, publisher, isbn, doi,
                                edition, volume, journal_name, url, page_start, page_end,
                                parent_source_id, translation_of_id, created_by)
           VALUES ($1::source_type, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
           RETURNING id"#,
        source_type as _,
        title,
        title_display,
        publication_year,
        publisher,
        isbn,
        doi,
        edition,
        volume,
        journal_name,
        url,
        page_start,
        page_end,
        parent_source_id,
        translation_of_id,
        created_by,
    )
    .fetch_one(pool)
    .await?;

    get_source(pool, id).await
}

pub async fn update_source(
    pool: &PgPool,
    source_id: Uuid,
    title: Option<&str>,
    title_display: Option<&str>,
    publication_year: Option<i16>,
    publisher: Option<&str>,
    isbn: Option<&[String]>,
    doi: Option<&str>,
    edition: Option<&str>,
    volume: Option<&str>,
    journal_name: Option<&str>,
    url: Option<&str>,
    page_start: Option<i32>,
    page_end: Option<i32>,
    parent_source_id: Option<Uuid>,
    translation_of_id: Option<Uuid>,
    protected: Option<bool>,
) -> Result<SourceResponse, AppError> {
    sqlx::query!(
        r#"UPDATE sources
           SET title = COALESCE($2, title),
               title_display = COALESCE($3, title_display),
               publication_year = COALESCE($4, publication_year),
               publisher = COALESCE($5, publisher),
               isbn = COALESCE($6, isbn),
               doi = COALESCE($7, doi),
               edition = COALESCE($8, edition),
               volume = COALESCE($9, volume),
               journal_name = COALESCE($10, journal_name),
               url = COALESCE($11, url),
               page_start = COALESCE($12, page_start),
               page_end = COALESCE($13, page_end),
               parent_source_id = COALESCE($14, parent_source_id),
               translation_of_id = COALESCE($15, translation_of_id),
               protected = COALESCE($16, protected)
           WHERE id = $1"#,
        source_id,
        title,
        title_display,
        publication_year,
        publisher,
        isbn,
        doi,
        edition,
        volume,
        journal_name,
        url,
        page_start,
        page_end,
        parent_source_id,
        translation_of_id,
        protected,
    )
    .execute(pool)
    .await?;

    get_source(pool, source_id).await
}

pub async fn link_source_person(
    pool: &PgPool,
    source_id: Uuid,
    person_id: Uuid,
    role: &str,
    position: i16,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"INSERT INTO source_persons (source_id, person_id, role, position)
           VALUES ($1, $2, $3::source_person_role, $4)
           ON CONFLICT (source_id, person_id, role)
           DO UPDATE SET position = EXCLUDED.position"#,
        source_id,
        person_id,
        role as _,
        position,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn unlink_source_person(
    pool: &PgPool,
    source_id: Uuid,
    person_id: Uuid,
    role: &str,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"DELETE FROM source_persons
           WHERE source_id = $1 AND person_id = $2 AND role = $3::source_person_role"#,
        source_id,
        person_id,
        role as _,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn check_source_references(
    pool: &PgPool,
    source_id: Uuid,
    viewing_user: Uuid,
) -> Result<ReferenceCheckResponse, AppError> {
    // 1. Resource references (book quotations)
    struct ResourceRow {
        id: Uuid,
    }
    let resource_rows = sqlx::query_as!(
        ResourceRow,
        r#"SELECT id FROM resources
           WHERE source_id = $1 AND archived_at IS NULL"#,
        source_id,
    )
    .fetch_all(pool)
    .await?;
    let resources = ReferencedResources {
        count: resource_rows.len() as i64,
        ids: resource_rows
            .into_iter()
            .map(|r| r.id.to_string())
            .collect(),
    };

    // 2. Child sources (parent_source_id or translation_of_id points here)
    struct ChildRow {
        id: Uuid,
        title: String,
        relation: String,
    }
    let child_rows = sqlx::query_as!(
        ChildRow,
        r#"SELECT id, title,
                  CASE WHEN parent_source_id = $1 THEN 'parent' ELSE 'translation' END
                  AS "relation!"
           FROM sources
           WHERE parent_source_id = $1 OR translation_of_id = $1"#,
        source_id,
    )
    .fetch_all(pool)
    .await?;
    let child_sources = ReferencedChildSources {
        count: child_rows.len() as i64,
        items: child_rows
            .into_iter()
            .map(|r| ReferencedChildSource {
                id: r.id.to_string(),
                title: r.title,
                relation: r.relation,
            })
            .collect(),
    };

    // 3. Article citations — scan markdown for ":cite{...sources="...<uuid>..."...}"
    // Cheap enough for a delete-time check; uses ILIKE on the source ID substring.
    struct ArticleRow {
        id: Uuid,
        title: String,
        slug: String,
        status: String,
        user_id: Uuid,
    }
    let id_str = source_id.to_string();
    let id_pattern = format!("%{id_str}%");
    let article_rows = sqlx::query_as!(
        ArticleRow,
        r#"SELECT id, title, slug, status::TEXT AS "status!", user_id
           FROM articles
           WHERE markdown ILIKE $1"#,
        id_pattern,
    )
    .fetch_all(pool)
    .await?;
    let articles = ReferencedArticles {
        count: article_rows.len() as i64,
        items: article_rows
            .into_iter()
            .map(|r| ReferencedArticle {
                id: r.id.to_string(),
                title: r.title,
                slug: r.slug,
                status: r.status,
                is_mine: r.user_id == viewing_user,
            })
            .collect(),
    };

    let total = resources.count + child_sources.count + articles.count;
    Ok(ReferenceCheckResponse {
        total,
        resources,
        child_sources,
        articles,
    })
}

pub async fn browse_sources(
    pool: &PgPool,
    q: Option<&str>,
    source_type: Option<&str>,
    created_by: Option<Uuid>,
    protected: Option<bool>,
    page: i32,
    per_page: i32,
) -> Result<(Vec<SourceSearchResponse>, i64), AppError> {
    let offset = (page - 1).max(0) as i64 * per_page as i64;
    let limit = per_page as i64;
    let pattern = q.map(|s| format!("%{s}%"));

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM sources s
           WHERE ($1::TEXT IS NULL OR s.title ILIKE $1)
             AND ($2::source_type IS NULL OR s.source_type = $2::source_type)
             AND ($3::UUID IS NULL OR s.created_by = $3)
             AND ($4::BOOLEAN IS NULL OR s.protected = $4)"#,
        pattern.as_deref(),
        source_type as _,
        created_by,
        protected,
    )
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query_as!(
        SourceRow,
        r#"SELECT s.id, s.source_type::TEXT AS "source_type!", s.title, s.title_display,
                  s.publication_year, s.publisher, s.isbn, s.doi, s.edition, s.volume,
                  s.journal_name, s.url, s.page_start, s.page_end,
                  s.parent_source_id, s.translation_of_id,
                  s.created_by AS "created_by!", s.protected
           FROM sources s
           WHERE ($1::TEXT IS NULL OR s.title ILIKE $1)
             AND ($2::source_type IS NULL OR s.source_type = $2::source_type)
             AND ($3::UUID IS NULL OR s.created_by = $3)
             AND ($4::BOOLEAN IS NULL OR s.protected = $4)
           ORDER BY s.created_at DESC
           LIMIT $5 OFFSET $6"#,
        pattern.as_deref(),
        source_type as _,
        created_by,
        protected,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let source_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
    let persons = fetch_source_persons(pool, &source_ids).await?;

    let sources = rows
        .into_iter()
        .map(|r| {
            let sp = persons.get(&r.id).cloned().unwrap_or_default();
            SourceSearchResponse {
                id: r.id.to_string(),
                source_type: r.source_type,
                title: r.title,
                publication_year: r.publication_year,
                created_by: r.created_by.to_string(),
                protected: r.protected,
                persons: sp,
            }
        })
        .collect();

    Ok((sources, total))
}

pub async fn delete_source(pool: &PgPool, source_id: Uuid) -> Result<(), AppError> {
    let affected = sqlx::query!(r#"DELETE FROM sources WHERE id = $1"#, source_id)
        .execute(pool)
        .await?
        .rows_affected();
    if affected == 0 {
        return Err(AppError::NotFound("Source not found".into()));
    }
    Ok(())
}

pub async fn is_source_protected(pool: &PgPool, source_id: Uuid) -> Result<bool, AppError> {
    let protected =
        sqlx::query_scalar!(r#"SELECT protected FROM sources WHERE id = $1"#, source_id,)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Source not found".into()))?;

    Ok(protected)
}

pub async fn is_person_protected(pool: &PgPool, person_id: Uuid) -> Result<bool, AppError> {
    let protected =
        sqlx::query_scalar!(r#"SELECT protected FROM persons WHERE id = $1"#, person_id,)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Person not found".into()))?;

    Ok(protected)
}

// ── Helpers ────────────────────────────────────────────────

pub async fn fetch_source_persons(
    pool: &PgPool,
    source_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<SourcePersonResponse>>, AppError> {
    let rows = sqlx::query_as!(
        SourcePersonRow,
        r#"SELECT sp.source_id, sp.person_id, p.name, p.sort_name,
                  sp.role::TEXT AS "role!", sp.position,
                  p.created_by AS "person_created_by!", p.protected AS "person_protected!"
           FROM source_persons sp
           JOIN persons p ON p.id = sp.person_id
           WHERE sp.source_id = ANY($1)
           ORDER BY sp.position"#,
        source_ids,
    )
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<Uuid, Vec<SourcePersonResponse>> = HashMap::new();
    for r in rows {
        map.entry(r.source_id)
            .or_default()
            .push(SourcePersonResponse {
                person_id: r.person_id.to_string(),
                name: r.name,
                sort_name: r.sort_name,
                role: r.role,
                position: r.position,
                created_by: r.person_created_by.to_string(),
                protected: r.person_protected,
            });
    }
    Ok(map)
}

pub async fn fetch_parent_source(
    pool: &PgPool,
    parent_id: Uuid,
) -> Result<ParentSourceResponse, AppError> {
    struct ParentRow {
        id: Uuid,
        source_type: String,
        title: String,
        publication_year: Option<i16>,
        publisher: Option<String>,
    }

    let row = sqlx::query_as!(
        ParentRow,
        r#"SELECT id, source_type::TEXT AS "source_type!", title, publication_year, publisher
           FROM sources
           WHERE id = $1"#,
        parent_id,
    )
    .fetch_one(pool)
    .await?;

    let persons_map = fetch_source_persons(pool, &[parent_id]).await?;
    let persons = persons_map.get(&parent_id).cloned().unwrap_or_default();

    Ok(ParentSourceResponse {
        id: row.id.to_string(),
        source_type: row.source_type,
        title: row.title,
        publication_year: row.publication_year,
        publisher: row.publisher,
        persons,
    })
}

fn build_source_response(
    row: SourceRow,
    persons: Vec<SourcePersonResponse>,
    parent: Option<Box<ParentSourceResponse>>,
) -> SourceResponse {
    SourceResponse {
        id: row.id.to_string(),
        source_type: row.source_type,
        title: row.title,
        title_display: row.title_display,
        publication_year: row.publication_year,
        publisher: row.publisher,
        isbn: row.isbn,
        doi: row.doi,
        edition: row.edition,
        volume: row.volume,
        journal_name: row.journal_name,
        url: row.url,
        page_start: row.page_start,
        page_end: row.page_end,
        translation_of_id: row.translation_of_id.map(|id| id.to_string()),
        created_by: row.created_by.to_string(),
        protected: row.protected,
        persons,
        parent,
    }
}
