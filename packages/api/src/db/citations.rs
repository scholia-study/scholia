use sqlx::PgPool;
use uuid::Uuid;

/// Resolve the effective bibliographic source for an anchor in a hosted text.
///
/// Walks ancestors of `anchor_node_id` looking for the deepest non-null
/// `toc_nodes.source_id`. If none of the ancestors carry one (the common
/// case — most works are not compilations), falls back to the book's
/// root `books.source_id`.
///
/// Used at quotation-create time to denormalize `quotations.source_id`,
/// avoiding per-row ancestor walks at read time.
pub async fn resolve_effective_source(
    pool: &PgPool,
    book_id: Uuid,
    anchor_node_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    // Single LTREE-indexed query: find the deepest ancestor of the anchor
    // (within the same book) that carries a non-null source_id.
    let nested: Option<Uuid> = sqlx::query_scalar!(
        r#"
        WITH target AS (
            SELECT path FROM toc_nodes WHERE id = $2 AND book_id = $1
        )
        SELECT anc.source_id AS "source_id!"
        FROM toc_nodes anc, target
        WHERE anc.book_id = $1
          AND anc.path @> target.path
          AND anc.source_id IS NOT NULL
        ORDER BY anc.depth DESC
        LIMIT 1
        "#,
        book_id,
        anchor_node_id,
    )
    .fetch_optional(pool)
    .await?;

    if let Some(source_id) = nested {
        return Ok(source_id);
    }

    // Fall back to the hosted text's root source.
    let root: Uuid = sqlx::query_scalar!(r#"SELECT source_id FROM books WHERE id = $1"#, book_id,)
        .fetch_one(pool)
        .await?;

    Ok(root)
}
