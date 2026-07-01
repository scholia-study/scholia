-- Drama support (Ibsen's Emperor and Galilean): a `speaker` block type for a
-- character's speech label and a `stage` block type for stage directions /
-- dramatis-personae apparatus. Both are non-clickable chrome (their sentences
-- carry sentence_number = NULL), rendered like headings. See the drama plan and
-- ADR 0005.
--
-- Safe in one transaction: neither statement *uses* the new enum value, so the
-- PG rule "an added enum value can't be used in the same transaction that adds
-- it" does not apply (mirrors 0001 `figure` and 0006 `verse`).

ALTER TYPE block_type ADD VALUE IF NOT EXISTS 'speaker';
ALTER TYPE block_type ADD VALUE IF NOT EXISTS 'stage';
