-- Ancient Greek full-text search folding (ADR 0003 continuation).
--
-- Greek FTS needs its own tokenisation: readers can't reliably type accents
-- or breathings on a Latin keyboard, so those must fold away for search to
-- find anything. But the iota subscript (U+0345) is a real morphological
-- signal (it marks the dative singular of the first and second
-- declensions), so it's deliberately kept instead of being folded with the
-- rest of the combining diacritics. Postgres has no built-in Greek text
-- search config, so `grc` is tokenised unstemmed via 'simple', with all the
-- folding done in `grc_fold` before tokenisation.

CREATE OR REPLACE FUNCTION grc_fold(t TEXT) RETURNS TEXT
LANGUAGE sql IMMUTABLE STRICT PARALLEL SAFE AS $$
    SELECT normalize(
        regexp_replace(
            normalize(lower(translate(t, U&'\03C2', U&'\03C3')), NFD),
            '[̀-̈́͆-ͯ]', '', 'g'
        ), NFC);
$$;

-- Greek gets unstemmed 'simple' tokenisation; the accent/breathing/case
-- folding happens in grc_fold, not here.
CREATE OR REPLACE FUNCTION ts_config_for_lang(lang TEXT)
RETURNS regconfig
LANGUAGE sql IMMUTABLE AS $$
    SELECT CASE lang
        WHEN 'en' THEN 'english'::regconfig
        WHEN 'de' THEN 'german'::regconfig
        WHEN 'grc' THEN 'simple'::regconfig
        ELSE 'simple'::regconfig
    END
$$;

-- book_tsv and book_tsquery are a deliberately mirrored pair: index-time and
-- query-time text both pass through the same language-conditional folding,
-- so they can never drift apart.
CREATE OR REPLACE FUNCTION book_tsv(lang TEXT, t TEXT) RETURNS tsvector
LANGUAGE sql IMMUTABLE AS $$
    SELECT to_tsvector(
        ts_config_for_lang(lang),
        CASE WHEN lang = 'grc' THEN grc_fold(COALESCE(t, '')) ELSE COALESCE(t, '') END
    )
$$;

CREATE OR REPLACE FUNCTION book_tsquery(lang TEXT, q TEXT) RETURNS tsquery
LANGUAGE sql IMMUTABLE AS $$
    SELECT websearch_to_tsquery(
        ts_config_for_lang(lang),
        CASE WHEN lang = 'grc' THEN grc_fold(COALESCE(q, '')) ELSE COALESCE(q, '') END
    )
$$;

CREATE OR REPLACE FUNCTION set_book_tsv()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    lang TEXT;
BEGIN
    SELECT language INTO lang FROM books WHERE id = NEW.book_id;
    NEW.tsv := book_tsv(lang, NEW.text);
    RETURN NEW;
END
$$;

-- Restricted to grc books: every other language's stored tsvector is
-- byte-identical to what it already was, so this is a no-op for every
-- existing corpus.
UPDATE sentences s
    SET tsv = book_tsv(b.language, s.text)
    FROM books b
    WHERE b.id = s.book_id AND b.language = 'grc';

UPDATE content_blocks cb
    SET tsv = book_tsv(b.language, cb.text)
    FROM books b
    WHERE b.id = cb.book_id AND b.language = 'grc';
