//! Derived index of article `::quotation` directives → sentence anchors.
//!
//! `article_passage_references` rows let the reader surface which
//! platform articles quote a passage. Rows are derived data: re-synced
//! from the article markdown on every save and on publish, rebuildable
//! at any time. The read path filters on article status and projects
//! across translations, mirroring `list_quotations_for_node`.

use regex::Regex;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::OnceLock;
use uuid::Uuid;

use crate::modules::writing::article_passage_references::models::PassageArticleResponse;
use crate::modules::writing::quotations::db::resolve_sentence;
use crate::system::error::AppError;

/// The `::quotation{…}` directive pattern, shared with
/// `render_article_markdown` so the renderer and this index can never
/// drift on what counts as a quotation embed.
pub(crate) fn quotation_directive_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"::quotation\{([^}]+)\}").expect("directive regex"))
}

#[derive(Debug, PartialEq)]
pub(crate) struct PassageDirective {
    pub book_slug: String,
    pub start: i32,
    pub end: Option<i32>,
    pub kind: String,
}

/// Attr grammar as accepted by `render_article_markdown`: quoted
/// `key="value"` pairs first, then bare `key=number` for keys not
/// already seen.
fn parse_directive_attrs(attrs_str: &str) -> HashMap<String, String> {
    static QUOTED_RE: OnceLock<Regex> = OnceLock::new();
    static BARE_RE: OnceLock<Regex> = OnceLock::new();
    let quoted = QUOTED_RE.get_or_init(|| Regex::new(r#"(\w+)="([^"]*)""#).expect("attr regex"));
    let bare = BARE_RE.get_or_init(|| Regex::new(r"(\w+)=(\d+)").expect("num regex"));

    let mut attrs = HashMap::new();
    for cap in quoted.captures_iter(attrs_str) {
        attrs
            .entry(cap[1].to_string())
            .or_insert_with(|| cap[2].to_string());
    }
    for cap in bare.captures_iter(attrs_str) {
        attrs
            .entry(cap[1].to_string())
            .or_insert_with(|| cap[2].to_string());
    }
    attrs
}

/// Extract the passage anchors from an article's `::quotation`
/// directives. Malformed directives (missing book/start, unparsable
/// numbers, unknown kind) are skipped — the renderer tolerates them,
/// so the index must too.
pub(crate) fn parse_quotation_directives(markdown: &str) -> Vec<PassageDirective> {
    quotation_directive_regex()
        .captures_iter(markdown)
        .filter_map(|caps| {
            let attrs = parse_directive_attrs(&caps[1]);
            let book_slug = attrs.get("book").filter(|s| !s.is_empty())?.clone();
            let start: i32 = attrs.get("start")?.parse().ok()?;
            let end: Option<i32> = attrs.get("end").and_then(|e| e.parse().ok());
            let kind = attrs
                .get("kind")
                .cloned()
                .unwrap_or_else(|| "body".to_string());
            if !matches!(kind.as_str(), "body" | "footnote" | "figure") {
                return None;
            }
            let (start, end) = match end {
                Some(e) if e < start => (e, Some(start)),
                Some(e) if e == start => (start, None),
                other => (start, other),
            };
            Some(PassageDirective {
                book_slug,
                start,
                end,
                kind,
            })
        })
        .collect()
}

struct ResolvedAnchor {
    book_id: Uuid,
    node_id: Uuid,
    start_id: Uuid,
    end_id: Option<Uuid>,
    kind: String,
}

/// Resolve one directive endpoint, separating "this directive points
/// at nothing" (Ok(None) — the row is skipped, like the renderer
/// tolerating a directive that hydrates to nothing) from a genuine
/// database failure (Err — fails the whole sync/save).
async fn resolve_directive_sentence(
    pool: &PgPool,
    book_slug: &str,
    book_id: Uuid,
    number: i32,
    kind: &str,
) -> Result<Option<crate::modules::writing::quotations::db::SentenceLookup>, AppError> {
    match resolve_sentence(pool, book_id, number, kind).await {
        Ok(s) => Ok(Some(s)),
        Err(AppError::BadRequest(_)) => {
            tracing::warn!(book = %book_slug, number, kind = %kind,
                "article passage ref: sentence not found, skipping directive");
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

/// Re-derive the passage-reference rows for an article from its
/// markdown. Called on every markdown save and on publish.
///
/// Directives address sentences by book-wide `sentence_number`, which
/// can shift across text re-imports. The anchored UUID rows survive
/// reconcile like quotations do; only the next save re-resolves from
/// the (possibly stale) numbers. A directive that no longer resolves
/// produces no row (logged at warn) — a stale quote must not block the
/// author from saving the article. Database errors always propagate.
pub async fn sync_article_passage_references(
    pool: &PgPool,
    article_id: Uuid,
    markdown: &str,
) -> Result<(), AppError> {
    // Resolution is read-only and happens outside the transaction.
    let mut book_ids: HashMap<String, Option<Uuid>> = HashMap::new();
    let mut anchors: Vec<ResolvedAnchor> = Vec::new();
    for d in parse_quotation_directives(markdown) {
        let book_id = match book_ids.get(&d.book_slug) {
            Some(cached) => *cached,
            None => {
                let found =
                    sqlx::query_scalar!(r#"SELECT id FROM books WHERE slug = $1"#, d.book_slug,)
                        .fetch_optional(pool)
                        .await?;
                book_ids.insert(d.book_slug.clone(), found);
                found
            }
        };
        let Some(book_id) = book_id else {
            tracing::warn!(book = %d.book_slug,
                "article passage ref: unknown book, skipping directive");
            continue;
        };

        let Some(start) =
            resolve_directive_sentence(pool, &d.book_slug, book_id, d.start, &d.kind).await?
        else {
            continue;
        };
        let end_id = match d.end {
            Some(n) => {
                match resolve_directive_sentence(pool, &d.book_slug, book_id, n, &d.kind).await? {
                    Some(s) => Some(s.id),
                    None => continue,
                }
            }
            None => None,
        };
        anchors.push(ResolvedAnchor {
            book_id,
            node_id: start.node_id,
            start_id: start.id,
            end_id,
            kind: d.kind,
        });
    }

    // Delete-and-reinsert in one transaction so readers never see a
    // half-synced article. Diffing buys nothing at a handful of rows.
    let mut tx = pool.begin().await?;
    sqlx::query!(
        r#"DELETE FROM article_passage_references WHERE article_id = $1"#,
        article_id,
    )
    .execute(&mut *tx)
    .await?;
    for a in anchors {
        sqlx::query!(
            r#"INSERT INTO article_passage_references (
                   article_id, book_id, anchor_node_id,
                   anchor_sentence_start_id, anchor_sentence_end_id,
                   sentence_kind
               ) VALUES ($1, $2, $3, $4, $5, $6::sentence_kind)
               ON CONFLICT DO NOTHING"#,
            article_id,
            a.book_id,
            a.node_id,
            a.start_id,
            a.end_id,
            a.kind as _,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

struct PassageArticleRow {
    id: Uuid,
    slug: String,
    title: String,
    author_user_id: Uuid,
    author_display_name: String,
    author_handle: Option<String>,
    published_at: Option<time::OffsetDateTime>,
    total: i64,
}

fn fmt_time(t: time::OffsetDateTime) -> String {
    t.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// Published/archived articles whose quoted ranges overlap the target
/// selection, deduped per article, newest first. `total` (same value on
/// every row via a window count) feeds the menu badge.
///
/// Cross-translation matching, three branches:
///   (a) same book — plain sentence_number overlap (figures compare
///       `figure_number`), the semantics of `list_resources`;
///   (b) cross-book, non-verse works (Kant, Ibsen — block-locked
///       translations): anchor sentences map into the target book by
///       `natural_key` equality, then overlap. Gated on the peer book
///       having no 'verse' reference system so Bible translations
///       (which are NOT block-locked) can never pseudo-match here;
///   (c) cross-book, verse works (Bible): non-empty intersection of
///       the anchor's covered verses and the selection's covered
///       verses, both canonicalized through
///       cross_translation_alignments (identity when no row) — the
///       projection pattern of `list_quotations_for_node`. Both
///       covered-verse windows start at the verse in progress (nearest
///       marker at-or-before the range start within its node), so
///       mid-verse ranges match correctly.
pub async fn list_article_references(
    pool: &PgPool,
    book_id: Uuid,
    start: i32,
    end: i32,
    kind: &str,
    limit: i64,
    offset: i64,
) -> Result<(Vec<PassageArticleResponse>, i64), AppError> {
    let rows = sqlx::query_as!(
        PassageArticleRow,
        r#"WITH target AS (
               SELECT b.id AS book_id,
                      COALESCE(src.translation_of_id, src.id) AS work_root
               FROM books b
               JOIN sources src ON src.id = b.source_id
               WHERE b.id = $1
           ),
           sel_lo AS (
               SELECT COALESCE((
                   SELECT MAX(s2.sentence_number)
                   FROM sentences s2
                   JOIN page_markers pm2 ON pm2.sentence_id = s2.id
                   JOIN reference_systems rs2 ON rs2.id = pm2.system_id
                                             AND rs2.slug = 'verse'
                   WHERE s2.book_id = (SELECT book_id FROM target)
                     AND s2.node_id = (
                         SELECT s3.node_id FROM sentences s3
                         WHERE s3.book_id = (SELECT book_id FROM target)
                           AND s3.sentence_number = $2
                           AND s3.block_id IS NOT NULL
                     )
                     AND s2.sentence_number <= $2
               ), $2) AS lo
           ),
           target_verses AS (
               SELECT DISTINCT
                   CASE WHEN va.book_id IS NULL THEN tn.source_ref
                        ELSE va.canonical_source_ref END AS canonical_src,
                   CASE WHEN va.book_id IS NULL THEN pm.ref_value
                        ELSE va.canonical_ref_value END AS canonical_ref
               FROM target t
               CROSS JOIN sel_lo
               JOIN sentences s ON s.book_id = t.book_id
                               AND s.block_id IS NOT NULL
                               AND s.sentence_number BETWEEN sel_lo.lo AND $3
               JOIN toc_nodes tn ON tn.id = s.node_id
               JOIN page_markers pm ON pm.sentence_id = s.id
               JOIN reference_systems rs ON rs.id = pm.system_id
                                        AND rs.slug = 'verse'
               LEFT JOIN cross_translation_alignments va
                      ON va.book_id = t.book_id
                     AND va.system_slug = 'verse'
                     AND va.source_ref = tn.source_ref
                     AND va.local_ref_value = pm.ref_value
           ),
           matches AS (
               SELECT DISTINCT apr.article_id
               FROM article_passage_references apr
               CROSS JOIN target t
               JOIN books pb ON pb.id = apr.book_id
               JOIN sources ps ON ps.id = pb.source_id
               JOIN sentences pss ON pss.id = apr.anchor_sentence_start_id
               LEFT JOIN content_blocks pcb ON pcb.id = pss.block_id
               LEFT JOIN sentences pse ON pse.id = apr.anchor_sentence_end_id
               WHERE apr.sentence_kind = $4::sentence_kind
                 AND COALESCE(ps.translation_of_id, ps.id) = t.work_root
                 AND (
                     (apr.book_id = t.book_id
                      AND COALESCE(pss.sentence_number, pcb.figure_number) <= $3
                      AND COALESCE(pse.sentence_number, pss.sentence_number,
                                   pcb.figure_number) >= $2)
                     OR
                     (apr.book_id <> t.book_id
                      AND NOT EXISTS (
                          SELECT 1 FROM reference_systems rsx
                          WHERE rsx.book_id = apr.book_id AND rsx.slug = 'verse'
                      )
                      AND EXISTS (
                          SELECT 1
                          FROM sentences ts
                          JOIN sentences te
                            ON te.book_id = t.book_id
                           AND te.natural_key =
                               COALESCE(pse.natural_key, pss.natural_key)
                          WHERE ts.book_id = t.book_id
                            AND ts.natural_key = pss.natural_key
                            AND ts.sentence_number <= $3
                            AND te.sentence_number >= $2
                      ))
                     OR
                     (apr.book_id <> t.book_id
                      AND EXISTS (
                          SELECT 1
                          FROM sentences ps2
                          JOIN toc_nodes ptn ON ptn.id = ps2.node_id
                          JOIN page_markers ppm ON ppm.sentence_id = ps2.id
                          JOIN reference_systems prs ON prs.id = ppm.system_id
                                                    AND prs.book_id = apr.book_id
                                                    AND prs.slug = 'verse'
                          LEFT JOIN cross_translation_alignments pva
                                 ON pva.book_id = apr.book_id
                                AND pva.system_slug = 'verse'
                                AND pva.source_ref = ptn.source_ref
                                AND pva.local_ref_value = ppm.ref_value
                          JOIN target_verses tv
                                 ON tv.canonical_src = CASE
                                        WHEN pva.book_id IS NULL THEN ptn.source_ref
                                        ELSE pva.canonical_source_ref END
                                AND tv.canonical_ref = CASE
                                        WHEN pva.book_id IS NULL THEN ppm.ref_value
                                        ELSE pva.canonical_ref_value END
                          WHERE ps2.book_id = apr.book_id
                            AND ps2.sentence_number BETWEEN
                                COALESCE((
                                    SELECT MAX(s4.sentence_number)
                                    FROM sentences s4
                                    JOIN page_markers pm4 ON pm4.sentence_id = s4.id
                                    JOIN reference_systems rs4 ON rs4.id = pm4.system_id
                                                              AND rs4.book_id = apr.book_id
                                                              AND rs4.slug = 'verse'
                                    WHERE s4.book_id = apr.book_id
                                      AND s4.node_id = pss.node_id
                                      AND s4.sentence_number <= pss.sentence_number
                                ), pss.sentence_number)
                                AND COALESCE(pse.sentence_number, pss.sentence_number)
                      ))
                 )
           )
           SELECT a.id,
                  a.slug,
                  a.title,
                  u.id AS "author_user_id!",
                  u.display_name AS "author_display_name!",
                  u.handle AS "author_handle?",
                  a.published_at,
                  COUNT(*) OVER () AS "total!"
           FROM matches m
           JOIN articles a ON a.id = m.article_id
           JOIN users u ON u.id = a.user_id
           WHERE a.status IN ('published', 'archived')
           ORDER BY a.published_at DESC NULLS LAST, a.id
           LIMIT $5 OFFSET $6"#,
        book_id,
        start,
        end,
        kind as _,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map(|r| r.total).unwrap_or(0);
    let articles = rows
        .into_iter()
        .map(|r| PassageArticleResponse {
            id: r.id.to_string(),
            slug: r.slug,
            title: r.title,
            author_user_id: r.author_user_id.to_string(),
            author_display_name: r.author_display_name,
            author_handle: r.author_handle,
            published_at: r.published_at.map(fmt_time),
        })
        .collect();
    Ok((articles, total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_directive() {
        let md = r#"Intro text.

::quotation{book="kjv-bible" node="john-3" start=101 end=103 kind="body" mode="source" layout="stacked"}

Outro."#;
        assert_eq!(
            parse_quotation_directives(md),
            vec![PassageDirective {
                book_slug: "kjv-bible".into(),
                start: 101,
                end: Some(103),
                kind: "body".into(),
            }]
        );
    }

    #[test]
    fn parses_quoted_numbers_and_defaults_kind() {
        let md = r#"::quotation{book="milton" start="7" end="7"}"#;
        assert_eq!(
            parse_quotation_directives(md),
            vec![PassageDirective {
                book_slug: "milton".into(),
                start: 7,
                end: None,
                kind: "body".into(),
            }]
        );
    }

    #[test]
    fn normalizes_reversed_range() {
        let md = r#"::quotation{book="milton" start=9 end=4}"#;
        assert_eq!(
            parse_quotation_directives(md),
            vec![PassageDirective {
                book_slug: "milton".into(),
                start: 4,
                end: Some(9),
                kind: "body".into(),
            }]
        );
    }

    #[test]
    fn skips_malformed_directives() {
        let md = r#"::quotation{node="x" start=1}
::quotation{book="" start=1}
::quotation{book="a" start=oops}
::quotation{book="a" start=1 kind="banana"}
::article-quotation{id="abc"}"#;
        assert!(parse_quotation_directives(md).is_empty());
    }

    #[test]
    fn parses_footnote_kind_and_multiple_directives() {
        let md = r#"::quotation{book="kant" start=5 kind="footnote"}
Some prose.
::quotation{book="kant" start=6 end=8}"#;
        let parsed = parse_quotation_directives(md);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].kind, "footnote");
        assert_eq!(parsed[1].end, Some(8));
    }
}
