use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::toc::TocNodeResponse;

struct TocRow {
    id: Uuid,
    parent_id: Option<Uuid>,
    ncx_id: String,
    slug: String,
    label: String,
    depth: i16,
    play_order: i32,
    has_content: bool,
}

pub async fn get_toc_tree(pool: &PgPool, slug: &str) -> Result<Vec<TocNodeResponse>, AppError> {
    let rows = sqlx::query_as!(
        TocRow,
        r#"SELECT
               tn.id,
               tn.parent_id,
               tn.ncx_id,
               tn.slug,
               tn.label,
               tn.depth,
               tn.play_order,
               EXISTS(SELECT 1 FROM content_blocks cb WHERE cb.node_id = tn.id) AS "has_content!"
           FROM toc_nodes tn
           JOIN books b ON b.id = tn.book_id
           WHERE b.slug = $1
           ORDER BY tn.play_order"#,
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
    // Two-pass approach: first create all nodes, then assemble into tree.
    let mut nodes: HashMap<Uuid, TocNodeResponse> = HashMap::new();
    let mut order: Vec<(Uuid, Option<Uuid>)> = Vec::new();

    for row in rows {
        order.push((row.id, row.parent_id));
        nodes.insert(
            row.id,
            TocNodeResponse {
                id: row.id.to_string(),
                ncx_id: row.ncx_id,
                slug: row.slug,
                label: row.label,
                depth: row.depth,
                play_order: row.play_order,
                has_content: row.has_content,
                children: Vec::new(),
            },
        );
    }

    // Process in reverse order so children are removed before parents.
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

    // We processed in reverse, so reverse to restore play_order.
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
