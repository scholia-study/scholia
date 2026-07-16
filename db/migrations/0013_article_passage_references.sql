-- Derived index of ::quotation{book=… node=… start=N end=N kind=…}
-- directives embedded in article markdown, resolved to sentence UUIDs at
-- sync time (mirroring the quotations anchor shape). Rebuildable at any
-- time from articles.markdown; rows are synced on every article markdown
-- save and on publish. Drafts may have rows; the read path filters on
-- articles.status IN ('published', 'archived').

CREATE TABLE article_passage_references (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id                UUID NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    book_id                   UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    anchor_node_id            UUID NOT NULL REFERENCES toc_nodes(id) ON DELETE CASCADE,
    anchor_sentence_start_id  UUID NOT NULL REFERENCES sentences(id) ON DELETE CASCADE,
    anchor_sentence_end_id    UUID REFERENCES sentences(id) ON DELETE CASCADE,
    sentence_kind             sentence_kind NOT NULL DEFAULT 'body',
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- One row per (article, quoted range). COALESCE maps the nullable end id
-- to a sentinel so single-sentence ranges dedupe too (NULLs are distinct
-- in unique indexes); same trick as idx_quotations_user_range.
CREATE UNIQUE INDEX idx_article_passage_refs_article_range
    ON article_passage_references (article_id, anchor_sentence_start_id,
        COALESCE(anchor_sentence_end_id, '00000000-0000-0000-0000-000000000000'));

CREATE INDEX idx_article_passage_refs_book
    ON article_passage_references (book_id, sentence_kind);

-- FK-end indexes so sentence deletes during reconcile don't seq-scan
-- (parity with 0011_index_sentence_fk_ends.sql).
CREATE INDEX idx_article_passage_refs_anchor_start
    ON article_passage_references (anchor_sentence_start_id);
CREATE INDEX idx_article_passage_refs_anchor_end
    ON article_passage_references (anchor_sentence_end_id);
