-- Declarative citation systems. `cite_priority` (lowest wins; NULL = not a
-- default citation) says WHICH reference system(s) a book cites by, replacing
-- the old `reference_label = first marker where ref_type='inline'` heuristic.
-- `cite_template` (tokens {parent}=parent node label, {self}=node label,
-- {ref}=marker value; {ref} expands to first–last for a range) says HOW to
-- phrase it. A non-NULL template = citation-capable; a non-NULL priority = a
-- default citation candidate. A book with no priority system falls back to
-- sentence-number citation ("s. N").

ALTER TABLE reference_systems ADD COLUMN cite_priority SMALLINT;
ALTER TABLE reference_systems ADD COLUMN cite_template TEXT;

-- Backfill existing rows. Slugs are book-scoped and unique except 'line'
-- (Shakespeare vs Milton), disambiguated by book. Keep these in sync with the
-- ingest config (each importer writes the same values on (re)import).

-- Bible: cite by verse, e.g. "Romans 13:2" ({parent}=Bible-book node).
UPDATE reference_systems SET cite_priority = 0, cite_template = '{parent} {ref}'
WHERE slug = 'verse';

-- Kant: A/B (and edition) systems are citation-capable but NOT the default
-- (priority NULL) — Kant cites by sentence. Kritik der reinen Vernunft
-- (aa_iii/b_edition) + Kritik der Urteilskraft (aa_v/e1790).
UPDATE reference_systems SET cite_template = 'AA III {ref}' WHERE slug = 'aa_iii';
UPDATE reference_systems SET cite_template = 'B {ref}'      WHERE slug = 'b_edition';
UPDATE reference_systems SET cite_template = 'AA V {ref}'   WHERE slug = 'aa_v';
UPDATE reference_systems SET cite_template = 'E {ref}'      WHERE slug = 'e1790';
