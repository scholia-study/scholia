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
    'paragraph', 'heading', 'separator'
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
    original_text     TEXT,
    original_html     TEXT,
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

-- Individual sentences within content blocks or footnotes.
-- Block sentences: heading/paragraph sentences (block_id set, footnote_id NULL).
-- Footnote sentences: belong to a footnote (footnote_id set, block_id NULL).
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
    block_id                  UUID REFERENCES content_blocks(id) ON DELETE CASCADE,
    footnote_id               UUID,
    position                  SMALLINT NOT NULL,
    sentence_number           INT,
    source_sentence_start_id  UUID REFERENCES sentences(id) ON DELETE SET NULL,
    source_sentence_end_id    UUID REFERENCES sentences(id) ON DELETE SET NULL,
    text                      TEXT NOT NULL,
    html                      TEXT NOT NULL,
    original_text             TEXT,
    original_html             TEXT,
    admin_notes               TEXT,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_sentence_parent CHECK (
        (block_id IS NOT NULL AND footnote_id IS NULL) OR
        (block_id IS NULL AND footnote_id IS NOT NULL)
    ),

    CONSTRAINT chk_source_sentence_range CHECK (
        source_sentence_end_id IS NULL OR source_sentence_start_id IS NOT NULL
    )
);

CREATE UNIQUE INDEX idx_sentences_block_num
    ON sentences (book_id, sentence_number)
    WHERE sentence_number IS NOT NULL AND block_id IS NOT NULL;
CREATE UNIQUE INDEX idx_sentences_fn_num
    ON sentences (book_id, sentence_number)
    WHERE sentence_number IS NOT NULL AND footnote_id IS NOT NULL;
CREATE UNIQUE INDEX idx_sentences_block_pos ON sentences (block_id, position)
    WHERE block_id IS NOT NULL;
CREATE UNIQUE INDEX idx_sentences_footnote_pos ON sentences (footnote_id, position)
    WHERE footnote_id IS NOT NULL;
CREATE INDEX idx_sentences_node ON sentences (node_id);
CREATE INDEX idx_sentences_source ON sentences (source_sentence_start_id)
    WHERE source_sentence_start_id IS NOT NULL;
CREATE INDEX idx_sentences_fts ON sentences
    USING gin (to_tsvector('german', text));

-- ============================================================
-- FOOTNOTES (authorial notes attached to anchor sentences)
-- ============================================================

CREATE TABLE footnotes (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    book_id             UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    number              INT NOT NULL,
    anchor_sentence_id  UUID NOT NULL REFERENCES sentences(id) ON DELETE CASCADE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (book_id, number)
);
CREATE INDEX idx_footnotes_anchor ON footnotes (anchor_sentence_id);

-- Now add the FK from sentences.footnote_id -> footnotes.id
ALTER TABLE sentences ADD CONSTRAINT fk_sentences_footnote
    FOREIGN KEY (footnote_id) REFERENCES footnotes(id) ON DELETE CASCADE;

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
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    display_name      TEXT NOT NULL,
    email             TEXT NOT NULL UNIQUE,
    password_hash         TEXT,
    avatar_url            TEXT,
    email_verified_at     TIMESTAMPTZ,
    sessions_invalidated_at TIMESTAMPTZ,
    admin_notes       TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- BIBLIOGRAPHIC TABLES (sources & persons for commentary)
-- ============================================================

CREATE TABLE persons (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    sort_name   TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TYPE source_type AS ENUM ('book', 'article', 'chapter', 'journal', 'web');

CREATE TABLE sources (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_type       source_type NOT NULL,
    title             TEXT NOT NULL,
    publication_year  SMALLINT,
    publisher         TEXT,
    isbn              TEXT[],
    doi               TEXT,
    edition           TEXT,
    volume            TEXT,
    journal_name      TEXT,
    url               TEXT,
    page_start        INT,
    page_end          INT,
    parent_source_id  UUID REFERENCES sources(id) ON DELETE SET NULL,
    created_by        UUID REFERENCES users(id),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_chapter_has_parent CHECK (
        source_type != 'chapter' OR parent_source_id IS NOT NULL
    ),
    CONSTRAINT chk_no_parent CHECK (
        source_type NOT IN ('book', 'web') OR parent_source_id IS NULL
    )
);

CREATE INDEX idx_sources_parent ON sources (parent_source_id)
    WHERE parent_source_id IS NOT NULL;
CREATE INDEX idx_sources_title ON sources USING gin (to_tsvector('english', title));

CREATE TYPE source_person_role AS ENUM ('author', 'editor', 'translator', 'contributor');

CREATE TABLE source_persons (
    source_id  UUID NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    person_id  UUID NOT NULL REFERENCES persons(id) ON DELETE RESTRICT,
    role       source_person_role NOT NULL,
    position   SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (source_id, person_id, role)
);

CREATE INDEX idx_source_persons_person ON source_persons (person_id);

-- ============================================================
-- RESOURCE TABLES (curated/editorial content)
-- ============================================================

-- Resources attached to text locations: commentary, definitions,
-- external links, essays, etc.
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

CREATE TYPE resource_type AS ENUM ('verbatim', 'paraphrase', 'allusion');
CREATE TYPE verbatim_kind AS ENUM ('entirety', 'fragmentary');
CREATE TYPE sentence_kind AS ENUM ('body', 'footnote');

CREATE TABLE resources (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    book_id                   UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    resource_type             resource_type NOT NULL,
    anchor_node_id            UUID NOT NULL REFERENCES toc_nodes(id) ON DELETE CASCADE,
    anchor_block_id           UUID REFERENCES content_blocks(id) ON DELETE CASCADE,
    anchor_sentence_start_id  UUID REFERENCES sentences(id) ON DELETE CASCADE,
    anchor_sentence_end_id    UUID REFERENCES sentences(id) ON DELETE CASCADE,
    sentence_kind             sentence_kind NOT NULL DEFAULT 'body',
    source_id                 UUID REFERENCES sources(id),
    source_page_start         INT,
    source_page_end           INT,
    source_location_freeform  TEXT,
    verbatim_kind             verbatim_kind,
    quoted_text               TEXT,
    editor_note               TEXT,
    is_featured               BOOLEAN NOT NULL DEFAULT false,
    archived_at               TIMESTAMPTZ,
    archived_by               UUID REFERENCES users(id),
    admin_notes               TEXT,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_resource_anchor CHECK (
        anchor_sentence_end_id IS NULL OR anchor_sentence_start_id IS NOT NULL
    ),
    CONSTRAINT chk_source_location CHECK (
        NOT (source_page_start IS NOT NULL AND source_location_freeform IS NOT NULL)
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
CREATE INDEX idx_resources_source ON resources (source_id)
    WHERE source_id IS NOT NULL;
CREATE INDEX idx_resources_active ON resources (book_id, sentence_kind)
    WHERE archived_at IS NULL;

-- ============================================================
-- ROLES & PERMISSIONS
-- ============================================================

CREATE TABLE roles (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE user_roles (
    user_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id  UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX idx_user_roles_role ON user_roles (role_id);

-- Seed default roles
INSERT INTO roles (name) VALUES ('admin'), ('editor'), ('user');

-- ============================================================
-- OAUTH ACCOUNTS
-- ============================================================

CREATE TABLE user_oauth_accounts (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider          TEXT NOT NULL,
    provider_user_id  TEXT NOT NULL,
    email             TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (provider, provider_user_id)
);

CREATE INDEX idx_oauth_user ON user_oauth_accounts (user_id);

-- ============================================================
-- USER SESSIONS (maps users to tower-sessions session IDs)
-- ============================================================

CREATE TABLE user_sessions (
    session_id  TEXT NOT NULL,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (session_id)
);

CREATE INDEX idx_user_sessions_user ON user_sessions (user_id);

-- ============================================================
-- EMAIL VERIFICATION TOKENS
-- ============================================================

CREATE TABLE email_verification_tokens (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL,
    expires_at  TIMESTAMPTZ NOT NULL,
    used_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_email_verify_user ON email_verification_tokens (user_id);

-- ============================================================
-- PASSWORD RESET TOKENS
-- ============================================================

CREATE TABLE password_reset_tokens (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL,
    expires_at  TIMESTAMPTZ NOT NULL,
    used_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_password_reset_user ON password_reset_tokens (user_id);

-- ============================================================
-- QUOTATIONS & NOTES (user-saved text anchors + commentary)
-- ============================================================

-- A quotation is a user's saved pointer to a sentence or sentence range.
-- No copied text — always a strict reference to the source.
CREATE TABLE quotations (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id                   UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    anchor_node_id            UUID NOT NULL REFERENCES toc_nodes(id) ON DELETE CASCADE,
    anchor_sentence_start_id  UUID NOT NULL REFERENCES sentences(id) ON DELETE CASCADE,
    anchor_sentence_end_id    UUID REFERENCES sentences(id) ON DELETE CASCADE,
    sentence_kind             sentence_kind NOT NULL DEFAULT 'body',
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Deduplicate: one quotation per user per unique sentence range.
-- COALESCE handles nullable end_id for the unique constraint.
CREATE UNIQUE INDEX idx_quotations_user_range
    ON quotations (user_id, anchor_sentence_start_id, COALESCE(anchor_sentence_end_id, '00000000-0000-0000-0000-000000000000'));
CREATE INDEX idx_quotations_node ON quotations (user_id, book_id, anchor_node_id);
CREATE INDEX idx_quotations_user_book ON quotations (user_id, book_id, created_at DESC);

-- Notes attached to quotations. Plain text, private, one-to-many.
CREATE TABLE quotation_notes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    quotation_id    UUID NOT NULL REFERENCES quotations(id) ON DELETE CASCADE,
    body            TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_qnotes_quotation ON quotation_notes (quotation_id, created_at DESC);

-- Free-form tags, scoped per user.
CREATE TABLE tags (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name      TEXT NOT NULL,
    UNIQUE(user_id, name)
);

-- Many-to-many: tags on notes.
CREATE TABLE quotation_note_tags (
    note_id   UUID NOT NULL REFERENCES quotation_notes(id) ON DELETE CASCADE,
    tag_id    UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (note_id, tag_id)
);

CREATE INDEX idx_qntags_tag ON quotation_note_tags (tag_id);

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
