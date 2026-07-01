-- Index the unindexed foreign-key columns that REFERENCE sentences(id), so
-- deleting a book (cascade / SET NULL) doesn't sequential-scan the whole
-- sentences table once per deleted row.

CREATE INDEX IF NOT EXISTS idx_sentences_source_end
    ON sentences (source_sentence_end_id)
    WHERE source_sentence_end_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_xref_source_sentence_end
    ON cross_references (source_sentence_end_id);
CREATE INDEX IF NOT EXISTS idx_xref_target_sentence_end
    ON cross_references (target_sentence_end_id);

CREATE INDEX IF NOT EXISTS idx_quotations_anchor_start
    ON quotations (anchor_sentence_start_id);
CREATE INDEX IF NOT EXISTS idx_quotations_anchor_end
    ON quotations (anchor_sentence_end_id);
