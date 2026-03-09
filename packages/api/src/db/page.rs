use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::node::{ContentBlockResponse, NodeDetail, PageMarkerResponse, SentenceResponse};
use crate::models::page::NodePage;

struct NodeRow {
    id: Uuid,
    source_ref: String,
    slug: String,
    label: String,
    depth: i16,
    sort_order: i32,
}

struct BlockRow {
    id: Uuid,
    node_id: Uuid,
    position: i16,
    block_type: String,
    paragraph_number: Option<i32>,
    html: String,
    original_html: Option<String>,
}

struct SentenceRow {
    id: Uuid,
    block_id: Uuid,
    position: i16,
    sentence_number: Option<i32>,
    text: String,
    html: String,
    original_text: Option<String>,
    original_html: Option<String>,
}

struct MarkerRow {
    sentence_id: Uuid,
    system_slug: String,
    ref_value: String,
    sort_order: i32,
    char_offset: Option<i32>,
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
            r#"SELECT tn.id, tn.source_ref, tn.slug, tn.label, tn.depth, tn.sort_order
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
            r#"SELECT tn.id, tn.source_ref, tn.slug, tn.label, tn.depth, tn.sort_order
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
        r#"SELECT id, node_id, position, block_type::TEXT AS "block_type!", paragraph_number, html, original_html
           FROM content_blocks
           WHERE node_id = ANY($1)
           ORDER BY node_id, position"#,
        &node_ids,
    )
    .fetch_all(pool)
    .await?;

    let sentences = sqlx::query_as!(
        SentenceRow,
        r#"SELECT id, block_id, position, sentence_number, text, html, original_text, original_html
           FROM sentences
           WHERE block_id = ANY(
               SELECT id FROM content_blocks WHERE node_id = ANY($1)
           )
           ORDER BY block_id, position"#,
        &node_ids,
    )
    .fetch_all(pool)
    .await?;

    let markers = sqlx::query_as!(
        MarkerRow,
        r#"SELECT pm.sentence_id, rs.slug AS system_slug, pm.ref_value, pm.sort_order, pm.char_offset
           FROM page_markers pm
           JOIN reference_systems rs ON rs.id = pm.system_id
           WHERE pm.sentence_id = ANY(
               SELECT s.id FROM sentences s
               JOIN content_blocks cb ON cb.id = s.block_id
               WHERE cb.node_id = ANY($1)
           )
           ORDER BY pm.sort_order"#,
        &node_ids,
    )
    .fetch_all(pool)
    .await?;

    // Group markers by sentence_id
    let mut marker_map: std::collections::HashMap<Uuid, Vec<PageMarkerResponse>> =
        std::collections::HashMap::new();
    for m in markers {
        marker_map.entry(m.sentence_id).or_default().push(
            PageMarkerResponse {
                system_slug: m.system_slug,
                ref_value: m.ref_value,
                sort_order: m.sort_order,
                char_offset: m.char_offset,
            },
        );
    }

    // Group sentences by block_id
    let mut sentence_map: std::collections::HashMap<Uuid, Vec<SentenceResponse>> =
        std::collections::HashMap::new();
    for s in sentences {
        sentence_map
            .entry(s.block_id)
            .or_default()
            .push(SentenceResponse {
                id: s.id.to_string(),
                position: s.position,
                sentence_number: s.sentence_number,
                text: s.text,
                html: s.html,
                original_text: if include_original { s.original_text } else { None },
                original_html: if include_original { s.original_html } else { None },
                page_markers: marker_map.remove(&s.id).unwrap_or_default(),
            });
    }

    // Group blocks by node_id
    let mut block_map: std::collections::HashMap<Uuid, Vec<ContentBlockResponse>> =
        std::collections::HashMap::new();
    for b in blocks {
        let sentences = sentence_map.remove(&b.id).unwrap_or_default();
        block_map
            .entry(b.node_id)
            .or_default()
            .push(ContentBlockResponse {
                id: b.id.to_string(),
                position: b.position,
                block_type: b.block_type,
                paragraph_number: b.paragraph_number,
                html: b.html,
                original_html: if include_original { b.original_html } else { None },
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
            blocks: block_map.remove(&n.id).unwrap_or_default(),
        })
        .collect();

    Ok(NodePage {
        nodes: result_nodes,
        has_more,
        has_previous,
    })
}
