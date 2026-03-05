-- Prospero: interactive reader schema
-- PostgreSQL 15+, requires ltree extension

CREATE EXTENSION IF NOT EXISTS ltree;

-- ============================================================
-- TEXT TABLES
-- ============================================================

-- One row per work. For translations, source_book_id points to
-- the original work. Source texts have source_book_id = NULL.
CREATE TABLE books (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_book_id  UUID REFERENCES books(id) ON DELETE SET NULL,
    slug            TEXT NOT NULL UNIQUE,
    title           TEXT NOT NULL,
    author          TEXT NOT NULL,
    language        TEXT NOT NULL,
    source          TEXT,
    source_date     TEXT,
    admin_notes     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The table-of-contents tree. Each node is a section heading
-- that can contain content blocks and child nodes.
--
-- Uses adjacency list (parent_id) for simple parent/child
-- queries AND materialized path (ltree) for efficient
-- ancestor/descendant queries. Both are maintained because
-- the text data is static once imported.
--
-- source_ref: generic per-source identifier
--   Hegel: NCX id (e.g. "np-42")
--   Kant: position string (e.g. "001", "003")
-- sort_order: display/reading order within the book
CREATE TABLE toc_nodes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    book_id         UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    parent_id       UUID REFERENCES toc_nodes(id) ON DELETE CASCADE,
    source_node_id  UUID REFERENCES toc_nodes(id) ON DELETE SET NULL,
    source_ref      TEXT NOT NULL,
    slug            TEXT NOT NULL,
    path            LTREE NOT NULL,
    sort_order      INT NOT NULL,
    depth           SMALLINT NOT NULL,
    label           TEXT NOT NULL,
    admin_notes     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (book_id, source_ref),
    UNIQUE (book_id, slug),
    UNIQUE (book_id, sort_order)
);

CREATE INDEX idx_nodes_path ON toc_nodes USING gist (path);
CREATE INDEX idx_nodes_parent ON toc_nodes (parent_id);
CREATE INDEX idx_nodes_book_order ON toc_nodes (book_id, sort_order);

-- Content blocks: the actual text units within each section.
-- Four types: paragraph (body text), heading (section title),
-- footnote (authorial note), separator (visual break).
CREATE TYPE block_type AS ENUM (
    'paragraph', 'heading', 'footnote', 'separator'
);

CREATE TABLE content_blocks (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    book_id           UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    node_id           UUID NOT NULL REFERENCES toc_nodes(id) ON DELETE CASCADE,
    position          SMALLINT NOT NULL,
    block_type        block_type NOT NULL,
    paragraph_number  INT,
    text              TEXT NOT NULL DEFAULT '',
    html              TEXT NOT NULL DEFAULT '',
    admin_notes       TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (node_id, position)
);

CREATE UNIQUE INDEX idx_blocks_para_num
    ON content_blocks (book_id, paragraph_number)
    WHERE paragraph_number IS NOT NULL;
CREATE INDEX idx_blocks_node_pos ON content_blocks (node_id, position);
CREATE INDEX idx_blocks_fts ON content_blocks
    USING gin (to_tsvector('german', text))
    WHERE block_type = 'paragraph';

-- Individual sentences within content blocks.
-- All non-separator block types get sentence rows:
--   heading   → 1 sentence (the heading text)
--   footnote  → 1+ sentences (sentence-split like paragraphs)
--   paragraph → 1+ sentences
--   separator → no sentences
--
-- sentence_number is only set for paragraph sentences (global
-- body-text enumeration). Heading/footnote sentences exist for
-- anchoring but are not counted.
--
-- For translation sentences, source_sentence_start/end_id point
-- to the source sentence(s) this was translated from.
--   1:1  — start set, end NULL
--   merge (2 source → 1 translated) — start + end set
--   split (1 source → 2 translated) — both point to same source via start
CREATE TABLE sentences (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    book_id                   UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    node_id                   UUID NOT NULL REFERENCES toc_nodes(id) ON DELETE CASCADE,
    block_id                  UUID NOT NULL REFERENCES content_blocks(id) ON DELETE CASCADE,
    position                  SMALLINT NOT NULL,
    sentence_number           INT,
    source_sentence_start_id  UUID REFERENCES sentences(id) ON DELETE SET NULL,
    source_sentence_end_id    UUID REFERENCES sentences(id) ON DELETE SET NULL,
    text                      TEXT NOT NULL,
    html                      TEXT NOT NULL,
    admin_notes               TEXT,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (block_id, position),

    CONSTRAINT chk_source_sentence_range CHECK (
        source_sentence_end_id IS NULL OR source_sentence_start_id IS NOT NULL
    )
);

CREATE UNIQUE INDEX idx_sentences_num
    ON sentences (book_id, sentence_number)
    WHERE sentence_number IS NOT NULL;
CREATE INDEX idx_sentences_block_pos ON sentences (block_id, position);
CREATE INDEX idx_sentences_node ON sentences (node_id);
CREATE INDEX idx_sentences_source ON sentences (source_sentence_start_id)
    WHERE source_sentence_start_id IS NOT NULL;
CREATE INDEX idx_sentences_fts ON sentences
    USING gin (to_tsvector('german', text));

-- ============================================================
-- REFERENCE SYSTEMS (page numbers, edition markers, etc.)
-- ============================================================

-- Each book can have multiple reference systems (e.g. Zeno page
-- numbers, Akademie-Ausgabe pages, B-edition markers).
-- ref_type tells the frontend how to render:
--   'block'  → margin annotation
--   'inline' → inline marker within text
CREATE TABLE reference_systems (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    book_id       UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    slug          TEXT NOT NULL,
    label         TEXT NOT NULL,
    description   TEXT,
    ref_type      TEXT NOT NULL CHECK (ref_type IN ('block', 'inline')),
    admin_notes   TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (book_id, slug)
);

-- Individual page/reference markers anchored to sentences.
-- No block_id — derive via sentences.block_id.
-- char_offset is relative to sentence.text; NULL = start of sentence.
CREATE TABLE page_markers (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id     UUID NOT NULL REFERENCES reference_systems(id) ON DELETE CASCADE,
    sentence_id   UUID NOT NULL REFERENCES sentences(id) ON DELETE CASCADE,
    ref_value     TEXT NOT NULL,
    sort_order    INT NOT NULL,
    char_offset   INT,
    admin_notes   TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_markers_sentence ON page_markers (sentence_id);
CREATE INDEX idx_markers_system_order ON page_markers (system_id, sort_order);
CREATE INDEX idx_markers_system_value ON page_markers (system_id, ref_value);

-- ============================================================
-- RESOURCE TABLES (curated/editorial content)
-- ============================================================

-- Resources attached to text locations: commentary, definitions,
-- external links, essays, etc. Uses a type discriminator + JSONB
-- for type-specific fields.
--
-- ANCHOR PATTERN (shared by resources, user_notes, chat_conversations):
--
-- Every anchored row points to a text location at one of four
-- granularities:
--
--   node-level:       anchor_node_id only
--   block-level:      + anchor_block_id
--   single sentence:  + anchor_sentence_start_id (end is NULL)
--   sentence range:   + anchor_sentence_start_id + anchor_sentence_end_id
--
-- Sentence ranges can span multiple paragraphs (e.g. the last
-- sentence of one paragraph through the first of the next).
-- The sentences themselves carry their block FK, so we don't
-- require anchor_block_id when sentences are set.
CREATE TABLE resources (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    book_id                   UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    resource_type             TEXT NOT NULL,
    anchor_node_id            UUID NOT NULL REFERENCES toc_nodes(id) ON DELETE CASCADE,
    anchor_block_id           UUID REFERENCES content_blocks(id) ON DELETE CASCADE,
    anchor_sentence_start_id  UUID REFERENCES sentences(id) ON DELETE CASCADE,
    anchor_sentence_end_id    UUID REFERENCES sentences(id) ON DELETE CASCADE,
    title                     TEXT,
    body                      TEXT,
    metadata                  JSONB NOT NULL DEFAULT '{}',
    sort_order                INT NOT NULL DEFAULT 0,
    admin_notes               TEXT,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_resource_anchor CHECK (
        anchor_sentence_end_id IS NULL OR anchor_sentence_start_id IS NOT NULL
    )
);

CREATE INDEX idx_resources_sentence_start ON resources (anchor_sentence_start_id)
    WHERE anchor_sentence_start_id IS NOT NULL;
CREATE INDEX idx_resources_sentence_end ON resources (anchor_sentence_end_id)
    WHERE anchor_sentence_end_id IS NOT NULL;
CREATE INDEX idx_resources_block ON resources (anchor_block_id)
    WHERE anchor_block_id IS NOT NULL;
CREATE INDEX idx_resources_node ON resources (anchor_node_id);
CREATE INDEX idx_resources_book_type ON resources (book_id, resource_type);

-- Links between two text locations. Separate from resources
-- because it connects TWO anchors and needs bidirectional queries.
CREATE TABLE cross_references (
    id                            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    book_id                       UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    label                         TEXT NOT NULL,
    description                   TEXT,

    source_node_id                UUID NOT NULL REFERENCES toc_nodes(id) ON DELETE CASCADE,
    source_block_id               UUID REFERENCES content_blocks(id) ON DELETE CASCADE,
    source_sentence_start_id      UUID REFERENCES sentences(id) ON DELETE CASCADE,
    source_sentence_end_id        UUID REFERENCES sentences(id) ON DELETE CASCADE,

    target_node_id                UUID NOT NULL REFERENCES toc_nodes(id) ON DELETE CASCADE,
    target_block_id               UUID REFERENCES content_blocks(id) ON DELETE CASCADE,
    target_sentence_start_id      UUID REFERENCES sentences(id) ON DELETE CASCADE,
    target_sentence_end_id        UUID REFERENCES sentences(id) ON DELETE CASCADE,

    metadata                      JSONB NOT NULL DEFAULT '{}',
    admin_notes                   TEXT,
    created_at                    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                    TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_xref_source CHECK (
        source_sentence_end_id IS NULL OR source_sentence_start_id IS NOT NULL
    ),
    CONSTRAINT chk_xref_target CHECK (
        target_sentence_end_id IS NULL OR target_sentence_start_id IS NOT NULL
    )
);

CREATE INDEX idx_xref_source_sentence ON cross_references (source_sentence_start_id)
    WHERE source_sentence_start_id IS NOT NULL;
CREATE INDEX idx_xref_target_sentence ON cross_references (target_sentence_start_id)
    WHERE target_sentence_start_id IS NOT NULL;
CREATE INDEX idx_xref_source_block ON cross_references (source_block_id)
    WHERE source_block_id IS NOT NULL;
CREATE INDEX idx_xref_target_block ON cross_references (target_block_id)
    WHERE target_block_id IS NOT NULL;

-- ============================================================
-- USER TABLES
-- ============================================================

CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id   TEXT UNIQUE,
    display_name  TEXT NOT NULL,
    email         TEXT,
    admin_notes   TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- User notes anchored to text locations. Same anchor pattern
-- as resources but with an owner.
CREATE TABLE user_notes (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id                   UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    anchor_node_id            UUID NOT NULL REFERENCES toc_nodes(id) ON DELETE CASCADE,
    anchor_block_id           UUID REFERENCES content_blocks(id) ON DELETE CASCADE,
    anchor_sentence_start_id  UUID REFERENCES sentences(id) ON DELETE CASCADE,
    anchor_sentence_end_id    UUID REFERENCES sentences(id) ON DELETE CASCADE,
    body                      TEXT NOT NULL,
    is_public                 BOOLEAN NOT NULL DEFAULT FALSE,
    admin_notes               TEXT,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_note_anchor CHECK (
        anchor_sentence_end_id IS NULL OR anchor_sentence_start_id IS NOT NULL
    )
);

CREATE INDEX idx_notes_sentence_start ON user_notes (anchor_sentence_start_id)
    WHERE anchor_sentence_start_id IS NOT NULL;
CREATE INDEX idx_notes_block ON user_notes (anchor_block_id)
    WHERE anchor_block_id IS NOT NULL;
CREATE INDEX idx_notes_node ON user_notes (anchor_node_id);
CREATE INDEX idx_notes_user_book ON user_notes (user_id, book_id, created_at DESC);

-- AI chat conversations anchored to text locations.
CREATE TABLE chat_conversations (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id                   UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    anchor_node_id            UUID NOT NULL REFERENCES toc_nodes(id) ON DELETE CASCADE,
    anchor_block_id           UUID REFERENCES content_blocks(id) ON DELETE CASCADE,
    anchor_sentence_start_id  UUID REFERENCES sentences(id) ON DELETE CASCADE,
    anchor_sentence_end_id    UUID REFERENCES sentences(id) ON DELETE CASCADE,
    title               TEXT,
    model               TEXT,
    admin_notes         TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_chat_anchor CHECK (
        anchor_sentence_end_id IS NULL OR anchor_sentence_start_id IS NOT NULL
    )
);

CREATE INDEX idx_chats_sentence ON chat_conversations (anchor_sentence_start_id)
    WHERE anchor_sentence_start_id IS NOT NULL;
CREATE INDEX idx_chats_node ON chat_conversations (anchor_node_id);
CREATE INDEX idx_chats_user ON chat_conversations (user_id, updated_at DESC);

-- Individual messages within a conversation.
CREATE TABLE chat_messages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES chat_conversations(id) ON DELETE CASCADE,
    role            TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content         TEXT NOT NULL,
    admin_notes     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_messages_conv ON chat_messages (conversation_id, created_at);
