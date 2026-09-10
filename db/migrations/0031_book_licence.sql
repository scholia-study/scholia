-- Rights summary per hosted edition, surfaced prominently in the "About
-- this text" panel. `about_text` already carries the full provenance prose;
-- this is the standout label. Every ingestor now sets it (the struct-JSON
-- carries `book.licence`; `struct_to_db` and `bible_to_db` reject an empty
-- value), so the column is NOT NULL after backfilling the existing rows.
--
-- Scholia makes no licensing claim over source texts drawn from public-
-- domain editions or open APIs. Where a layer piggybacks a licensed
-- transcription (Deutsches Textarchiv, hegeledition.com) that licence is
-- carried through. Original translations and modernized reading texts
-- prepared by Scholia are CC BY-NC-ND 4.0.

ALTER TABLE books ADD COLUMN licence TEXT;

UPDATE books SET licence = 'Public Domain'
WHERE slug IN (
    'kjv-bible', 'web-bible', 'asv-bible', 'bbe-bible', 'darby-bible',
    'essays-in-pragmaticism',
    'politeia'
);

UPDATE books SET licence = 'Public Domain (source text); CC BY-NC-ND 4.0 (modernized reading text)'
WHERE slug IN (
    'kritik-der-reinen-vernunft-b',
    'kritik-der-urteilskraft',
    'leviathan',
    'paradise-lost',
    'shakespeares-sonnets',
    'keiser-og-galileer'
);

UPDATE books SET licence = 'Public Domain. German text layers from the Deutsches Textarchiv (CC BY-SA 4.0).'
WHERE slug IN ('phaenomenologie-des-geistes', 'das-seyn-1812');

UPDATE books SET licence = 'Public Domain. German text layers from hegeledition.com (CC BY 4.0) and the Deutsches Textarchiv (CC BY-SA 4.0).'
WHERE slug = 'wissenschaft-der-logik';

UPDATE books SET licence = 'CC BY-NC-ND 4.0'
WHERE slug IN (
    'critique-of-pure-reason-b',
    'critique-of-the-power-of-judgment',
    'phenomenology-of-spirit',
    'science-of-logic',
    'doctrine-of-being-1812',
    'republic',
    'emperor-and-galilean'
);

-- Safety net for any book row not enumerated above (e.g. a corpus imported
-- into a branch DB but not yet in the roster): the dominant case is a
-- Scholia digitization of a public-domain source.
UPDATE books SET licence = 'Public Domain' WHERE licence IS NULL;

ALTER TABLE books ALTER COLUMN licence SET NOT NULL;
