use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::db::sources::fetch_source_persons;
use crate::error::AppError;
use crate::models::resource::{
    ParentSourceResponse, ResourceResponse, SourcePersonResponse, SourceResponse,
};

// ── Row types ──────────────────────────────────────────────

struct ResourceRow {
    id: Uuid,
    resource_type: String,
    verbatim_kind: Option<String>,
    start_number: Option<i32>,
    end_number: Option<i32>,
    sentence_kind: String,
    quoted_text: Option<String>,
    editor_note: Option<String>,
    source_page_start: Option<i32>,
    source_page_end: Option<i32>,
    source_location_freeform: Option<String>,
    is_featured: bool,
    admin_notes: Option<String>,
    created_at: time::OffsetDateTime,
    // Source fields (joined)
    src_id: Option<Uuid>,
    src_type: Option<String>,
    src_title: Option<String>,
    src_title_display: Option<String>,
    src_year: Option<i16>,
    src_publisher: Option<String>,
    src_isbn: Option<Vec<String>>,
    src_doi: Option<String>,
    src_edition: Option<String>,
    src_volume: Option<String>,
    src_journal_name: Option<String>,
    src_url: Option<String>,
    src_page_start: Option<i32>,
    src_page_end: Option<i32>,
    src_parent_id: Option<Uuid>,
    src_translation_of_id: Option<Uuid>,
}

// ── Queries ────────────────────────────────────────────────

pub async fn list_resources(
    pool: &PgPool,
    book_id: Uuid,
    start: i32,
    end: i32,
    kind: &str,
) -> Result<Vec<ResourceResponse>, AppError> {
    let rows = sqlx::query_as!(
        ResourceRow,
        r#"SELECT r.id,
                  r.resource_type::TEXT AS "resource_type!",
                  r.verbatim_kind::TEXT AS "verbatim_kind?",
                  ss.sentence_number AS "start_number?",
                  se.sentence_number AS "end_number?",
                  r.sentence_kind::TEXT AS "sentence_kind!",
                  r.quoted_text, r.editor_note,
                  r.source_page_start, r.source_page_end, r.source_location_freeform,
                  r.is_featured, r.admin_notes, r.created_at,
                  s.id AS "src_id?",
                  s.source_type::TEXT AS "src_type?",
                  s.title AS "src_title?",
                  s.title_display AS "src_title_display?",
                  s.publication_year AS "src_year?",
                  s.publisher AS "src_publisher?",
                  s.isbn AS "src_isbn?",
                  s.doi AS "src_doi?",
                  s.edition AS "src_edition?",
                  s.volume AS "src_volume?",
                  s.journal_name AS "src_journal_name?",
                  s.url AS "src_url?",
                  s.page_start AS "src_page_start?",
                  s.page_end AS "src_page_end?",
                  s.parent_source_id AS "src_parent_id?",
                  s.translation_of_id AS "src_translation_of_id?"
           FROM resources r
           JOIN sentences ss ON ss.id = r.anchor_sentence_start_id
           LEFT JOIN sentences se ON se.id = r.anchor_sentence_end_id
           LEFT JOIN sources s ON s.id = r.source_id
           WHERE r.book_id = $1
             AND r.archived_at IS NULL
             AND r.sentence_kind = $2::sentence_kind
             AND ss.sentence_number <= $4
             AND COALESCE(se.sentence_number, ss.sentence_number) >= $3
           ORDER BY r.is_featured DESC,
                    s.publication_year DESC NULLS LAST,
                    r.source_page_start ASC NULLS LAST"#,
        book_id,
        kind as _,
        start,
        end,
    )
    .fetch_all(pool)
    .await?;

    // Collect unique source IDs and parent IDs for batch fetching
    let source_ids: Vec<Uuid> = rows.iter().filter_map(|r| r.src_id).collect();
    let parent_ids: Vec<Uuid> = rows
        .iter()
        .filter_map(|r| r.src_parent_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let persons_map = fetch_source_persons(pool, &source_ids).await?;

    // Batch fetch parent sources
    let mut parent_persons_map: HashMap<Uuid, Vec<SourcePersonResponse>> = HashMap::new();
    if !parent_ids.is_empty() {
        parent_persons_map = fetch_source_persons(pool, &parent_ids).await?;
    }

    let parent_rows = if !parent_ids.is_empty() {
        sqlx::query_as!(
            ParentRow,
            r#"SELECT id, source_type::TEXT AS "source_type!", title, publication_year, publisher
               FROM sources
               WHERE id = ANY($1)"#,
            &parent_ids,
        )
        .fetch_all(pool)
        .await?
    } else {
        vec![]
    };

    let parent_map: HashMap<Uuid, ParentSourceResponse> = parent_rows
        .into_iter()
        .map(|r| {
            let persons = parent_persons_map.get(&r.id).cloned().unwrap_or_default();
            (
                r.id,
                ParentSourceResponse {
                    id: r.id.to_string(),
                    source_type: r.source_type,
                    title: r.title,
                    publication_year: r.publication_year,
                    publisher: r.publisher,
                    persons,
                },
            )
        })
        .collect();

    // Build responses
    Ok(rows
        .into_iter()
        .map(|r| {
            let source = r.src_id.map(|sid| {
                let persons = persons_map.get(&sid).cloned().unwrap_or_default();
                let parent = r
                    .src_parent_id
                    .and_then(|pid| parent_map.get(&pid).cloned())
                    .map(Box::new);
                SourceResponse {
                    id: sid.to_string(),
                    source_type: r.src_type.unwrap_or_default(),
                    title: r.src_title.unwrap_or_default(),
                    title_display: r.src_title_display,
                    publication_year: r.src_year,
                    publisher: r.src_publisher,
                    isbn: r.src_isbn,
                    doi: r.src_doi,
                    edition: r.src_edition,
                    volume: r.src_volume,
                    journal_name: r.src_journal_name,
                    url: r.src_url,
                    page_start: r.src_page_start,
                    page_end: r.src_page_end,
                    translation_of_id: r.src_translation_of_id.map(|id| id.to_string()),
                    persons,
                    parent,
                }
            });

            ResourceResponse {
                id: r.id.to_string(),
                resource_type: r.resource_type,
                verbatim_kind: r.verbatim_kind,
                anchor_sentence_start_number: r.start_number.unwrap_or(0),
                anchor_sentence_end_number: r.end_number,
                sentence_kind: r.sentence_kind,
                quoted_text: r.quoted_text,
                editor_note: r.editor_note,
                source,
                source_page_start: r.source_page_start,
                source_page_end: r.source_page_end,
                source_location_freeform: r.source_location_freeform,
                is_featured: r.is_featured,
                admin_notes: r.admin_notes,
                created_at: r.created_at.format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
            }
        })
        .collect())
}

struct SentenceLookup {
    id: Uuid,
    node_id: Uuid,
}

pub async fn create_resource(
    pool: &PgPool,
    book_id: Uuid,
    resource_type: &str,
    verbatim_kind: Option<&str>,
    sentence_start: i32,
    sentence_end: Option<i32>,
    sentence_kind: &str,
    source_id: Option<Uuid>,
    source_page_start: Option<i32>,
    source_page_end: Option<i32>,
    source_location_freeform: Option<&str>,
    quoted_text: Option<&str>,
    editor_note: Option<&str>,
    is_featured: bool,
    admin_notes: Option<&str>,
) -> Result<Uuid, AppError> {
    // Validate range
    let actual_end = sentence_end.unwrap_or(sentence_start);
    if actual_end - sentence_start + 1 > 15 {
        return Err(AppError::BadRequest(
            "Sentence range cannot exceed 15 sentences".to_string(),
        ));
    }

    // Resolve start sentence
    let is_body = sentence_kind == "body";
    let start_sent = if is_body {
        sqlx::query_as!(
            SentenceLookup,
            r#"SELECT id, node_id FROM sentences
               WHERE book_id = $1 AND sentence_number = $2 AND block_id IS NOT NULL"#,
            book_id,
            sentence_start,
        )
        .fetch_one(pool)
        .await
    } else {
        sqlx::query_as!(
            SentenceLookup,
            r#"SELECT id, node_id FROM sentences
               WHERE book_id = $1 AND sentence_number = $2 AND footnote_id IS NOT NULL"#,
            book_id,
            sentence_start,
        )
        .fetch_one(pool)
        .await
    }
    .map_err(|_| {
        AppError::BadRequest(format!(
            "Sentence {sentence_start} not found for kind '{sentence_kind}'"
        ))
    })?;

    // Resolve end sentence (if range)
    let end_sent_id = if let Some(end_num) = sentence_end {
        let end_sent = if is_body {
            sqlx::query_as!(
                SentenceLookup,
                r#"SELECT id, node_id FROM sentences
                   WHERE book_id = $1 AND sentence_number = $2 AND block_id IS NOT NULL"#,
                book_id,
                end_num,
            )
            .fetch_one(pool)
            .await
        } else {
            sqlx::query_as!(
                SentenceLookup,
                r#"SELECT id, node_id FROM sentences
                   WHERE book_id = $1 AND sentence_number = $2 AND footnote_id IS NOT NULL"#,
                book_id,
                end_num,
            )
            .fetch_one(pool)
            .await
        }
        .map_err(|_| {
            AppError::BadRequest(format!(
                "Sentence {end_num} not found for kind '{sentence_kind}'"
            ))
        })?;
        Some(end_sent.id)
    } else {
        None
    };

    let id = sqlx::query_scalar!(
        r#"INSERT INTO resources (
               book_id, resource_type, anchor_node_id,
               anchor_sentence_start_id, anchor_sentence_end_id,
               sentence_kind, source_id,
               source_page_start, source_page_end, source_location_freeform,
               verbatim_kind, quoted_text, editor_note, is_featured, admin_notes
           ) VALUES (
               $1, $2::resource_type, $3, $4, $5,
               $6::sentence_kind, $7, $8, $9, $10,
               $11::verbatim_kind, $12, $13, $14, $15
           )
           RETURNING id"#,
        book_id,
        resource_type as _,
        start_sent.node_id,
        start_sent.id,
        end_sent_id,
        sentence_kind as _,
        source_id,
        source_page_start,
        source_page_end,
        source_location_freeform,
        verbatim_kind as _,
        quoted_text,
        editor_note,
        is_featured,
        admin_notes,
    )
    .fetch_one(pool)
    .await?;

    Ok(id)
}

pub async fn update_resource(
    pool: &PgPool,
    resource_id: Uuid,
    book_id: Uuid,
    resource_type: Option<&str>,
    verbatim_kind: Option<&str>,
    sentence_start: Option<i32>,
    sentence_end: Option<i32>,
    sentence_kind: Option<&str>,
    source_id: Option<Uuid>,
    source_page_start: Option<i32>,
    source_page_end: Option<i32>,
    source_location_freeform: Option<&str>,
    quoted_text: Option<&str>,
    editor_note: Option<&str>,
    is_featured: Option<bool>,
    admin_notes: Option<&str>,
) -> Result<(), AppError> {
    // If sentence range is being updated, resolve IDs
    if let Some(start) = sentence_start {
        let end = sentence_end.unwrap_or(start);
        if end - start + 1 > 15 {
            return Err(AppError::BadRequest(
                "Sentence range cannot exceed 15 sentences".to_string(),
            ));
        }

        let kind = sentence_kind.unwrap_or("body");
        let is_body = kind == "body";

        let start_sent = if is_body {
            sqlx::query_as!(
                SentenceLookup,
                r#"SELECT id, node_id FROM sentences
                   WHERE book_id = $1 AND sentence_number = $2 AND block_id IS NOT NULL"#,
                book_id,
                start,
            )
            .fetch_one(pool)
            .await
        } else {
            sqlx::query_as!(
                SentenceLookup,
                r#"SELECT id, node_id FROM sentences
                   WHERE book_id = $1 AND sentence_number = $2 AND footnote_id IS NOT NULL"#,
                book_id,
                start,
            )
            .fetch_one(pool)
            .await
        }
        .map_err(|_| AppError::BadRequest(format!("Sentence {start} not found")))?;

        let end_sent_id = if sentence_end.is_some() {
            let end_sent = if is_body {
                sqlx::query_as!(
                    SentenceLookup,
                    r#"SELECT id, node_id FROM sentences
                       WHERE book_id = $1 AND sentence_number = $2 AND block_id IS NOT NULL"#,
                    book_id,
                    end,
                )
                .fetch_one(pool)
                .await
            } else {
                sqlx::query_as!(
                    SentenceLookup,
                    r#"SELECT id, node_id FROM sentences
                       WHERE book_id = $1 AND sentence_number = $2 AND footnote_id IS NOT NULL"#,
                    book_id,
                    end,
                )
                .fetch_one(pool)
                .await
            }
            .map_err(|_| AppError::BadRequest(format!("Sentence {end} not found")))?;
            Some(end_sent.id)
        } else {
            None
        };

        sqlx::query!(
            r#"UPDATE resources
               SET anchor_node_id = $2,
                   anchor_sentence_start_id = $3,
                   anchor_sentence_end_id = $4,
                   sentence_kind = COALESCE($5::sentence_kind, sentence_kind),
                   updated_at = now()
               WHERE id = $1"#,
            resource_id,
            start_sent.node_id,
            start_sent.id,
            end_sent_id,
            sentence_kind as _,
        )
        .execute(pool)
        .await?;
    }

    // Update remaining fields
    sqlx::query!(
        r#"UPDATE resources
           SET resource_type = COALESCE($2::resource_type, resource_type),
               verbatim_kind = COALESCE($3::verbatim_kind, verbatim_kind),
               source_id = COALESCE($4, source_id),
               source_page_start = COALESCE($5, source_page_start),
               source_page_end = COALESCE($6, source_page_end),
               source_location_freeform = COALESCE($7, source_location_freeform),
               quoted_text = COALESCE($8, quoted_text),
               editor_note = COALESCE($9, editor_note),
               is_featured = COALESCE($10, is_featured),
               admin_notes = COALESCE($11, admin_notes),
               updated_at = now()
           WHERE id = $1"#,
        resource_id,
        resource_type as _,
        verbatim_kind as _,
        source_id,
        source_page_start,
        source_page_end,
        source_location_freeform,
        quoted_text,
        editor_note,
        is_featured,
        admin_notes,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn soft_delete_resource(
    pool: &PgPool,
    resource_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"UPDATE resources
           SET archived_at = now(), archived_by = $2
           WHERE id = $1 AND archived_at IS NULL"#,
        resource_id,
        user_id,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Resource not found".to_string()));
    }

    Ok(())
}

pub async fn get_resource_book_id(pool: &PgPool, resource_id: Uuid) -> Result<Uuid, AppError> {
    let book_id = sqlx::query_scalar!(
        r#"SELECT book_id FROM resources WHERE id = $1"#,
        resource_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(book_id)
}

struct ParentRow {
    id: Uuid,
    source_type: String,
    title: String,
    publication_year: Option<i16>,
    publisher: Option<String>,
}
