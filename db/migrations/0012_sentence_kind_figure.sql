-- Quotations on figure blocks. Figure anchor sentences sit outside the body
-- enumeration (sentence_number IS NULL) and are addressed by
-- content_blocks.figure_number instead, so they need their own sentence_kind
-- the way footnote sentences do.
ALTER TYPE sentence_kind ADD VALUE 'figure';
