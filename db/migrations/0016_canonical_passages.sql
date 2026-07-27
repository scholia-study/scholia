-- Canonical passages: edition-independent passage identity so resources
-- can project across editions of the same work (ADR 0008). Sibling
-- editions' sentences carrying the same content share one row; the
-- basis records which alignment mechanism minted it (1:1 sentence links
-- for Kant/poetry/drama, verse coordinates for Bibles). Importers seed
-- the mapping; existing corpora backfill on their next re-ingest.

CREATE TYPE canonical_basis AS ENUM ('sentence', 'verse');

CREATE TABLE canonical_passages (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_root        UUID NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    basis            canonical_basis NOT NULL,
    sentence_kind    sentence_kind NOT NULL DEFAULT 'body',
    -- Total order within (work_root, sentence_kind) for range-overlap
    -- tests: the source edition's sentence_number (sentence basis) or
    -- canonical-traversal position (verse basis).
    ordinal          INT NOT NULL,
    -- Sentence basis: the source-edition sentence this passage is.
    root_sentence_id UUID REFERENCES sentences(id) ON DELETE CASCADE,
    -- Verse basis: the canonical (KJV-numbering) coordinates.
    source_ref       TEXT,
    ref_value        TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_cp_sentence_basis CHECK
        ((basis = 'sentence') = (root_sentence_id IS NOT NULL)),
    CONSTRAINT chk_cp_verse_basis CHECK
        ((basis = 'verse') = (source_ref IS NOT NULL AND ref_value IS NOT NULL))
);

-- Idempotent seeding keys, one per basis.
CREATE UNIQUE INDEX uq_canonical_passages_sentence
    ON canonical_passages (root_sentence_id) WHERE basis = 'sentence';
CREATE UNIQUE INDEX uq_canonical_passages_verse
    ON canonical_passages (work_root, source_ref, ref_value) WHERE basis = 'verse';

CREATE INDEX idx_canonical_passages_work_ordinal
    ON canonical_passages (work_root, sentence_kind, ordinal);

-- NULL = no cross-edition identity (standalone works, DARBY's
-- translation-only title verses); such sentences never project.
ALTER TABLE sentences ADD COLUMN canonical_passage_id UUID
    REFERENCES canonical_passages(id) ON DELETE SET NULL;
CREATE INDEX idx_sentences_canonical_passage
    ON sentences (canonical_passage_id) WHERE canonical_passage_id IS NOT NULL;

-- Cataloguer's claim about what the resource is about: the passage in
-- any form ('work'), this language's translation layer ('language' —
-- all English editions "on the same page"), or this one edition's
-- actual text ('edition' — verbatim wording, translation-commentary).
-- Display/filter metadata only — it never suppresses projection; the
-- origin book (resources.book_id → books.language) supplies which
-- language/edition the claim is anchored to.
CREATE TYPE resource_scope AS ENUM ('work', 'language', 'edition');
ALTER TABLE resources ADD COLUMN scope resource_scope NOT NULL DEFAULT 'work';
