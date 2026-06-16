-- Poetry support (ADR 0003): a `verse` block type and per-line indentation.
--
-- Both statements are safe in one transaction: neither *uses* the new enum
-- value, so the PG rule "an added enum value can't be used in the same
-- transaction that adds it" does not apply here.

ALTER TYPE block_type ADD VALUE IF NOT EXISTS 'verse';

-- Verse line indent depth: 0/NULL = flush, 1.. = indent levels (rendered as
-- CSS padding). Distinct from `sentences.segment`, which is Kant's
-- hanging-indent run grouping and stays NULL for verse.
ALTER TABLE sentences ADD COLUMN indent SMALLINT;
