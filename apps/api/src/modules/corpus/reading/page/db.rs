use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::modules::corpus::reading::facsimile::db as facsimile;
use crate::modules::corpus::reading::nodes::db as nodes_db;
use crate::modules::corpus::reading::nodes::models::{
    ContentBlockResponse, FootnoteResponse, FootnoteSentenceResponse, NodeDetail,
    PageMarkerResponse, SentenceResponse,
};
use crate::modules::corpus::reading::page::models::NodePage;
use crate::system::error::AppError;

struct NodeRow {
    id: Uuid,
    source_ref: String,
    slug: String,
    label: String,
    depth: i16,
    sort_order: i32,
    source_node_id: Option<Uuid>,
    content_size: i32,
    original_size: i32,
}

struct BlockRow {
    id: Uuid,
    node_id: Uuid,
    position: i16,
    block_type: String,
    paragraph_number: Option<i32>,
    figure_number: Option<i32>,
    html: String,
    original_html: Option<String>,
}

struct SentenceRow {
    id: Uuid,
    block_id: Uuid,
    position: i16,
    sentence_number: Option<i32>,
    segment: Option<i16>,
    text: String,
    html: String,
    original_text: Option<String>,
    original_html: Option<String>,
    source_sentence_start_id: Option<Uuid>,
    source_sentence_end_id: Option<Uuid>,
}

struct MarkerRow {
    sentence_id: Uuid,
    system_slug: String,
    margin_prefix: Option<String>,
    ref_value: String,
    sort_order: i32,
    char_offset: Option<i32>,
    storage_key: Option<String>,
}

struct FootnoteRow {
    id: Uuid,
    number: i32,
    anchor_sentence_id: Uuid,
}

struct FootnoteSentenceRow {
    id: Uuid,
    footnote_id: Uuid,
    position: i16,
    sentence_number: Option<i32>,
    text: String,
    html: String,
    original_text: Option<String>,
    original_html: Option<String>,
}

/// Byte budget for one page of the reader's infinite scroll, in estimated
/// serialized bytes (`toc_nodes.content_size`, + `original_size` when
/// originals are requested).
const PAGE_BUDGET_BYTES: i64 = 650_000;

/// Share of an anchored window's budget spent above the anchor, so the reader
/// has somewhere to scroll up before the reverse fetch fires.
const BACK_BUDGET_BYTES: i64 = PAGE_BUDGET_BYTES / 4;

/// Node-count cap, which is what sizes pages of short-node books (Bible
/// chapters, sonnets): the 20-node window the reader's TOC navigation was tuned
/// on. A higher cap let those books fill the byte budget at ~2x the old payload.
const MAX_PAGE_NODES: usize = 20;

pub enum PageWindow<'a> {
    /// Nodes after this sort_order cursor.
    After(i32),
    /// Nodes before this sort_order cursor.
    Before(i32),
    /// A window around the node with this slug: the anchor and what follows,
    /// plus up to `BACK_BUDGET_BYTES` of the nodes above it.
    At(&'a str),
}

impl NodeRow {
    fn size(&self, include_original: bool) -> i64 {
        let original = if include_original {
            self.original_size
        } else {
            0
        };
        (self.content_size + original) as i64
    }
}

/// How many leading `sizes` fit in `budget`, at most `max_nodes`, and the bytes
/// they use. With `take_first` the first one is taken even if it alone exceeds
/// the budget, so a page never comes back empty while nodes remain.
fn fit_within(sizes: &[i64], budget: i64, max_nodes: usize, take_first: bool) -> (usize, i64) {
    let mut used = 0;
    let mut count = 0;
    for &size in sizes.iter().take(max_nodes) {
        if used + size > budget && !(take_first && count == 0) {
            break;
        }
        used += size;
        count += 1;
    }
    (count, used)
}

pub async fn get_node_page(
    pool: &PgPool,
    slug: &str,
    window: PageWindow<'_>,
    include_original: bool,
) -> Result<NodePage, AppError> {
    let sizes =
        |rows: &[NodeRow]| -> Vec<i64> { rows.iter().map(|r| r.size(include_original)).collect() };

    let (nodes, has_more, has_previous) = match window {
        PageWindow::After(after) => {
            let mut rows = nodes_after(pool, slug, after).await?;
            let (n, _) = fit_within(&sizes(&rows), PAGE_BUDGET_BYTES, MAX_PAGE_NODES, true);
            let has_more = n < rows.len();
            rows.truncate(n);
            let has_previous = match rows.first() {
                Some(first) => node_exists_before(pool, slug, first.sort_order).await?,
                None => false,
            };
            (rows, has_more, has_previous)
        }
        PageWindow::Before(before) => {
            let mut rows = nodes_before(pool, slug, before).await?;
            let (n, _) = fit_within(&sizes(&rows), PAGE_BUDGET_BYTES, MAX_PAGE_NODES, true);
            let has_previous = n < rows.len();
            rows.truncate(n);
            rows.reverse();
            let has_more = match rows.last() {
                Some(last) => node_exists_after(pool, slug, last.sort_order).await?,
                None => false,
            };
            (rows, has_more, has_previous)
        }
        PageWindow::At(node_slug) => {
            let anchor = sqlx::query_scalar!(
                r#"SELECT tn.sort_order
                   FROM toc_nodes tn
                   JOIN books b ON b.id = tn.book_id
                   WHERE b.slug = $1 AND tn.slug = $2"#,
                slug,
                node_slug,
            )
            .fetch_optional(pool)
            .await?;
            let Some(anchor) = anchor else {
                return Ok(NodePage {
                    nodes: vec![],
                    has_more: false,
                    has_previous: false,
                });
            };

            let mut above = nodes_before(pool, slug, anchor).await?;
            let (n_above, above_used) =
                fit_within(&sizes(&above), BACK_BUDGET_BYTES, MAX_PAGE_NODES - 1, false);
            let has_previous = n_above < above.len();
            above.truncate(n_above);
            above.reverse();

            let mut from_anchor = nodes_after(pool, slug, anchor - 1).await?;
            let (n_from, _) = fit_within(
                &sizes(&from_anchor),
                PAGE_BUDGET_BYTES - above_used,
                MAX_PAGE_NODES - n_above,
                true,
            );
            let has_more = n_from < from_anchor.len();
            from_anchor.truncate(n_from);

            above.extend(from_anchor);
            (above, has_more, has_previous)
        }
    };

    assemble_node_page(pool, nodes, include_original, has_more, has_previous).await
}

/// Up to `MAX_PAGE_NODES + 1` nodes after `after`, ascending — one past the cap
/// so a page that stops at the cap still reports `has_more`.
async fn nodes_after(pool: &PgPool, slug: &str, after: i32) -> Result<Vec<NodeRow>, AppError> {
    let rows = sqlx::query_as!(
        NodeRow,
        r#"SELECT tn.id, tn.source_ref, tn.slug, tn.label, tn.depth, tn.sort_order, tn.source_node_id,
                  tn.content_size, tn.original_size
           FROM toc_nodes tn
           JOIN books b ON b.id = tn.book_id
           WHERE b.slug = $1 AND tn.sort_order > $2
           ORDER BY tn.sort_order
           LIMIT $3"#,
        slug,
        after,
        (MAX_PAGE_NODES + 1) as i64,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Up to `MAX_PAGE_NODES + 1` nodes before `before`, nearest first (descending).
async fn nodes_before(pool: &PgPool, slug: &str, before: i32) -> Result<Vec<NodeRow>, AppError> {
    let rows = sqlx::query_as!(
        NodeRow,
        r#"SELECT tn.id, tn.source_ref, tn.slug, tn.label, tn.depth, tn.sort_order, tn.source_node_id,
                  tn.content_size, tn.original_size
           FROM toc_nodes tn
           JOIN books b ON b.id = tn.book_id
           WHERE b.slug = $1 AND tn.sort_order < $2
           ORDER BY tn.sort_order DESC
           LIMIT $3"#,
        slug,
        before,
        (MAX_PAGE_NODES + 1) as i64,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn node_exists_before(pool: &PgPool, slug: &str, sort_order: i32) -> Result<bool, AppError> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(
            SELECT 1 FROM toc_nodes tn
            JOIN books b ON b.id = tn.book_id
            WHERE b.slug = $1 AND tn.sort_order < $2
        ) AS "exists!""#,
        slug,
        sort_order,
    )
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

async fn node_exists_after(pool: &PgPool, slug: &str, sort_order: i32) -> Result<bool, AppError> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(
            SELECT 1 FROM toc_nodes tn
            JOIN books b ON b.id = tn.book_id
            WHERE b.slug = $1 AND tn.sort_order > $2
        ) AS "exists!""#,
        slug,
        sort_order,
    )
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

pub async fn get_nodes_by_source_ids(
    pool: &PgPool,
    slug: &str,
    source_node_ids: &[Uuid],
    include_original: bool,
) -> Result<NodePage, AppError> {
    let nodes = sqlx::query_as!(
        NodeRow,
        r#"SELECT tn.id, tn.source_ref, tn.slug, tn.label, tn.depth, tn.sort_order, tn.source_node_id,
                  tn.content_size, tn.original_size
           FROM toc_nodes tn
           JOIN books b ON b.id = tn.book_id
           WHERE b.slug = $1 AND tn.source_node_id = ANY($2)
           ORDER BY tn.sort_order"#,
        slug,
        source_node_ids,
    )
    .fetch_all(pool)
    .await?;

    assemble_node_page(pool, nodes, include_original, false, false).await
}

pub async fn get_nodes_by_ids(
    pool: &PgPool,
    slug: &str,
    ids: &[Uuid],
    include_original: bool,
) -> Result<NodePage, AppError> {
    let nodes = sqlx::query_as!(
        NodeRow,
        r#"SELECT tn.id, tn.source_ref, tn.slug, tn.label, tn.depth, tn.sort_order, tn.source_node_id,
                  tn.content_size, tn.original_size
           FROM toc_nodes tn
           JOIN books b ON b.id = tn.book_id
           WHERE b.slug = $1 AND tn.id = ANY($2)
           ORDER BY tn.sort_order"#,
        slug,
        ids,
    )
    .fetch_all(pool)
    .await?;

    assemble_node_page(pool, nodes, include_original, false, false).await
}

async fn assemble_node_page(
    pool: &PgPool,
    nodes: Vec<NodeRow>,
    include_original: bool,
    has_more: bool,
    has_previous: bool,
) -> Result<NodePage, AppError> {
    if nodes.is_empty() {
        return Ok(NodePage {
            nodes: vec![],
            has_more: false,
            has_previous: false,
        });
    }

    let node_ids: Vec<Uuid> = nodes.iter().map(|n| n.id).collect();

    let blocks = sqlx::query_as!(
        BlockRow,
        r#"SELECT id, node_id, position, block_type::TEXT AS "block_type!", paragraph_number, figure_number, html, original_html
           FROM content_blocks
           WHERE node_id = ANY($1)
           ORDER BY node_id, position"#,
        &node_ids,
    )
    .fetch_all(pool)
    .await?;

    let sentences = sqlx::query_as!(
        SentenceRow,
        r#"SELECT id, block_id AS "block_id!", position, sentence_number, segment, text, html, original_text, original_html,
                  source_sentence_start_id, source_sentence_end_id
           FROM sentences
           WHERE block_id IS NOT NULL AND block_id = ANY(
               SELECT id FROM content_blocks WHERE node_id = ANY($1)
           )
           ORDER BY block_id, position"#,
        &node_ids,
    )
    .fetch_all(pool)
    .await?;

    let markers = sqlx::query_as!(
        MarkerRow,
        r#"SELECT pm.sentence_id, rs.slug AS system_slug, rs.margin_prefix, pm.ref_value, pm.sort_order, pm.char_offset,
                  fp.storage_key AS "storage_key?"
           FROM page_markers pm
           JOIN reference_systems rs ON rs.id = pm.system_id
           JOIN sentences s ON s.id = pm.sentence_id
           LEFT JOIN facsimile_pages fp
               ON fp.reference_system_id = pm.system_id
               AND fp.ref_value = pm.ref_value
           WHERE s.node_id = ANY($1)
           ORDER BY pm.sort_order"#,
        &node_ids,
    )
    .fetch_all(pool)
    .await?;

    let sentence_ids: Vec<Uuid> = sentences.iter().map(|s| s.id).collect();
    let mut margin_note_map =
        nodes_db::margin_notes_by_anchor(pool, &sentence_ids, include_original).await?;

    let footnotes = if sentence_ids.is_empty() {
        vec![]
    } else {
        sqlx::query_as!(
            FootnoteRow,
            r#"SELECT id, number, anchor_sentence_id
               FROM footnotes
               WHERE anchor_sentence_id = ANY($1)
               ORDER BY number"#,
            &sentence_ids,
        )
        .fetch_all(pool)
        .await?
    };

    let footnote_ids: Vec<Uuid> = footnotes.iter().map(|f| f.id).collect();

    let footnote_sentences = if footnote_ids.is_empty() {
        vec![]
    } else {
        sqlx::query_as!(
            FootnoteSentenceRow,
            r#"SELECT id, footnote_id AS "footnote_id!", position, sentence_number, text, html, original_text, original_html
               FROM sentences
               WHERE footnote_id = ANY($1)
               ORDER BY footnote_id, position"#,
            &footnote_ids,
        )
        .fetch_all(pool)
        .await?
    };

    // Group footnote sentences by footnote_id
    let mut fn_sentence_map: HashMap<Uuid, Vec<FootnoteSentenceResponse>> = HashMap::new();
    for fs in footnote_sentences {
        fn_sentence_map
            .entry(fs.footnote_id)
            .or_default()
            .push(FootnoteSentenceResponse {
                id: fs.id.to_string(),
                position: fs.position,
                sentence_number: fs.sentence_number,
                text: fs.text,
                html: fs.html,
                original_text: if include_original {
                    fs.original_text
                } else {
                    None
                },
                original_html: if include_original {
                    fs.original_html
                } else {
                    None
                },
            });
    }

    // Group footnotes by anchor_sentence_id
    let mut footnote_map: HashMap<Uuid, Vec<FootnoteResponse>> = HashMap::new();
    for f in footnotes {
        footnote_map
            .entry(f.anchor_sentence_id)
            .or_default()
            .push(FootnoteResponse {
                id: f.id.to_string(),
                number: f.number,
                sentences: fn_sentence_map.remove(&f.id).unwrap_or_default(),
            });
    }

    // Group markers by sentence_id
    let mut marker_map: HashMap<Uuid, Vec<PageMarkerResponse>> = HashMap::new();
    for m in markers {
        marker_map
            .entry(m.sentence_id)
            .or_default()
            .push(PageMarkerResponse {
                system_slug: m.system_slug,
                margin_prefix: m.margin_prefix,
                ref_value: m.ref_value,
                sort_order: m.sort_order,
                char_offset: m.char_offset,
                image_url: m.storage_key.as_deref().and_then(facsimile::resolve_url),
            });
    }

    // Group sentences by block_id
    let mut sentence_map: HashMap<Uuid, Vec<SentenceResponse>> = HashMap::new();
    for s in sentences {
        sentence_map
            .entry(s.block_id)
            .or_default()
            .push(SentenceResponse {
                id: s.id.to_string(),
                position: s.position,
                sentence_number: s.sentence_number,
                figure_number: None,
                segment: s.segment,
                text: s.text,
                html: s.html,
                original_text: if include_original {
                    s.original_text
                } else {
                    None
                },
                original_html: if include_original {
                    s.original_html
                } else {
                    None
                },
                source_sentence_start_id: s.source_sentence_start_id.map(|id| id.to_string()),
                source_sentence_end_id: s.source_sentence_end_id.map(|id| id.to_string()),
                page_markers: marker_map.remove(&s.id).unwrap_or_default(),
                footnotes: footnote_map.remove(&s.id).unwrap_or_default(),
                margin_notes: margin_note_map.remove(&s.id).unwrap_or_default(),
            });
    }

    // Group blocks by node_id
    let mut block_map: HashMap<Uuid, Vec<ContentBlockResponse>> = HashMap::new();
    for b in blocks {
        let mut sentences = sentence_map.remove(&b.id).unwrap_or_default();
        // A figure carries its number on the block; copy it onto the anchor
        // sentence so the reader can key selection as `fig{N}`.
        if let Some(fig) = b.figure_number {
            for s in &mut sentences {
                s.figure_number = Some(fig);
            }
        }
        block_map
            .entry(b.node_id)
            .or_default()
            .push(ContentBlockResponse {
                id: b.id.to_string(),
                position: b.position,
                block_type: b.block_type,
                paragraph_number: b.paragraph_number,
                figure_number: b.figure_number,
                html: b.html,
                original_html: if include_original {
                    b.original_html
                } else {
                    None
                },
                sentences,
            });
    }

    let result_nodes = nodes
        .into_iter()
        .map(|n| NodeDetail {
            id: n.id.to_string(),
            source_ref: n.source_ref,
            slug: n.slug,
            label: n.label,
            depth: n.depth,
            sort_order: n.sort_order,
            source_node_id: n.source_node_id.map(|id| id.to_string()),
            blocks: block_map.remove(&n.id).unwrap_or_default(),
        })
        .collect();

    Ok(NodePage {
        nodes: result_nodes,
        has_more,
        has_previous,
    })
}

#[cfg(test)]
mod tests {
    use super::fit_within;

    #[test]
    fn fills_until_budget() {
        assert_eq!(fit_within(&[40, 40, 40], 100, 50, true), (2, 80));
    }

    #[test]
    fn stops_at_node_cap() {
        assert_eq!(fit_within(&[1; 10], 100, 3, true), (3, 3));
    }

    #[test]
    fn oversized_first_node_still_taken() {
        assert_eq!(fit_within(&[500, 10], 100, 50, true), (1, 500));
    }

    #[test]
    fn oversized_first_node_skipped_when_optional() {
        assert_eq!(fit_within(&[500, 10], 100, 50, false), (0, 0));
    }

    #[test]
    fn empty_input() {
        assert_eq!(fit_within(&[], 100, 50, true), (0, 0));
    }
}
