use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::modules::corpus::reading::facsimile::db as facsimile;
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

pub async fn get_node_page(
    pool: &PgPool,
    slug: &str,
    after: Option<i32>,
    before: Option<i32>,
    limit: i32,
    include_original: bool,
) -> Result<NodePage, AppError> {
    let fetch_limit = (limit + 1) as i64;

    // `before` takes precedence: fetch nodes *before* the cursor (reverse order)
    let (nodes, is_backward) = if let Some(before_cursor) = before {
        let rows = sqlx::query_as!(
            NodeRow,
            r#"SELECT tn.id, tn.source_ref, tn.slug, tn.label, tn.depth, tn.sort_order, tn.source_node_id
               FROM toc_nodes tn
               JOIN books b ON b.id = tn.book_id
               WHERE b.slug = $1 AND tn.sort_order < $2
               ORDER BY tn.sort_order DESC
               LIMIT $3"#,
            slug,
            before_cursor,
            fetch_limit,
        )
        .fetch_all(pool)
        .await?;
        (rows, true)
    } else {
        let after_cursor = after.unwrap_or(-1);
        let rows = sqlx::query_as!(
            NodeRow,
            r#"SELECT tn.id, tn.source_ref, tn.slug, tn.label, tn.depth, tn.sort_order, tn.source_node_id
               FROM toc_nodes tn
               JOIN books b ON b.id = tn.book_id
               WHERE b.slug = $1 AND tn.sort_order > $2
               ORDER BY tn.sort_order
               LIMIT $3"#,
            slug,
            after_cursor,
            fetch_limit,
        )
        .fetch_all(pool)
        .await?;
        (rows, false)
    };

    let has_extra = nodes.len() as i64 > limit as i64;
    let mut nodes: Vec<NodeRow> = nodes.into_iter().take(limit as usize).collect();

    // For backward fetch, results came in DESC order — reverse to ASC
    if is_backward {
        nodes.reverse();
    }

    let (has_more, has_previous) = if is_backward {
        // backward page: has_extra means there are earlier nodes; check if there are later nodes
        let has_later = if nodes.is_empty() {
            false
        } else {
            let last_sort = nodes.last().unwrap().sort_order;
            sqlx::query_scalar!(
                r#"SELECT EXISTS(
                    SELECT 1 FROM toc_nodes tn
                    JOIN books b ON b.id = tn.book_id
                    WHERE b.slug = $1 AND tn.sort_order > $2
                ) AS "exists!""#,
                slug,
                last_sort,
            )
            .fetch_one(pool)
            .await?
        };
        (has_later, has_extra)
    } else {
        // forward page: has_extra means there are later nodes; check if there are earlier nodes
        let has_earlier = if nodes.is_empty() {
            false
        } else {
            let first_sort = nodes.first().unwrap().sort_order;
            first_sort > 0
        };
        (has_extra, has_earlier)
    };

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
        r#"SELECT pm.sentence_id, rs.slug AS system_slug, pm.ref_value, pm.sort_order, pm.char_offset,
                  fp.storage_key AS "storage_key?"
           FROM page_markers pm
           JOIN reference_systems rs ON rs.id = pm.system_id
           LEFT JOIN facsimile_pages fp
               ON fp.reference_system_id = pm.system_id
               AND fp.ref_value = pm.ref_value
           WHERE pm.sentence_id = ANY(
               SELECT s.id FROM sentences s
               WHERE s.block_id IS NOT NULL AND s.block_id = ANY(
                   SELECT id FROM content_blocks WHERE node_id = ANY($1)
               )
           )
           ORDER BY pm.sort_order"#,
        &node_ids,
    )
    .fetch_all(pool)
    .await?;

    // Fetch footnotes anchored to sentences in these nodes
    let anchor_sentence_ids: Vec<Uuid> = sentences.iter().map(|s| s.id).collect();

    let footnotes = if anchor_sentence_ids.is_empty() {
        vec![]
    } else {
        sqlx::query_as!(
            FootnoteRow,
            r#"SELECT id, number, anchor_sentence_id
               FROM footnotes
               WHERE anchor_sentence_id = ANY($1)
               ORDER BY number"#,
            &anchor_sentence_ids,
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

/// Resolve a node slug to its sort_order, then return a forward window that
/// starts `back` positions before the anchor (inclusive). Used by the route
/// loader so it can prefetch the chapter window without a separate TOC fetch.
pub async fn get_node_page_at(
    pool: &PgPool,
    book_slug: &str,
    node_slug: &str,
    back: i32,
    limit: i32,
    include_original: bool,
) -> Result<NodePage, AppError> {
    let row = sqlx::query!(
        r#"SELECT tn.sort_order
           FROM toc_nodes tn
           JOIN books b ON b.id = tn.book_id
           WHERE b.slug = $1 AND tn.slug = $2"#,
        book_slug,
        node_slug,
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(NodePage {
            nodes: vec![],
            has_more: false,
            has_previous: false,
        });
    };

    // The SQL predicate is `sort_order > after`, so to include the anchor
    // (sort_order S) plus `back` nodes before it, the cursor must be
    // S - 1 - back. Clamp to -1, not 0: -1 is the sentinel meaning "before
    // node 0" (`sort_order > -1` includes the first node), so an anchor at
    // sort_order 0 stays in its own window.
    let after = (row.sort_order - 1 - back).max(-1);
    get_node_page(pool, book_slug, Some(after), None, limit, include_original).await
}

pub async fn get_nodes_by_source_ids(
    pool: &PgPool,
    slug: &str,
    source_node_ids: &[Uuid],
    include_original: bool,
) -> Result<NodePage, AppError> {
    let nodes = sqlx::query_as!(
        NodeRow,
        r#"SELECT tn.id, tn.source_ref, tn.slug, tn.label, tn.depth, tn.sort_order, tn.source_node_id
           FROM toc_nodes tn
           JOIN books b ON b.id = tn.book_id
           WHERE b.slug = $1 AND tn.source_node_id = ANY($2)
           ORDER BY tn.sort_order"#,
        slug,
        source_node_ids,
    )
    .fetch_all(pool)
    .await?;

    assemble_node_page(pool, nodes, include_original).await
}

pub async fn get_nodes_by_ids(
    pool: &PgPool,
    slug: &str,
    ids: &[Uuid],
    include_original: bool,
) -> Result<NodePage, AppError> {
    let nodes = sqlx::query_as!(
        NodeRow,
        r#"SELECT tn.id, tn.source_ref, tn.slug, tn.label, tn.depth, tn.sort_order, tn.source_node_id
           FROM toc_nodes tn
           JOIN books b ON b.id = tn.book_id
           WHERE b.slug = $1 AND tn.id = ANY($2)
           ORDER BY tn.sort_order"#,
        slug,
        ids,
    )
    .fetch_all(pool)
    .await?;

    assemble_node_page(pool, nodes, include_original).await
}

async fn assemble_node_page(
    pool: &PgPool,
    nodes: Vec<NodeRow>,
    include_original: bool,
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
        r#"SELECT pm.sentence_id, rs.slug AS system_slug, pm.ref_value, pm.sort_order, pm.char_offset,
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
        has_more: false,
        has_previous: false,
    })
}
