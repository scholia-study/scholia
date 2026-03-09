use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::node::{ContentBlockResponse, NodeDetail, PageMarkerResponse, SentenceResponse};

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

pub async fn get_node_content(
    pool: &PgPool,
    book_slug: &str,
    node_slug: &str,
) -> Result<NodeDetail, AppError> {
    let node = sqlx::query_as!(
        NodeRow,
        r#"SELECT tn.id, tn.source_ref, tn.slug, tn.label, tn.depth, tn.sort_order
           FROM toc_nodes tn
           JOIN books b ON b.id = tn.book_id
           WHERE b.slug = $1 AND tn.slug = $2"#,
        book_slug,
        node_slug,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Node not found: {node_slug}")))?;

    let blocks = sqlx::query_as!(
        BlockRow,
        r#"SELECT id, position, block_type::TEXT AS "block_type!", paragraph_number, html, original_html
           FROM content_blocks
           WHERE node_id = $1
           ORDER BY position"#,
        node.id,
    )
    .fetch_all(pool)
    .await?;

    let sentences = sqlx::query_as!(
        SentenceRow,
        r#"SELECT id, block_id, position, sentence_number, text, html, original_text, original_html
           FROM sentences
           WHERE node_id = $1
           ORDER BY block_id, position"#,
        node.id,
    )
    .fetch_all(pool)
    .await?;

    let markers = sqlx::query_as!(
        MarkerRow,
        r#"SELECT pm.sentence_id, rs.slug AS system_slug, pm.ref_value, pm.sort_order, pm.char_offset
           FROM page_markers pm
           JOIN reference_systems rs ON rs.id = pm.system_id
           JOIN sentences s ON s.id = pm.sentence_id
           WHERE s.node_id = $1
           ORDER BY pm.sort_order"#,
        node.id,
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
                original_text: s.original_text,
                original_html: s.original_html,
                page_markers: marker_map.remove(&s.id).unwrap_or_default(),
            });
    }

    let blocks = blocks
        .into_iter()
        .map(|b| {
            let sentences = sentence_map.remove(&b.id).unwrap_or_default();
            ContentBlockResponse {
                id: b.id.to_string(),
                position: b.position,
                block_type: b.block_type,
                paragraph_number: b.paragraph_number,
                html: b.html,
                original_html: b.original_html,
                sentences,
            }
        })
        .collect();

    Ok(NodeDetail {
        id: node.id.to_string(),
        source_ref: node.source_ref,
        slug: node.slug,
        label: node.label,
        depth: node.depth,
        sort_order: node.sort_order,
        blocks,
    })
}
