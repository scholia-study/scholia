-- Reader margin prefix per reference system, concatenated VERBATIM with the
-- marker value — spacing is part of the prefix ("B " → "B 344", "9." →
-- "9.53"). NULL = bare number: the copy-text's own pagination and line
-- numbers stay unprefixed; scholarly/imported systems carry one so a reader
-- can tell which system a margin number belongs to.

ALTER TABLE reference_systems ADD COLUMN margin_prefix TEXT;

-- Backfill
UPDATE reference_systems SET margin_prefix = 'AA ' WHERE slug IN ('aa_iii', 'aa_v');
UPDATE reference_systems SET margin_prefix = 'B '  WHERE slug = 'b_edition';
UPDATE reference_systems SET margin_prefix = 'E '  WHERE slug = 'e1790';
UPDATE reference_systems SET margin_prefix = '9.'  WHERE slug = 'gw9';
