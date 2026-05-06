use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::toc::TocNodeResponse;

struct TocRow {
    id: Uuid,
    parent_id: Option<Uuid>,
    source_ref: String,
    slug: String,
    label: String,
    label_html: Option<String>,
    depth: i16,
    sort_order: i32,
    has_content: bool,
    source_node_id: Option<Uuid>,
    source_id: Option<Uuid>,
}

pub async fn get_toc_tree(pool: &PgPool, slug: &str) -> Result<Vec<TocNodeResponse>, AppError> {
    let rows = sqlx::query_as!(
        TocRow,
        r#"SELECT
               tn.id,
               tn.parent_id,
               tn.source_ref,
               tn.slug,
               tn.label,
               tn.label_html,
               tn.depth,
               tn.sort_order,
               EXISTS(SELECT 1 FROM content_blocks cb WHERE cb.node_id = tn.id) AS "has_content!",
               tn.source_node_id,
               tn.source_id
           FROM toc_nodes tn
           JOIN books b ON b.id = tn.book_id
           WHERE b.slug = $1
           ORDER BY tn.sort_order"#,
        slug,
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Err(AppError::NotFound(format!("Book not found: {slug}")));
    }

    Ok(build_tree(rows))
}

fn build_tree(rows: Vec<TocRow>) -> Vec<TocNodeResponse> {
    let mut nodes: HashMap<Uuid, TocNodeResponse> = HashMap::new();
    let mut order: Vec<(Uuid, Option<Uuid>)> = Vec::new();

    for row in rows {
        order.push((row.id, row.parent_id));
        nodes.insert(
            row.id,
            TocNodeResponse {
                id: row.id.to_string(),
                source_ref: row.source_ref,
                slug: row.slug,
                label: row.label,
                label_html: row.label_html,
                depth: row.depth,
                sort_order: row.sort_order,
                has_content: row.has_content,
                source_node_id: row.source_node_id.map(|id| id.to_string()),
                source_id: row.source_id.map(|id| id.to_string()),
                children: Vec::new(),
            },
        );
    }

    let mut roots = Vec::new();
    for (id, parent_id) in order.iter().rev() {
        let node = nodes.remove(id).unwrap();
        match parent_id {
            Some(pid) => {
                if let Some(parent) = nodes.get_mut(pid) {
                    parent.children.push(node);
                } else {
                    roots.push(node);
                }
            }
            None => roots.push(node),
        }
    }

    roots.reverse();
    reverse_children(&mut roots);
    roots
}

fn reverse_children(nodes: &mut [TocNodeResponse]) {
    for node in nodes.iter_mut() {
        node.children.reverse();
        reverse_children(&mut node.children);
    }
}
