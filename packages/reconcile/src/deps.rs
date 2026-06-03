//! Keeping dependents attached to the right sentence when reconciliation merges
//! one away or splits one in two. These touch every table that references
//! `sentences.id`, so the logic lives in exactly one place to avoid drift.

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

/// Does this sentence have user/editor data (or a footnote anchor) hanging off
/// it? Used to refuse a destructive delete that would orphan real data.
pub async fn sentence_has_dependents(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<bool, Box<dyn std::error::Error>> {
    let count: i64 = sqlx::query_scalar(
        "SELECT
           (SELECT count(*) FROM quotations WHERE anchor_sentence_start_id = $1 OR anchor_sentence_end_id = $1)
         + (SELECT count(*) FROM resources  WHERE anchor_sentence_start_id = $1 OR anchor_sentence_end_id = $1)
         + (SELECT count(*) FROM cross_references WHERE source_sentence_start_id = $1 OR source_sentence_end_id = $1 OR target_sentence_start_id = $1 OR target_sentence_end_id = $1)
         + (SELECT count(*) FROM footnotes WHERE anchor_sentence_id = $1)",
    )
    .bind(id)
    .fetch_one(&mut **tx)
    .await?;
    Ok(count > 0)
}

/// Repoint every reference from `from` onto `to` (merge survivor). Returns the
/// number of rows moved.
pub async fn migrate_dependents(
    tx: &mut Transaction<'_, Postgres>,
    from: Uuid,
    to: Uuid,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut moved = 0u32;
    let stmts = [
        "UPDATE quotations SET anchor_sentence_start_id = $2 WHERE anchor_sentence_start_id = $1",
        "UPDATE quotations SET anchor_sentence_end_id = $2 WHERE anchor_sentence_end_id = $1",
        "UPDATE resources SET anchor_sentence_start_id = $2 WHERE anchor_sentence_start_id = $1",
        "UPDATE resources SET anchor_sentence_end_id = $2 WHERE anchor_sentence_end_id = $1",
        "UPDATE cross_references SET source_sentence_start_id = $2 WHERE source_sentence_start_id = $1",
        "UPDATE cross_references SET source_sentence_end_id = $2 WHERE source_sentence_end_id = $1",
        "UPDATE cross_references SET target_sentence_start_id = $2 WHERE target_sentence_start_id = $1",
        "UPDATE cross_references SET target_sentence_end_id = $2 WHERE target_sentence_end_id = $1",
        "UPDATE footnotes SET anchor_sentence_id = $2 WHERE anchor_sentence_id = $1",
        // translation links from the peer book
        "UPDATE sentences SET source_sentence_start_id = $2 WHERE source_sentence_start_id = $1",
        "UPDATE sentences SET source_sentence_end_id = $2 WHERE source_sentence_end_id = $1",
    ];
    for sql in stmts {
        let res = sqlx::query(sql)
            .bind(from)
            .bind(to)
            .execute(&mut **tx)
            .await?;
        moved += res.rows_affected() as u32;
    }
    Ok(moved)
}

/// After a split, a single-sentence anchor on the first half (`end IS NULL`)
/// should grow to cover the new second half so the quoted text is preserved.
pub async fn extend_anchors_to(
    tx: &mut Transaction<'_, Postgres>,
    first_half: Uuid,
    second_half: Uuid,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut moved = 0u32;
    let stmts = [
        "UPDATE quotations SET anchor_sentence_end_id = $2 WHERE anchor_sentence_start_id = $1 AND anchor_sentence_end_id IS NULL",
        "UPDATE resources SET anchor_sentence_end_id = $2 WHERE anchor_sentence_start_id = $1 AND anchor_sentence_end_id IS NULL",
        "UPDATE cross_references SET source_sentence_end_id = $2 WHERE source_sentence_start_id = $1 AND source_sentence_end_id IS NULL",
        "UPDATE cross_references SET target_sentence_end_id = $2 WHERE target_sentence_start_id = $1 AND target_sentence_end_id IS NULL",
    ];
    for sql in stmts {
        let res = sqlx::query(sql)
            .bind(first_half)
            .bind(second_half)
            .execute(&mut **tx)
            .await?;
        moved += res.rows_affected() as u32;
    }
    Ok(moved)
}
