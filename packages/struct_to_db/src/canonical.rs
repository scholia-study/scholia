//! Canonical-passage seeding for sentence-locked works (ADR 0008).
//!
//! The root edition (the source book of a translation run, or the book
//! itself otherwise) mints one `canonical_passages` row per numbered
//! sentence and stamps its own rows; a translation then copies each
//! linked source sentence's passage through `source_sentence_start_id`.
//! Every statement is set-based and idempotent, so the pass runs
//! unchanged after both the fresh-insert and reconcile paths — and a
//! translation import re-mints the source book first, making corpus
//! import order irrelevant.

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub struct CanonicalReport {
    pub minted: u64,
    pub stamped: u64,
}

pub async fn seed_sentence_basis(
    tx: &mut Transaction<'_, Postgres>,
    book_id: Uuid,
    source_book_id: Option<Uuid>,
) -> Result<CanonicalReport, sqlx::Error> {
    let root_book = source_book_id.unwrap_or(book_id);

    // A fresh import bulk-loads sentences earlier in this transaction, so
    // the planner has no statistics for them yet — ANALYZE first or the
    // stamp joins degrade to nested-loop seq scans.
    sqlx::query("ANALYZE sentences").execute(&mut **tx).await?;

    // Mint passages for root sentences that lack one. Figure sentences sit
    // outside the sentence enumeration (sentence_number NULL) and are
    // excluded — figure-anchored resources stay edition-local.
    let minted = sqlx::query(
        r#"WITH wr AS (
               SELECT COALESCE(s.translation_of_id, s.id) AS root
               FROM books b JOIN sources s ON s.id = b.source_id
               WHERE b.id = $1
           )
           INSERT INTO canonical_passages
               (work_root, basis, sentence_kind, ordinal, root_sentence_id)
           SELECT wr.root, 'sentence',
                  CASE WHEN se.footnote_id IS NOT NULL THEN 'footnote'
                       ELSE 'body' END::sentence_kind,
                  se.sentence_number, se.id
           FROM sentences se CROSS JOIN wr
           WHERE se.book_id = $1 AND se.sentence_number IS NOT NULL
           ON CONFLICT (root_sentence_id) WHERE basis = 'sentence' DO NOTHING"#,
    )
    .bind(root_book)
    .execute(&mut **tx)
    .await?
    .rows_affected();

    // canonical_passages may have just gained the whole book's rows.
    sqlx::query("ANALYZE canonical_passages")
        .execute(&mut **tx)
        .await?;

    // Reconcile renumbering moves sentence_number; keep ordinals honest.
    // (Deleted root sentences cascade their passage away via the FK.)
    sqlx::query(
        r#"UPDATE canonical_passages cp
           SET ordinal = se.sentence_number
           FROM sentences se
           WHERE cp.root_sentence_id = se.id
             AND se.book_id = $1
             AND se.sentence_number IS NOT NULL
             AND cp.ordinal <> se.sentence_number"#,
    )
    .bind(root_book)
    .execute(&mut **tx)
    .await?;

    // Stamp the root edition's own sentences.
    let mut stamped = sqlx::query(
        r#"UPDATE sentences se
           SET canonical_passage_id = cp.id
           FROM canonical_passages cp
           WHERE cp.root_sentence_id = se.id
             AND se.book_id = $1
             AND se.canonical_passage_id IS DISTINCT FROM cp.id"#,
    )
    .bind(root_book)
    .execute(&mut **tx)
    .await?
    .rows_affected();

    if source_book_id.is_some() {
        // Translation: copy the linked source sentence's passage, and clear
        // stamps whose 1:1 link is gone.
        stamped += sqlx::query(
            r#"UPDATE sentences t
               SET canonical_passage_id = src.canonical_passage_id
               FROM sentences src
               WHERE src.id = t.source_sentence_start_id
                 AND t.book_id = $1
                 AND t.canonical_passage_id IS DISTINCT FROM src.canonical_passage_id"#,
        )
        .bind(book_id)
        .execute(&mut **tx)
        .await?
        .rows_affected();

        sqlx::query(
            r#"UPDATE sentences
               SET canonical_passage_id = NULL
               WHERE book_id = $1
                 AND source_sentence_start_id IS NULL
                 AND canonical_passage_id IS NOT NULL"#,
        )
        .bind(book_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(CanonicalReport { minted, stamped })
}
