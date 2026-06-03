-- Stable, structure-derived identity for a sentence, used by the
-- reconciling re-import (`kant1_struct_to_db`) to carry UUIDs — and the
-- quotations/resources/cross-references anchored to them — across edits
-- to the curated markdown.
--
-- Shape (block-scoped, TOC-tied):
--   block sentence:    "{node.source_ref}/b{block_position}/s{sentence_position}"
--   footnote sentence: "{node.source_ref}/fn{footnote_number}/s{sentence_position}"
--
-- The block is the unit of stability: a sentence split/merge leaves the
-- block count untouched and only reshuffles ordinals inside the one
-- affected paragraph, so every other sentence keeps an identical key. The
-- column is denormalized from structure and rewritten on every import; the
-- partial unique index guarantees no two sentences claim the same coordinate.
-- Nullable + partial index so the migration applies cleanly to pre-existing
-- rows (which get backfilled on the next import).

ALTER TABLE sentences ADD COLUMN natural_key TEXT;

CREATE UNIQUE INDEX idx_sentences_natural_key
    ON sentences (book_id, natural_key)
    WHERE natural_key IS NOT NULL;
