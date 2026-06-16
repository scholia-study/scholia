-- Per-language full-text search (ADR 0003).
--
-- The initial schema hard-coded `to_tsvector('german', text)` expression
-- indexes, which stem English texts (Bible, Shakespeare) under German rules.
-- Replace them with a stored `tsv` column populated, via trigger, from each
-- book's language — so every text is stemmed in its own language.
--
-- These FTS indexes are not yet queried by the application, so this is a pure
-- infrastructure swap: no query code changes. A generated column can't be used
-- (the per-row config isn't IMMUTABLE), hence a trigger-maintained column.

-- ISO language code -> text search configuration. Unknown languages fall back
-- to 'simple' (no stemming, but still tokenises).
CREATE OR REPLACE FUNCTION ts_config_for_lang(lang TEXT)
RETURNS regconfig
LANGUAGE sql IMMUTABLE AS $$
    SELECT CASE lang
        WHEN 'en' THEN 'english'::regconfig
        WHEN 'de' THEN 'german'::regconfig
        ELSE 'simple'::regconfig
    END
$$;

ALTER TABLE sentences ADD COLUMN tsv tsvector;
ALTER TABLE content_blocks ADD COLUMN tsv tsvector;

-- Shared by both tables — each has book_id, text, and tsv columns.
CREATE OR REPLACE FUNCTION set_book_tsv()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    cfg regconfig;
BEGIN
    SELECT ts_config_for_lang(language) INTO cfg FROM books WHERE id = NEW.book_id;
    NEW.tsv := to_tsvector(COALESCE(cfg, 'simple'::regconfig), COALESCE(NEW.text, ''));
    RETURN NEW;
END
$$;

CREATE TRIGGER trg_sentences_tsv
    BEFORE INSERT OR UPDATE OF text, book_id ON sentences
    FOR EACH ROW EXECUTE FUNCTION set_book_tsv();

CREATE TRIGGER trg_content_blocks_tsv
    BEFORE INSERT OR UPDATE OF text, book_id ON content_blocks
    FOR EACH ROW EXECUTE FUNCTION set_book_tsv();

-- Backfill any rows already present (set-based, joins each row to its book).
UPDATE sentences s
    SET tsv = to_tsvector(ts_config_for_lang(b.language), COALESCE(s.text, ''))
    FROM books b
    WHERE b.id = s.book_id;

UPDATE content_blocks cb
    SET tsv = to_tsvector(ts_config_for_lang(b.language), COALESCE(cb.text, ''))
    FROM books b
    WHERE b.id = cb.book_id;

-- Swap the german expression indexes for generic GIN over the tsv columns.
DROP INDEX IF EXISTS idx_sentences_fts;
DROP INDEX IF EXISTS idx_blocks_fts;
CREATE INDEX idx_sentences_fts ON sentences USING gin (tsv);
CREATE INDEX idx_blocks_fts ON content_blocks USING gin (tsv);
