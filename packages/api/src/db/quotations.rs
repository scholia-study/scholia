use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::quotation::{
    NoteResponse, NoteWithContextResponse, QuotationResponse, QuotationWithContextResponse,
    TagResponse,
};

// ── Row types ──────────────────────────────────────────────

struct QuotationRow {
    id: Uuid,
    start_number: Option<i32>,
    end_number: Option<i32>,
    sentence_kind: String,
    note_count: Option<i64>,
    created_at: time::OffsetDateTime,
}

struct SentenceLookup {
    id: Uuid,
    node_id: Uuid,
}

struct NoteRow {
    id: Uuid,
    body: String,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

struct TagRow {
    note_id: Uuid,
    tag_id: Uuid,
    tag_name: String,
}

struct OwnerRow {
    user_id: Uuid,
    book_id: Uuid,
}

// ── Helpers ────────────────────────────────────────────────

fn fmt_time(t: time::OffsetDateTime) -> String {
    t.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

fn quotation_from_row(r: QuotationRow) -> QuotationResponse {
    QuotationResponse {
        id: r.id.to_string(),
        anchor_sentence_start_number: r.start_number.unwrap_or(0),
        anchor_sentence_end_number: r.end_number,
        sentence_kind: r.sentence_kind,
        note_count: r.note_count.unwrap_or(0),
        created_at: fmt_time(r.created_at),
    }
}

async fn resolve_sentence(
    pool: &PgPool,
    book_id: Uuid,
    sentence_number: i32,
    sentence_kind: &str,
) -> Result<SentenceLookup, AppError> {
    let is_body = sentence_kind == "body";
    let sent = if is_body {
        sqlx::query_as!(
            SentenceLookup,
            r#"SELECT id, node_id FROM sentences
               WHERE book_id = $1 AND sentence_number = $2 AND block_id IS NOT NULL"#,
            book_id,
            sentence_number,
        )
        .fetch_one(pool)
        .await
    } else {
        sqlx::query_as!(
            SentenceLookup,
            r#"SELECT id, node_id FROM sentences
               WHERE book_id = $1 AND sentence_number = $2 AND footnote_id IS NOT NULL"#,
            book_id,
            sentence_number,
        )
        .fetch_one(pool)
        .await
    }
    .map_err(|_| {
        AppError::BadRequest(format!(
            "Sentence {sentence_number} not found for kind '{sentence_kind}'"
        ))
    })?;
    Ok(sent)
}

// ── Quotation queries ──────────────────────────────────────

pub async fn list_quotations_for_node(
    pool: &PgPool,
    user_id: Uuid,
    book_id: Uuid,
    node_id: Uuid,
) -> Result<Vec<QuotationResponse>, AppError> {
    let rows = sqlx::query_as!(
        QuotationRow,
        r#"SELECT q.id,
                  ss.sentence_number AS "start_number?",
                  se.sentence_number AS "end_number?",
                  q.sentence_kind::TEXT AS "sentence_kind!",
                  COUNT(qn.id) AS "note_count?",
                  q.created_at
           FROM quotations q
           JOIN sentences ss ON ss.id = q.anchor_sentence_start_id
           LEFT JOIN sentences se ON se.id = q.anchor_sentence_end_id
           LEFT JOIN quotation_notes qn ON qn.quotation_id = q.id
           WHERE q.user_id = $1
             AND q.book_id = $2
             AND q.anchor_node_id = $3
           GROUP BY q.id, ss.sentence_number, se.sentence_number
           ORDER BY ss.sentence_number"#,
        user_id,
        book_id,
        node_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(quotation_from_row).collect())
}

pub async fn create_quotation(
    pool: &PgPool,
    user_id: Uuid,
    book_id: Uuid,
    sentence_start: i32,
    sentence_end: Option<i32>,
    sentence_kind: &str,
) -> Result<(QuotationResponse, bool), AppError> {
    let start_sent = resolve_sentence(pool, book_id, sentence_start, sentence_kind).await?;

    let end_sent_id = if let Some(end_num) = sentence_end {
        Some(resolve_sentence(pool, book_id, end_num, sentence_kind).await?.id)
    } else {
        None
    };

    // Try insert, ON CONFLICT do nothing
    let inserted = sqlx::query_scalar!(
        r#"INSERT INTO quotations (
               user_id, book_id, anchor_node_id,
               anchor_sentence_start_id, anchor_sentence_end_id,
               sentence_kind
           ) VALUES ($1, $2, $3, $4, $5, $6::sentence_kind)
           ON CONFLICT (user_id, anchor_sentence_start_id, COALESCE(anchor_sentence_end_id, '00000000-0000-0000-0000-000000000000'))
           DO NOTHING
           RETURNING id"#,
        user_id,
        book_id,
        start_sent.node_id,
        start_sent.id,
        end_sent_id,
        sentence_kind as _,
    )
    .fetch_optional(pool)
    .await?;

    let (quotation_id, created) = if let Some(id) = inserted {
        (id, true)
    } else {
        // Already exists — fetch it
        let id = sqlx::query_scalar!(
            r#"SELECT id FROM quotations
               WHERE user_id = $1
                 AND anchor_sentence_start_id = $2
                 AND COALESCE(anchor_sentence_end_id, '00000000-0000-0000-0000-000000000000') = $3"#,
            user_id,
            start_sent.id,
            end_sent_id.unwrap_or_else(|| Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()),
        )
        .fetch_one(pool)
        .await?;
        (id, false)
    };

    // Fetch full response with note count
    let row = sqlx::query_as!(
        QuotationRow,
        r#"SELECT q.id,
                  ss.sentence_number AS "start_number?",
                  se.sentence_number AS "end_number?",
                  q.sentence_kind::TEXT AS "sentence_kind!",
                  COUNT(qn.id) AS "note_count?",
                  q.created_at
           FROM quotations q
           JOIN sentences ss ON ss.id = q.anchor_sentence_start_id
           LEFT JOIN sentences se ON se.id = q.anchor_sentence_end_id
           LEFT JOIN quotation_notes qn ON qn.quotation_id = q.id
           WHERE q.id = $1
           GROUP BY q.id, ss.sentence_number, se.sentence_number"#,
        quotation_id,
    )
    .fetch_one(pool)
    .await?;

    Ok((quotation_from_row(row), created))
}

pub async fn delete_quotation(
    pool: &PgPool,
    quotation_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"DELETE FROM quotations WHERE id = $1 AND user_id = $2"#,
        quotation_id,
        user_id,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Quotation not found".to_string()));
    }

    Ok(())
}

pub async fn get_quotation_owner(
    pool: &PgPool,
    quotation_id: Uuid,
) -> Result<(Uuid, Uuid), AppError> {
    let row = sqlx::query_as!(
        OwnerRow,
        r#"SELECT user_id, book_id FROM quotations WHERE id = $1"#,
        quotation_id,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Quotation not found".to_string()))?;

    Ok((row.user_id, row.book_id))
}

// ── Note queries ───────────────────────────────────────────

pub async fn list_notes(
    pool: &PgPool,
    quotation_id: Uuid,
) -> Result<Vec<NoteResponse>, AppError> {
    let notes = sqlx::query_as!(
        NoteRow,
        r#"SELECT id, body, created_at, updated_at
           FROM quotation_notes
           WHERE quotation_id = $1
           ORDER BY created_at DESC"#,
        quotation_id,
    )
    .fetch_all(pool)
    .await?;

    if notes.is_empty() {
        return Ok(vec![]);
    }

    let note_ids: Vec<Uuid> = notes.iter().map(|n| n.id).collect();

    let tag_rows = sqlx::query_as!(
        TagRow,
        r#"SELECT qnt.note_id, t.id AS tag_id, t.name AS tag_name
           FROM quotation_note_tags qnt
           JOIN tags t ON t.id = qnt.tag_id
           WHERE qnt.note_id = ANY($1)
           ORDER BY t.name"#,
        &note_ids,
    )
    .fetch_all(pool)
    .await?;

    let mut tags_map: HashMap<Uuid, Vec<TagResponse>> = HashMap::new();
    for tr in tag_rows {
        tags_map
            .entry(tr.note_id)
            .or_default()
            .push(TagResponse {
                id: tr.tag_id.to_string(),
                name: tr.tag_name,
            });
    }

    Ok(notes
        .into_iter()
        .map(|n| NoteResponse {
            id: n.id.to_string(),
            body: n.body,
            tags: tags_map.remove(&n.id).unwrap_or_default(),
            created_at: fmt_time(n.created_at),
            updated_at: fmt_time(n.updated_at),
        })
        .collect())
}

pub async fn create_note(
    pool: &PgPool,
    user_id: Uuid,
    quotation_id: Uuid,
    body: &str,
    tag_names: &[String],
) -> Result<NoteResponse, AppError> {
    let note_id = sqlx::query_scalar!(
        r#"INSERT INTO quotation_notes (quotation_id, body)
           VALUES ($1, $2)
           RETURNING id"#,
        quotation_id,
        body,
    )
    .fetch_one(pool)
    .await?;

    let tags = upsert_and_link_tags(pool, user_id, note_id, tag_names).await?;

    let note = sqlx::query_as!(
        NoteRow,
        r#"SELECT id, body, created_at, updated_at
           FROM quotation_notes WHERE id = $1"#,
        note_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(NoteResponse {
        id: note.id.to_string(),
        body: note.body,
        tags,
        created_at: fmt_time(note.created_at),
        updated_at: fmt_time(note.updated_at),
    })
}

pub async fn update_note(
    pool: &PgPool,
    note_id: Uuid,
    user_id: Uuid,
    body: Option<&str>,
    tag_names: Option<&[String]>,
) -> Result<(), AppError> {
    // Verify ownership via quotation
    let owner = sqlx::query_scalar!(
        r#"SELECT q.user_id
           FROM quotation_notes qn
           JOIN quotations q ON q.id = qn.quotation_id
           WHERE qn.id = $1"#,
        note_id,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Note not found".to_string()))?;

    if owner != user_id {
        return Err(AppError::Forbidden("Not your note".to_string()));
    }

    if let Some(body) = body {
        sqlx::query!(
            r#"UPDATE quotation_notes SET body = $2, updated_at = now() WHERE id = $1"#,
            note_id,
            body,
        )
        .execute(pool)
        .await?;
    }

    if let Some(tags) = tag_names {
        // Clear existing tags
        sqlx::query!(
            r#"DELETE FROM quotation_note_tags WHERE note_id = $1"#,
            note_id,
        )
        .execute(pool)
        .await?;

        upsert_and_link_tags(pool, user_id, note_id, tags).await?;
    }

    Ok(())
}

pub async fn delete_note(
    pool: &PgPool,
    note_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"DELETE FROM quotation_notes
           WHERE id = $1
             AND quotation_id IN (SELECT id FROM quotations WHERE user_id = $2)"#,
        note_id,
        user_id,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Note not found".to_string()));
    }

    Ok(())
}

// ── Tag queries ────────────────────────────────────────────

pub async fn list_tags(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<TagResponse>, AppError> {
    struct Row {
        id: Uuid,
        name: String,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"SELECT id, name FROM tags WHERE user_id = $1 ORDER BY name"#,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| TagResponse {
            id: r.id.to_string(),
            name: r.name,
        })
        .collect())
}

// ── Global listing queries ─────────────────────────────────

struct QuotationWithContextRow {
    id: Uuid,
    book_slug: String,
    book_title: String,
    node_label: String,
    node_slug: String,
    start_number: Option<i32>,
    end_number: Option<i32>,
    sentence_kind: String,
    start_text: Option<String>,
    end_text: Option<String>,
    note_count: Option<i64>,
    created_at: time::OffsetDateTime,
}

pub async fn list_all_quotations(
    pool: &PgPool,
    user_id: Uuid,
    book_slug: Option<&str>,
) -> Result<Vec<QuotationWithContextResponse>, AppError> {
    let rows = sqlx::query_as!(
        QuotationWithContextRow,
        r#"SELECT q.id,
                  b.slug AS "book_slug!",
                  b.title AS "book_title!",
                  n.label AS "node_label!",
                  n.slug AS "node_slug!",
                  ss.sentence_number AS "start_number?",
                  se.sentence_number AS "end_number?",
                  q.sentence_kind::TEXT AS "sentence_kind!",
                  ss.text AS "start_text?",
                  se.text AS "end_text?",
                  COUNT(qn.id) AS "note_count?",
                  q.created_at
           FROM quotations q
           JOIN books b ON b.id = q.book_id
           JOIN toc_nodes n ON n.id = q.anchor_node_id
           JOIN sentences ss ON ss.id = q.anchor_sentence_start_id
           LEFT JOIN sentences se ON se.id = q.anchor_sentence_end_id
           LEFT JOIN quotation_notes qn ON qn.quotation_id = q.id
           WHERE q.user_id = $1
             AND ($2::TEXT IS NULL OR b.slug = $2)
           GROUP BY q.id, b.slug, b.title, n.label, n.slug, ss.sentence_number, se.sentence_number, ss.text, se.text
           ORDER BY q.created_at DESC"#,
        user_id,
        book_slug,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let start_snippet = r.start_text.map(|t| truncate_snippet(&t, 80));
            let end_snippet = r.end_text.map(|t| truncate_snippet(&t, 60));
            QuotationWithContextResponse {
                id: r.id.to_string(),
                book_slug: r.book_slug,
                book_title: r.book_title,
                node_label: r.node_label,
                node_slug: r.node_slug,
                anchor_sentence_start_number: r.start_number.unwrap_or(0),
                anchor_sentence_end_number: r.end_number,
                sentence_kind: r.sentence_kind,
                start_text_snippet: start_snippet,
                end_text_snippet: end_snippet,
                note_count: r.note_count.unwrap_or(0),
                created_at: fmt_time(r.created_at),
            }
        })
        .collect())
}

struct NoteWithContextRow {
    id: Uuid,
    body: String,
    quotation_id: Uuid,
    book_slug: String,
    book_title: String,
    node_label: String,
    node_slug: String,
    start_number: Option<i32>,
    end_number: Option<i32>,
    sentence_kind: String,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

pub async fn list_all_notes(
    pool: &PgPool,
    user_id: Uuid,
    book_slug: Option<&str>,
) -> Result<Vec<NoteWithContextResponse>, AppError> {
    let rows = sqlx::query_as!(
        NoteWithContextRow,
        r#"SELECT qn.id, qn.body, qn.quotation_id,
                  b.slug AS "book_slug!",
                  b.title AS "book_title!",
                  n.label AS "node_label!",
                  n.slug AS "node_slug!",
                  ss.sentence_number AS "start_number?",
                  se.sentence_number AS "end_number?",
                  q.sentence_kind::TEXT AS "sentence_kind!",
                  qn.created_at, qn.updated_at
           FROM quotation_notes qn
           JOIN quotations q ON q.id = qn.quotation_id
           JOIN books b ON b.id = q.book_id
           JOIN toc_nodes n ON n.id = q.anchor_node_id
           JOIN sentences ss ON ss.id = q.anchor_sentence_start_id
           LEFT JOIN sentences se ON se.id = q.anchor_sentence_end_id
           WHERE q.user_id = $1
             AND ($2::TEXT IS NULL OR b.slug = $2)
           ORDER BY qn.created_at DESC"#,
        user_id,
        book_slug,
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(vec![]);
    }

    let note_ids: Vec<Uuid> = rows.iter().map(|n| n.id).collect();
    let tag_rows = sqlx::query_as!(
        TagRow,
        r#"SELECT qnt.note_id, t.id AS tag_id, t.name AS tag_name
           FROM quotation_note_tags qnt
           JOIN tags t ON t.id = qnt.tag_id
           WHERE qnt.note_id = ANY($1)
           ORDER BY t.name"#,
        &note_ids,
    )
    .fetch_all(pool)
    .await?;

    let mut tags_map: HashMap<Uuid, Vec<TagResponse>> = HashMap::new();
    for tr in tag_rows {
        tags_map
            .entry(tr.note_id)
            .or_default()
            .push(TagResponse {
                id: tr.tag_id.to_string(),
                name: tr.tag_name,
            });
    }

    Ok(rows
        .into_iter()
        .map(|r| NoteWithContextResponse {
            id: r.id.to_string(),
            body: r.body,
            tags: tags_map.remove(&r.id).unwrap_or_default(),
            book_slug: r.book_slug,
            book_title: r.book_title,
            node_label: r.node_label,
            node_slug: r.node_slug,
            anchor_sentence_start_number: r.start_number.unwrap_or(0),
            anchor_sentence_end_number: r.end_number,
            sentence_kind: r.sentence_kind,
            quotation_id: r.quotation_id.to_string(),
            created_at: fmt_time(r.created_at),
            updated_at: fmt_time(r.updated_at),
        })
        .collect())
}

fn truncate_snippet(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        let mut end = max_len;
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &text[..end])
    }
}

// ── Helpers ────────────────────────────────────────────────

async fn upsert_and_link_tags(
    pool: &PgPool,
    user_id: Uuid,
    note_id: Uuid,
    tag_names: &[String],
) -> Result<Vec<TagResponse>, AppError> {
    let mut tags = Vec::new();

    for name in tag_names {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            continue;
        }

        let tag_id = sqlx::query_scalar!(
            r#"INSERT INTO tags (user_id, name) VALUES ($1, $2)
               ON CONFLICT (user_id, name) DO UPDATE SET name = EXCLUDED.name
               RETURNING id"#,
            user_id,
            trimmed,
        )
        .fetch_one(pool)
        .await?;

        sqlx::query!(
            r#"INSERT INTO quotation_note_tags (note_id, tag_id) VALUES ($1, $2)
               ON CONFLICT DO NOTHING"#,
            note_id,
            tag_id,
        )
        .execute(pool)
        .await?;

        tags.push(TagResponse {
            id: tag_id.to_string(),
            name: trimmed.to_string(),
        });
    }

    Ok(tags)
}
