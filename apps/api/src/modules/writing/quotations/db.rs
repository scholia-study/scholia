use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::modules::writing::quotations::models::{
    NoteLimitsResponse, NoteResponse, NoteWithContextResponse, QuotationLimitsResponse,
    QuotationResponse, QuotationWithContextResponse, TagResponse,
};
use crate::system::auth::permissions::{Permission, resolve_permissions};
use crate::system::error::AppError;

// ── Tier limits ──────────────────────────────────────────
// Free tier: 50 quotations / 50 notes. Paid / staff: 10 000 of each
// (a hard cap to prevent abuse, not a usage target).
const FREE_QUOTATIONS: i32 = 50;
const FREE_NOTES: i32 = 50;
const PAID_QUOTATIONS: i32 = 10_000;
const PAID_NOTES: i32 = 10_000;

// ── Row types ──────────────────────────────────────────────

struct QuotationRow {
    id: Uuid,
    start_number: Option<i32>,
    end_number: Option<i32>,
    sentence_kind: String,
    note_count: Option<i64>,
    created_at: time::OffsetDateTime,
}

/// Extended row for `list_quotations_for_node` — same columns as
/// `QuotationRow` plus enrichment for badges and verse-marker projection.
///
/// `anchor_*` fields carry the peer quote's actual stored anchor (used
/// for badges, source links, "saved in WEB" text). `projected_*` fields
/// carry the target-local coordinates where the marker should render —
/// they coincide with `anchor_*` for same-book quotes and resolve via
/// cross_translation_alignments for cross-translation projection.
struct QuotationRowEx {
    id: Uuid,
    start_number: Option<i32>,
    end_number: Option<i32>,
    sentence_kind: String,
    note_count: Option<i64>,
    created_at: time::OffsetDateTime,
    book_slug: Option<String>,
    translation_label: Option<String>,
    anchor_source_ref: Option<String>,
    anchor_verse_start: Option<String>,
    anchor_verse_end: Option<String>,
    projected_source_ref: Option<String>,
    projected_verse_start: Option<String>,
    projected_verse_end: Option<String>,
}

pub(crate) struct SentenceLookup {
    pub(crate) id: Uuid,
    pub(crate) node_id: Uuid,
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
        book_slug: None,
        translation_label: None,
        anchor_source_ref: None,
        anchor_verse_start: None,
        anchor_verse_end: None,
        projected_source_ref: None,
        projected_verse_start: None,
        projected_verse_end: None,
    }
}

fn quotation_from_row_ex(r: QuotationRowEx) -> QuotationResponse {
    QuotationResponse {
        id: r.id.to_string(),
        anchor_sentence_start_number: r.start_number.unwrap_or(0),
        anchor_sentence_end_number: r.end_number,
        sentence_kind: r.sentence_kind,
        note_count: r.note_count.unwrap_or(0),
        created_at: fmt_time(r.created_at),
        book_slug: r.book_slug,
        translation_label: r.translation_label,
        anchor_source_ref: r.anchor_source_ref,
        anchor_verse_start: r.anchor_verse_start,
        anchor_verse_end: r.anchor_verse_end,
        projected_source_ref: r.projected_source_ref,
        projected_verse_start: r.projected_verse_start,
        projected_verse_end: r.projected_verse_end,
    }
}

// Shared with article_passage_references, which resolves article
// ::quotation directives to the same anchor shape. Not-found is
// BadRequest so callers can tell a bad sentence number apart from a
// genuine database failure (which propagates as Internal).
pub(crate) async fn resolve_sentence(
    pool: &PgPool,
    book_id: Uuid,
    sentence_number: i32,
    sentence_kind: &str,
) -> Result<SentenceLookup, AppError> {
    let sent = match sentence_kind {
        "body" => {
            sqlx::query_as!(
                SentenceLookup,
                r#"SELECT id, node_id FROM sentences
                   WHERE book_id = $1 AND sentence_number = $2 AND block_id IS NOT NULL"#,
                book_id,
                sentence_number,
            )
            .fetch_optional(pool)
            .await?
        }
        // Figure anchors have no sentence_number (they sit outside the body
        // enumeration); they are addressed by the block's figure_number.
        "figure" => {
            sqlx::query_as!(
                SentenceLookup,
                r#"SELECT s.id, s.node_id FROM sentences s
                   JOIN content_blocks cb ON cb.id = s.block_id
                   WHERE s.book_id = $1 AND cb.figure_number = $2"#,
                book_id,
                sentence_number,
            )
            .fetch_optional(pool)
            .await?
        }
        _ => {
            sqlx::query_as!(
                SentenceLookup,
                r#"SELECT id, node_id FROM sentences
                   WHERE book_id = $1 AND sentence_number = $2 AND footnote_id IS NOT NULL"#,
                book_id,
                sentence_number,
            )
            .fetch_optional(pool)
            .await?
        }
    }
    .ok_or_else(|| {
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
    // Returns own-book quotations PLUS peer-translation quotations whose
    // anchor toc-node shares this node's `source_ref` and whose book
    // shares the translation root. The frontend uses peer entries for
    // visual-marker projection only — saved quotations themselves stay
    // locked to their translation (PLAN_BIG_BOOKS.md Q7).
    //
    // Each row carries:
    //   - `book_slug` and `translation_label` — for the source badge
    //   - `anchor_source_ref` + `anchor_verse_start` / `_end` — for
    //     cross-translation marker matching by verse identity. The
    //     verse fields are populated only when the book has a `verse`
    //     reference system; books without it (Kant) leave them null
    //     and projection falls back to sentence_number matching, which
    //     is safe within a single book.
    //
    // book_id is retained in the signature for future per-book scopes
    // but is unused here — the source-of-truth is the node id, which
    // determines the translation family via toc_nodes/sources joins.
    let _ = book_id;
    // Two-stage projection through cross_translation_alignments:
    //
    // 1. `target_verses` enumerates the verse markers in the target
    //    chapter and resolves each to canonical (source_ref, ref_value)
    //    coordinates. No alignment row → identity mapping. Alignment
    //    row with non-null canonical → use the row. Alignment row with
    //    null canonical → translation-only verse (no peer can project
    //    here).
    // 2. The peer quote's anchor verse is similarly resolved to canonical
    //    via the peer's alignment row (or identity). The two canonical
    //    coordinates are matched, and the matching `target_verses` row's
    //    `local_ref` becomes the projected verse the marker should land
    //    on. Same-book quotes always project to their own anchor coords
    //    (the canonical roundtrip would also resolve correctly, but the
    //    direct fast path keeps Kant — which has no verse markers — and
    //    same-translation-different-source-ref edge cases simple).
    //
    // Books without a verse reference system (Kant) leave anchor_verse_*
    // null and the projection match falls back to source_ref equality
    // for the same-book branch; cross-book quotes only join through the
    // verse-keyed target_verses CTE, which is empty for non-verse books.
    let rows = sqlx::query_as!(
        QuotationRowEx,
        r#"WITH target AS (
               SELECT tn.source_ref AS target_source_ref,
                      tn.book_id AS target_book_id,
                      tn.id AS target_node_id,
                      COALESCE(s.translation_of_id, s.id) AS work_root
               FROM toc_nodes tn
               JOIN books b ON b.id = tn.book_id
               JOIN sources s ON s.id = b.source_id
               WHERE tn.id = $2
           ),
           target_verses AS (
               SELECT DISTINCT
                   pm.ref_value AS local_ref,
                   CASE
                       WHEN va.book_id IS NULL THEN t.target_source_ref
                       ELSE va.canonical_source_ref
                   END AS canonical_src,
                   CASE
                       WHEN va.book_id IS NULL THEN pm.ref_value
                       ELSE va.canonical_ref_value
                   END AS canonical_ref
               FROM target t
               JOIN sentences s ON s.node_id = t.target_node_id
               JOIN page_markers pm ON pm.sentence_id = s.id
               JOIN reference_systems rs ON rs.id = pm.system_id
               LEFT JOIN cross_translation_alignments va
                      ON va.book_id = t.target_book_id
                     AND va.system_slug = 'verse'
                     AND va.source_ref = t.target_source_ref
                     AND va.local_ref_value = pm.ref_value
               WHERE rs.slug = 'verse'
           )
           SELECT q.id,
                  COALESCE(ss.sentence_number, cbs.figure_number) AS "start_number?",
                  se.sentence_number AS "end_number?",
                  q.sentence_kind::TEXT AS "sentence_kind!",
                  COUNT(qn.id) AS "note_count?",
                  q.created_at,
                  qb.slug AS "book_slug?",
                  -- Compact translation badge: prefer the source's
                  -- `publisher` when it's short (e.g. "KJV", "WEB"),
                  -- otherwise fall back to the language code. Kant DE
                  -- thus shows "DE"; Bible KJV shows "KJV".
                  CASE
                      WHEN qs.publisher IS NOT NULL AND char_length(qs.publisher) <= 6
                          THEN qs.publisher
                      ELSE UPPER(qb.language)
                  END AS "translation_label?",
                  qtn.source_ref AS "anchor_source_ref?",
                  pm_start.ref_value AS "anchor_verse_start?",
                  pm_end.ref_value AS "anchor_verse_end?",
                  -- Projection coords: target-local for cross-translation,
                  -- identity for same-book.
                  CASE
                      WHEN qb.id = (SELECT target_book_id FROM target)
                          THEN qtn.source_ref
                      ELSE (SELECT target_source_ref FROM target)
                  END AS "projected_source_ref?",
                  CASE
                      WHEN qb.id = (SELECT target_book_id FROM target)
                          THEN pm_start.ref_value
                      ELSE COALESCE(tv_start.local_ref, tv_end.local_ref)
                  END AS "projected_verse_start?",
                  CASE
                      WHEN qb.id = (SELECT target_book_id FROM target)
                          THEN pm_end.ref_value
                      ELSE COALESCE(tv_end.local_ref, tv_start.local_ref)
                  END AS "projected_verse_end?"
           FROM quotations q
           JOIN toc_nodes qtn ON qtn.id = q.anchor_node_id
           JOIN books qb ON qb.id = qtn.book_id
           JOIN sources qs ON qs.id = qb.source_id
           JOIN sentences ss ON ss.id = q.anchor_sentence_start_id
           -- Figure anchors carry their number on the block, not the sentence
           LEFT JOIN content_blocks cbs ON cbs.id = ss.block_id
           LEFT JOIN sentences se ON se.id = q.anchor_sentence_end_id
           -- Peer's verse markers (if the peer book has a 'verse' system)
           LEFT JOIN page_markers pm_start
                  ON pm_start.sentence_id = ss.id
                 AND pm_start.system_id IN (
                     SELECT id FROM reference_systems
                     WHERE book_id = qb.id AND slug = 'verse'
                 )
           LEFT JOIN page_markers pm_end
                  ON pm_end.sentence_id = se.id
                 AND pm_end.system_id IN (
                     SELECT id FROM reference_systems
                     WHERE book_id = qb.id AND slug = 'verse'
                 )
           -- Peer's alignment row (if any) for start/end verses
           LEFT JOIN cross_translation_alignments va_start
                  ON va_start.book_id = qb.id
                 AND va_start.system_slug = 'verse'
                 AND va_start.source_ref = qtn.source_ref
                 AND va_start.local_ref_value = pm_start.ref_value
           LEFT JOIN cross_translation_alignments va_end
                  ON va_end.book_id = qb.id
                 AND va_end.system_slug = 'verse'
                 AND va_end.source_ref = qtn.source_ref
                 AND va_end.local_ref_value = pm_end.ref_value
           -- Match peer's canonical coords to a target verse
           LEFT JOIN target_verses tv_start
                  ON tv_start.canonical_src = CASE
                         WHEN va_start.book_id IS NULL THEN qtn.source_ref
                         ELSE va_start.canonical_source_ref
                     END
                 AND tv_start.canonical_ref = CASE
                         WHEN va_start.book_id IS NULL THEN pm_start.ref_value
                         ELSE va_start.canonical_ref_value
                     END
           LEFT JOIN target_verses tv_end
                  ON tv_end.canonical_src = CASE
                         WHEN va_end.book_id IS NULL THEN qtn.source_ref
                         ELSE va_end.canonical_source_ref
                     END
                 AND tv_end.canonical_ref = CASE
                         WHEN va_end.book_id IS NULL THEN pm_end.ref_value
                         ELSE va_end.canonical_ref_value
                     END
           LEFT JOIN quotation_notes qn ON qn.quotation_id = q.id
           WHERE q.user_id = $1
             AND COALESCE(qs.translation_of_id, qs.id) = (SELECT work_root FROM target)
             AND (
                 -- Same-book branch: include all quotes anchored to the
                 -- target's source_ref. Covers Kant (no verse markers)
                 -- and Bible same-translation/same-chapter alike.
                 (qb.id = (SELECT target_book_id FROM target)
                  AND qtn.source_ref = (SELECT target_source_ref FROM target))
                 OR
                 -- Cross-book branch: include peer-translation quotes whose
                 -- canonical coords project onto any verse in the target
                 -- chapter. Drift cases (Romans doxology, DARBY Psalms)
                 -- resolve naturally through the alignment table.
                 (qb.id <> (SELECT target_book_id FROM target)
                  AND tv_start.local_ref IS NOT NULL)
             )
           GROUP BY q.id, ss.id, se.id, ss.sentence_number, se.sentence_number,
                    cbs.figure_number,
                    qb.id, qb.slug, qb.language, qs.publisher, qtn.source_ref,
                    pm_start.ref_value, pm_end.ref_value,
                    tv_start.local_ref, tv_end.local_ref
           ORDER BY COALESCE(ss.sentence_number, cbs.figure_number)"#,
        user_id,
        node_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(quotation_from_row_ex).collect())
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
        Some(
            resolve_sentence(pool, book_id, end_num, sentence_kind)
                .await?
                .id,
        )
    } else {
        None
    };

    // Resolve the effective bibliographic source for this anchor (Shape 3:
    // a quotation in a compilation cites the per-book child source, not
    // just the hosted-text root).
    let effective_source_id =
        crate::modules::corpus::resolve_effective_source(pool, book_id, start_sent.node_id).await?;

    // Try insert, ON CONFLICT do nothing
    let inserted = sqlx::query_scalar!(
        r#"INSERT INTO quotations (
               user_id, book_id, anchor_node_id,
               anchor_sentence_start_id, anchor_sentence_end_id,
               sentence_kind, source_id
           ) VALUES ($1, $2, $3, $4, $5, $6::sentence_kind, $7)
           ON CONFLICT (user_id, anchor_sentence_start_id, COALESCE(anchor_sentence_end_id, '00000000-0000-0000-0000-000000000000'))
           DO NOTHING
           RETURNING id"#,
        user_id,
        book_id,
        start_sent.node_id,
        start_sent.id,
        end_sent_id,
        sentence_kind as _,
        effective_source_id,
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
                  COALESCE(ss.sentence_number, cbs.figure_number) AS "start_number?",
                  se.sentence_number AS "end_number?",
                  q.sentence_kind::TEXT AS "sentence_kind!",
                  COUNT(qn.id) AS "note_count?",
                  q.created_at
           FROM quotations q
           JOIN sentences ss ON ss.id = q.anchor_sentence_start_id
           LEFT JOIN content_blocks cbs ON cbs.id = ss.block_id
           LEFT JOIN sentences se ON se.id = q.anchor_sentence_end_id
           LEFT JOIN quotation_notes qn ON qn.quotation_id = q.id
           WHERE q.id = $1
           GROUP BY q.id, ss.sentence_number, cbs.figure_number, se.sentence_number"#,
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

pub async fn list_notes(pool: &PgPool, quotation_id: Uuid) -> Result<Vec<NoteResponse>, AppError> {
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
        tags_map.entry(tr.note_id).or_default().push(TagResponse {
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

pub async fn delete_note(pool: &PgPool, note_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
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

pub async fn list_tags(pool: &PgPool, user_id: Uuid) -> Result<Vec<TagResponse>, AppError> {
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

// ── Tier limit queries ─────────────────────────────────────

/// Total quotations for a user across both books and articles.
pub async fn get_user_quotation_count(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
    let count = sqlx::query_scalar!(
        r#"SELECT
               (SELECT COUNT(*) FROM quotations WHERE user_id = $1)
             + (SELECT COUNT(*) FROM article_quotations WHERE user_id = $1)
               AS "count!""#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// Total notes for a user across all quotations they own.
pub async fn get_user_note_count(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM quotation_notes qn
           LEFT JOIN quotations q ON q.id = qn.quotation_id
           LEFT JOIN article_quotations aq ON aq.id = qn.article_quotation_id
           WHERE q.user_id = $1 OR aq.user_id = $1"#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub fn get_quotation_limit(roles: &[String]) -> i32 {
    if resolve_permissions(roles).contains(&Permission::QuotationsLimit10000) {
        PAID_QUOTATIONS
    } else {
        FREE_QUOTATIONS
    }
}

pub fn get_note_limit(roles: &[String]) -> i32 {
    if resolve_permissions(roles).contains(&Permission::NotesLimit10000) {
        PAID_NOTES
    } else {
        FREE_NOTES
    }
}

pub async fn get_quotation_limits_response(
    pool: &PgPool,
    user_id: Uuid,
    roles: &[String],
) -> Result<QuotationLimitsResponse, AppError> {
    Ok(QuotationLimitsResponse {
        max: get_quotation_limit(roles),
        current: get_user_quotation_count(pool, user_id).await?,
    })
}

pub async fn get_note_limits_response(
    pool: &PgPool,
    user_id: Uuid,
    roles: &[String],
) -> Result<NoteLimitsResponse, AppError> {
    Ok(NoteLimitsResponse {
        max: get_note_limit(roles),
        current: get_user_note_count(pool, user_id).await?,
    })
}

// ── Global listing queries ─────────────────────────────────

struct QuotationWithContextRow {
    id: Uuid,
    book_slug: String,
    book_title: String,
    translation_label: Option<String>,
    parent_compilation_title: Option<String>,
    node_label: String,
    node_slug: String,
    start_number: Option<i32>,
    end_number: Option<i32>,
    sentence_kind: String,
    main_number: Option<i32>,
    start_text: Option<String>,
    end_text: Option<String>,
    has_source_view: Option<bool>,
    note_count: Option<i64>,
    created_at: time::OffsetDateTime,
}

pub async fn list_all_quotations(
    pool: &PgPool,
    user_id: Uuid,
    book_slug: Option<&str>,
) -> Result<Vec<QuotationWithContextResponse>, AppError> {
    // Citation source resolution (Shape 3): join via q.source_id (the
    // denormalized effective source) rather than b.source_id. For non-
    // compilations these are identical; for a quotation from Genesis
    // inside a King James Bible book, q.source_id points at the Genesis
    // child source and `parent` resolves to the Bible compilation.
    let rows = sqlx::query_as!(
        QuotationWithContextRow,
        r#"SELECT q.id,
                  b.slug AS "book_slug!",
                  COALESCE(s.title_display, s.title) AS "book_title!",
                  CASE
                      WHEN bs.publisher IS NOT NULL AND char_length(bs.publisher) <= 6
                          THEN bs.publisher
                      ELSE UPPER(b.language)
                  END AS "translation_label?",
                  COALESCE(parent.title_display, parent.title) AS "parent_compilation_title?",
                  n.label AS "node_label!",
                  n.slug AS "node_slug!",
                  COALESCE(ss.sentence_number, cbs.figure_number) AS "start_number?",
                  se.sentence_number AS "end_number?",
                  q.sentence_kind::TEXT AS "sentence_kind!",
                  ms.sentence_number AS "main_number?",
                  ss.text AS "start_text?",
                  se.text AS "end_text?",
                  -- True iff this book is a translation AND the original
                  -- work is itself a hosted text (has a `books` row).
                  -- Bible translations point at the canonical "The Bible"
                  -- bibliographic root, which has no books row → false.
                  -- Hegel/Kant translations point at the imported original
                  -- work → true.
                  EXISTS (
                      SELECT 1 FROM books orig_b
                      WHERE orig_b.source_id = bs.translation_of_id
                  ) AS "has_source_view?",
                  COUNT(qn.id) AS "note_count?",
                  q.created_at
           FROM quotations q
           JOIN books b ON b.id = q.book_id
           -- The book's own root source (carries language + publisher
           -- of the translation itself), distinct from `s` which is the
           -- effective citation source resolved per quotation.
           JOIN sources bs ON bs.id = b.source_id
           JOIN sources s ON s.id = q.source_id
           LEFT JOIN sources parent ON parent.id = s.parent_source_id
           JOIN toc_nodes n ON n.id = q.anchor_node_id
           JOIN sentences ss ON ss.id = q.anchor_sentence_start_id
           LEFT JOIN content_blocks cbs ON cbs.id = ss.block_id
           LEFT JOIN sentences se ON se.id = q.anchor_sentence_end_id
           LEFT JOIN footnotes fn ON fn.id = ss.footnote_id
           LEFT JOIN sentences ms ON ms.id = fn.anchor_sentence_id
           LEFT JOIN quotation_notes qn ON qn.quotation_id = q.id
           WHERE q.user_id = $1
             AND ($2::TEXT IS NULL OR b.slug = $2)
           GROUP BY q.id, b.slug, b.language, bs.publisher, bs.translation_of_id, s.title_display, s.title, parent.title_display, parent.title, n.label, n.slug, ss.sentence_number, cbs.figure_number, se.sentence_number, ss.text, se.text, ms.sentence_number
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
                translation_label: r.translation_label,
                book_title: r.book_title,
                parent_compilation_title: r.parent_compilation_title,
                node_label: r.node_label,
                node_slug: r.node_slug,
                anchor_sentence_start_number: r.start_number.unwrap_or(0),
                anchor_sentence_end_number: r.end_number,
                sentence_kind: r.sentence_kind,
                anchor_main_sentence_number: r.main_number,
                start_text_snippet: start_snippet,
                end_text_snippet: end_snippet,
                has_source_view: r.has_source_view.unwrap_or(false),
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
    translation_label: Option<String>,
    book_title: String,
    parent_compilation_title: Option<String>,
    node_label: String,
    node_slug: String,
    start_number: Option<i32>,
    end_number: Option<i32>,
    sentence_kind: String,
    main_number: Option<i32>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

pub async fn list_all_notes(
    pool: &PgPool,
    user_id: Uuid,
    book_slug: Option<&str>,
) -> Result<Vec<NoteWithContextResponse>, AppError> {
    // Shape 3: cite via q.source_id (effective source) + optional parent
    // compilation. See list_all_quotations for the rationale.
    let rows = sqlx::query_as!(
        NoteWithContextRow,
        r#"SELECT qn.id, qn.body, qn.quotation_id AS "quotation_id!",
                  b.slug AS "book_slug!",
                  CASE
                      WHEN bs.publisher IS NOT NULL AND char_length(bs.publisher) <= 6
                          THEN bs.publisher
                      ELSE UPPER(b.language)
                  END AS "translation_label?",
                  COALESCE(s.title_display, s.title) AS "book_title!",
                  COALESCE(parent.title_display, parent.title) AS "parent_compilation_title?",
                  n.label AS "node_label!",
                  n.slug AS "node_slug!",
                  COALESCE(ss.sentence_number, cbs.figure_number) AS "start_number?",
                  se.sentence_number AS "end_number?",
                  q.sentence_kind::TEXT AS "sentence_kind!",
                  ms.sentence_number AS "main_number?",
                  qn.created_at, qn.updated_at
           FROM quotation_notes qn
           JOIN quotations q ON q.id = qn.quotation_id
           JOIN books b ON b.id = q.book_id
           JOIN sources bs ON bs.id = b.source_id
           JOIN sources s ON s.id = q.source_id
           LEFT JOIN sources parent ON parent.id = s.parent_source_id
           JOIN toc_nodes n ON n.id = q.anchor_node_id
           JOIN sentences ss ON ss.id = q.anchor_sentence_start_id
           LEFT JOIN content_blocks cbs ON cbs.id = ss.block_id
           LEFT JOIN sentences se ON se.id = q.anchor_sentence_end_id
           LEFT JOIN footnotes fn ON fn.id = ss.footnote_id
           LEFT JOIN sentences ms ON ms.id = fn.anchor_sentence_id
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
        tags_map.entry(tr.note_id).or_default().push(TagResponse {
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
            translation_label: r.translation_label,
            book_title: r.book_title,
            parent_compilation_title: r.parent_compilation_title,
            node_label: r.node_label,
            node_slug: r.node_slug,
            anchor_sentence_start_number: r.start_number.unwrap_or(0),
            anchor_sentence_end_number: r.end_number,
            sentence_kind: r.sentence_kind,
            anchor_main_sentence_number: r.main_number,
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
