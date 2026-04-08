use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::resource::{
    ParentSourceResponse, SourcePersonResponse, SourceResponse,
    SourceSearchResponse,
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
}

struct SourcePersonRow {
    source_id: Uuid,
    person_id: Uuid,
    name: String,
    sort_name: Option<String>,
    role: String,
    position: i16,
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
                  s.parent_source_id, s.translation_of_id
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
                  page_start, page_end, parent_source_id, translation_of_id
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
    source_type: Option<&str>,
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
) -> Result<SourceResponse, AppError> {
    sqlx::query!(
        r#"UPDATE sources
           SET source_type = COALESCE($2::source_type, source_type),
               title = COALESCE($3, title),
               title_display = COALESCE($4, title_display),
               publication_year = COALESCE($5, publication_year),
               publisher = COALESCE($6, publisher),
               isbn = COALESCE($7, isbn),
               doi = COALESCE($8, doi),
               edition = COALESCE($9, edition),
               volume = COALESCE($10, volume),
               journal_name = COALESCE($11, journal_name),
               url = COALESCE($12, url),
               page_start = COALESCE($13, page_start),
               page_end = COALESCE($14, page_end),
               parent_source_id = COALESCE($15, parent_source_id),
               translation_of_id = COALESCE($16, translation_of_id)
           WHERE id = $1"#,
        source_id,
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
) -> Result<(i64, Vec<String>), AppError> {
    struct Row {
        id: Uuid,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"SELECT id FROM resources
           WHERE source_id = $1 AND archived_at IS NULL"#,
        source_id,
    )
    .fetch_all(pool)
    .await?;

    let count = rows.len() as i64;
    let ids = rows.into_iter().map(|r| r.id.to_string()).collect();
    Ok((count, ids))
}

// ── Helpers ────────────────────────────────────────────────

pub async fn fetch_source_persons(
    pool: &PgPool,
    source_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<SourcePersonResponse>>, AppError> {
    let rows = sqlx::query_as!(
        SourcePersonRow,
        r#"SELECT sp.source_id, sp.person_id, p.name, p.sort_name,
                  sp.role::TEXT AS "role!", sp.position
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
        persons,
        parent,
    }
}
