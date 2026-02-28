use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::node::{ContentBlockResponse, NodeDetail, SentenceResponse};

struct NodeRow {
    id: Uuid,
    ncx_id: String,
    label: String,
    depth: i16,
    play_order: i32,
}

struct BlockRow {
    id: Uuid,
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

pub async fn get_node_content(
    pool: &PgPool,
    slug: &str,
    ncx_id: &str,
) -> Result<NodeDetail, AppError> {
    let node = sqlx::query_as!(
        NodeRow,
        r#"SELECT tn.id, tn.ncx_id, tn.label, tn.depth, tn.play_order
           FROM toc_nodes tn
           JOIN books b ON b.id = tn.book_id
           WHERE b.slug = $1 AND tn.ncx_id = $2"#,
        slug,
        ncx_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Node not found: {ncx_id}")))?;

    let blocks = sqlx::query_as!(
        BlockRow,
        r#"SELECT id, position, block_type::TEXT AS "block_type!", paragraph_number, html
           FROM content_blocks
           WHERE node_id = $1
           ORDER BY position"#,
        node.id,
    )
    .fetch_all(pool)
    .await?;

    let sentences = sqlx::query_as!(
        SentenceRow,
        r#"SELECT id, block_id, position, sentence_number, text, html
           FROM sentences
           WHERE node_id = $1
           ORDER BY block_id, position"#,
        node.id,
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
                sentences,
            }
        })
        .collect();

    Ok(NodeDetail {
        id: node.id.to_string(),
        ncx_id: node.ncx_id,
        label: node.label,
        depth: node.depth,
        play_order: node.play_order,
        blocks,
    })
}
