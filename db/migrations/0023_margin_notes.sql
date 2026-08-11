-- Margin notes: authorial annotations anchored to a sentence and rendered
-- in the reader's gutter.
-- Distinct from footnotes (no in-text marker glyph; `*` stays reserved for
-- footnotes) but structurally their mirror: identity is a book-global
-- document-order number, bodies are real `sentences` rows so they inherit
-- the dual orthography layers, FTS trigger, quotation anchoring, and
-- reconcile UUID-carrying.

CREATE TABLE margin_notes (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    book_id             UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    number              INT NOT NULL,
    anchor_sentence_id  UUID NOT NULL REFERENCES sentences(id) ON DELETE CASCADE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (book_id, number)
);
CREATE INDEX idx_margin_notes_anchor ON margin_notes (anchor_sentence_id);

ALTER TABLE sentences
    ADD COLUMN margin_note_id UUID REFERENCES margin_notes(id) ON DELETE CASCADE;

ALTER TABLE sentences DROP CONSTRAINT chk_sentence_parent;
ALTER TABLE sentences ADD CONSTRAINT chk_sentence_parent CHECK (
    (block_id IS NOT NULL AND footnote_id IS NULL AND margin_note_id IS NULL) OR
    (block_id IS NULL AND footnote_id IS NOT NULL AND margin_note_id IS NULL) OR
    (block_id IS NULL AND footnote_id IS NULL AND margin_note_id IS NOT NULL)
);

-- Margin-note sentences get their own book-global numbering sequence,
-- separate from body and footnote sentences (same pattern as
-- idx_sentences_fn_num).
CREATE UNIQUE INDEX idx_sentences_mn_num
    ON sentences (book_id, sentence_number)
    WHERE sentence_number IS NOT NULL AND margin_note_id IS NOT NULL;
CREATE UNIQUE INDEX idx_sentences_margin_note_pos
    ON sentences (margin_note_id, position)
    WHERE margin_note_id IS NOT NULL;
CREATE INDEX idx_sentences_margin_note ON sentences (margin_note_id)
    WHERE margin_note_id IS NOT NULL;

-- Natural-key shape for margin-note sentences (see 0004):
--   "{node.source_ref}/mn{margin_note_number}/s{sentence_position}"

ALTER TYPE sentence_kind ADD VALUE 'margin';
