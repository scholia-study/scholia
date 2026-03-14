use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::node::{
    ContentBlockResponse, FootnoteResponse, FootnoteSentenceResponse, NodeDetail,
    PageMarkerResponse, SentenceResponse,
};

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

struct FootnoteRow {
    id: Uuid,
    number: i32,
    anchor_sentence_id: Uuid,
}

struct FootnoteSentenceRow {
    id: Uuid,
    footnote_id: Uuid,
    position: i16,
    text: String,
    html: String,
    original_text: Option<String>,
    original_html: Option<String>,
}

pub async fn get_node_content(
    pool: &PgPool,
    book_slug: &str,
    node_slug: &str,
    include_original: bool,
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
        r#"SELECT id, block_id AS "block_id!", position, sentence_number, text, html, original_text, original_html
           FROM sentences
           WHERE node_id = $1 AND block_id IS NOT NULL
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

    // Fetch footnotes anchored to sentences in this node
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
            r#"SELECT id, footnote_id AS "footnote_id!", position, text, html, original_text, original_html
               FROM sentences
               WHERE footnote_id = ANY($1)
               ORDER BY footnote_id, position"#,
            &footnote_ids,
        )
        .fetch_all(pool)
        .await?
    };

    // Group footnote sentences by footnote_id
    let mut fn_sentence_map: std::collections::HashMap<Uuid, Vec<FootnoteSentenceResponse>> =
        std::collections::HashMap::new();
    for fs in footnote_sentences {
        fn_sentence_map
            .entry(fs.footnote_id)
            .or_default()
            .push(FootnoteSentenceResponse {
                id: fs.id.to_string(),
                position: fs.position,
                text: fs.text,
                html: fs.html,
                original_text: if include_original { fs.original_text } else { None },
                original_html: if include_original { fs.original_html } else { None },
            });
    }

    // Group footnotes by anchor_sentence_id
    let mut footnote_map: std::collections::HashMap<Uuid, Vec<FootnoteResponse>> =
        std::collections::HashMap::new();
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
                footnotes: footnote_map.remove(&s.id).unwrap_or_default(),
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
                original_html: if include_original { b.original_html } else { None },
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
