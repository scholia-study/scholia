-- Indented-run index within a paragraph. NULL = normal paragraph flow;
-- 1, 2, 3… mark consecutive `+ `-authored indented runs (e.g. Kant's numbered
-- `1) 2) 3)` enumerations). The paragraph stays a single block with one
-- paragraph_number; the reader groups consecutive same-segment sentences into
-- one hanging-indent block. Only set for block (paragraph) sentences.

ALTER TABLE sentences ADD COLUMN segment SMALLINT;
