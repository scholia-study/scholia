use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::node::{ContentBlockResponse, NodeDetail, SentenceResponse};
use crate::models::page::NodePage;

struct NodeRow {
    id: Uuid,
    ncx_id: String,
    label: String,
    depth: i16,
    play_order: i32,
}

struct BlockRow {
    id: Uuid,
    node_id: Uuid,
    position: i16,
    block_type: String,
    paragraph_number: Option<i32>,
    html: String,
}

struct SentenceRow {
    id: Uuid,
    block_id: Uuid,
    position: i16,
    sentence_number: i32,
    text: String,
    html: String,
}

pub async fn get_node_page(
    pool: &PgPool,
    slug: &str,
    after: Option<i32>,
    limit: i32,
) -> Result<NodePage, AppError> {
    let after = after.unwrap_or(-1);

    // Fetch limit+1 nodes to determine has_more
    let fetch_limit = (limit + 1) as i64;
    let nodes = sqlx::query_as!(
        NodeRow,
        r#"SELECT tn.id, tn.ncx_id, tn.label, tn.depth, tn.play_order
           FROM toc_nodes tn
           JOIN books b ON b.id = tn.book_id
           WHERE b.slug = $1 AND tn.play_order > $2
           ORDER BY tn.play_order
           LIMIT $3"#,
        slug,
        after,
        fetch_limit,
    )
    .fetch_all(pool)
    .await?;

    let has_more = nodes.len() as i64 > limit as i64;
    let nodes: Vec<NodeRow> = nodes.into_iter().take(limit as usize).collect();

    if nodes.is_empty() {
        return Ok(NodePage {
            nodes: vec![],
            has_more: false,
        });
    }

    let node_ids: Vec<Uuid> = nodes.iter().map(|n| n.id).collect();

    // Fetch all blocks for these nodes
    let blocks = sqlx::query_as!(
        BlockRow,
        r#"SELECT id, node_id, position, block_type::TEXT AS "block_type!", paragraph_number, html
           FROM content_blocks
           WHERE node_id = ANY($1)
           ORDER BY node_id, position"#,
        &node_ids,
    )
    .fetch_all(pool)
    .await?;

    // Fetch all sentences for these nodes
    let sentences = sqlx::query_as!(
        SentenceRow,
        r#"SELECT id, block_id, position, sentence_number, text, html
           FROM sentences
           WHERE block_id = ANY(
               SELECT id FROM content_blocks WHERE node_id = ANY($1)
           )
           ORDER BY block_id, position"#,
        &node_ids,
    )
    .fetch_all(pool)
    .await?;

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
                sentences,
            });
    }

    // Assemble NodeDetail list in original order
    let result_nodes = nodes
        .into_iter()
        .map(|n| NodeDetail {
            id: n.id.to_string(),
            ncx_id: n.ncx_id,
            label: n.label,
            depth: n.depth,
            play_order: n.play_order,
            blocks: block_map.remove(&n.id).unwrap_or_default(),
        })
        .collect();

    Ok(NodePage {
        nodes: result_nodes,
        has_more,
    })
}
