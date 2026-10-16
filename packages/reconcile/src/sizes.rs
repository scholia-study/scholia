use sqlx::{Postgres, Transaction};
use uuid::Uuid;

/// Recompute `toc_nodes.content_size` / `original_size` for a whole book —
/// the reader's page window is byte-budgeted on them. Call at the end of every
/// import transaction that writes content; the definition lives in the
/// `recompute_node_sizes` SQL function (migration 0032).
pub async fn recompute_node_sizes(
    tx: &mut Transaction<'_, Postgres>,
    book_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT recompute_node_sizes($1)")
        .bind(book_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
