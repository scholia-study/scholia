-- Scholia: interactive reader schema
-- PostgreSQL 18+, requires ltree extension

CREATE EXTENSION IF NOT EXISTS ltree;

-- ============================================================
-- USER TABLES
-- ============================================================

CREATE TABLE users (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    display_name      TEXT NOT NULL,
    -- Bibliography sort key, "Last, First" form. Auto-derived from
    -- display_name at signup; user-editable thereafter.
    sort_name         TEXT,
    -- Public URL identifier. UNIQUE; auto-derived at signup; user-editable
    -- with a 30-day cooldown enforced in the handler. Charset is
    -- [a-z0-9-], 1–40 chars, no leading/trailing hyphen.
    handle            TEXT UNIQUE,
    -- Set whenever `handle` is updated. NULL means "never renamed";
    -- the cooldown check treats NULL as "no cooldown applies".
    handle_changed_at TIMESTAMPTZ,
    -- Profile page fields. All optional; rendered on /users/<handle>.
    bio               TEXT,
    title             TEXT,
    location          TEXT,
    website_url       TEXT,
    email             TEXT NOT NULL UNIQUE,
    password_hash         TEXT,
    avatar_url            TEXT,
    email_verified_at     TIMESTAMPTZ,
    sessions_invalidated_at TIMESTAMPTZ,
    -- Stripe customer ID. NULL until the user initiates their first
    -- checkout (lazy creation). Once set, never cleared — same customer
    -- is reused for all future subscriptions, refunds, etc.
    stripe_customer_id    TEXT UNIQUE,
    admin_notes       TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Permanent record of every handle ever held by a user. Used to prevent
-- handle recycling: a handle that was ever in `users.handle` (and later
-- changed) is reserved for that user forever — others cannot claim it,
-- but the original owner can rename back to it.
CREATE TABLE released_handles (
    handle      TEXT PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    released_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_released_handles_user ON released_handles (user_id);

-- Seed the system user. Owns all seed/import-created persons and sources.
-- Login is impossible: no password_hash, and the login handler rejects this email.
INSERT INTO users (id, display_name, handle, email, email_verified_at)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'System',
    'system',
    'system@scholia.local',
    now()
);

-- ============================================================
-- BIBLIOGRAPHIC TABLES (sources & persons)
-- ============================================================

CREATE TABLE persons (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL UNIQUE,
    sort_name   TEXT,
    protected   BOOLEAN NOT NULL DEFAULT false,
    created_by  UUID NOT NULL REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TYPE source_type AS ENUM ('book', 'article', 'chapter', 'journal', 'web');

CREATE TABLE sources (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_type       source_type NOT NULL,
    title             TEXT NOT NULL,
    title_display     TEXT,
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
    parent_source_id    UUID REFERENCES sources(id) ON DELETE SET NULL,
    translation_of_id   UUID REFERENCES sources(id) ON DELETE SET NULL,
    protected           BOOLEAN NOT NULL DEFAULT false,
    created_by          UUID NOT NULL REFERENCES users(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_chapter_has_parent CHECK (
        source_type != 'chapter' OR parent_source_id IS NOT NULL
    ),
    CONSTRAINT chk_no_parent CHECK (
        source_type NOT IN ('book', 'web') OR parent_source_id IS NULL
    ),
    UNIQUE (title, source_type, publication_year)
);

CREATE INDEX idx_sources_parent ON sources (parent_source_id)
    WHERE parent_source_id IS NOT NULL;
CREATE INDEX idx_sources_translation ON sources (translation_of_id)
    WHERE translation_of_id IS NOT NULL;
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
-- TEXT TABLES
-- ============================================================

-- One row per hosted text. Bibliographic metadata (title, authors,
-- translation linkage) lives in the linked source.
CREATE TABLE books (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id       UUID NOT NULL REFERENCES sources(id),
    slug            TEXT NOT NULL UNIQUE,
    language        TEXT NOT NULL,
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
    label_html      TEXT,
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
-- RESOURCE TABLES (curated/editorial content)
-- ============================================================

-- Resources attached to text locations: commentary, definitions,
-- external links, essays, etc.
--
-- ANCHOR PATTERN (shared by resources, user_notes):
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

-- Seed default roles. `honorary` is a comp-tier role granted manually
-- by admins (e.g. as a thank-you to contributors); it grants the same
-- elevated limits as the paid tiers but is never touched by Stripe
-- webhook role-sync, so a user can be honorary without a subscription
-- and won't lose the role if their (separate) paid sub lapses.
INSERT INTO roles (name) VALUES ('admin'), ('editor'), ('user'),
    ('scholiast'), ('scholiast_benefactor'), ('scholiast_patron'),
    ('honorary');

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

-- ═══════════════════════════════════════════════════════════
-- ARTICLES & TOPICS
-- ═══════════════════════════════════════════════════════════

CREATE TYPE article_status AS ENUM ('draft', 'published', 'archived');

-- Global topics managed by editors/admins.
CREATE TABLE topics (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL UNIQUE,
    slug       TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- User-authored articles with markdown source and rendered HTML.
CREATE TABLE articles (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title        TEXT NOT NULL,
    slug         TEXT NOT NULL UNIQUE,
    description  TEXT,
    markdown     TEXT NOT NULL DEFAULT '',
    html         TEXT NOT NULL DEFAULT '',
    status       article_status NOT NULL DEFAULT 'draft',
    published_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_articles_user_status ON articles (user_id, status);
CREATE INDEX idx_articles_user_updated ON articles (user_id, updated_at DESC);
CREATE INDEX idx_articles_published ON articles (status, published_at DESC) WHERE status = 'published';

-- Many-to-many: topics on articles (max 5 per article, enforced in app).
CREATE TABLE article_topics (
    article_id UUID NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    topic_id   UUID NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    PRIMARY KEY (article_id, topic_id)
);

CREATE INDEX idx_article_topics_topic ON article_topics (topic_id);

-- Seed initial topics.
INSERT INTO topics (name, slug) VALUES
    ('Philosophy', 'philosophy'),
    ('Metaphysics', 'metaphysics'),
    ('Epistemology', 'epistemology'),
    ('Ethics', 'ethics'),
    ('Aesthetics', 'aesthetics'),
    ('Logic', 'logic'),
    ('Political Philosophy', 'political-philosophy'),
    ('Philosophy of Mind', 'philosophy-of-mind'),
    ('Phenomenology', 'phenomenology'),
    ('German Idealism', 'german-idealism');

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

-- A snapshot quotation from a user-generated article.
-- Unlike book quotations (strict references), these copy the text
-- since articles are mutable. The article_id is nullable so the
-- quotation survives article deletion.
CREATE TABLE article_quotations (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id               UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    article_id            UUID REFERENCES articles(id) ON DELETE SET NULL,
    article_title         TEXT NOT NULL,
    author_display_name   TEXT NOT NULL,
    -- Snapshot of source author's bibliography sort key at save-time.
    -- Frozen with the rest of the snapshot (see chk_note_owner above).
    author_sort_name      TEXT,
    -- Snapshot of source article's published_at at save-time. Used for the
    -- year in bibliography entries; survives source-article deletion.
    source_published_at   TIMESTAMPTZ,
    text                  TEXT NOT NULL,
    html                  TEXT NOT NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_article_quotations_user ON article_quotations (user_id, created_at DESC);
CREATE INDEX idx_article_quotations_article ON article_quotations (article_id)
    WHERE article_id IS NOT NULL;

-- Notes attached to quotations. Plain text, private, one-to-many.
-- Polymorphic: exactly one of quotation_id / article_quotation_id must be set.
CREATE TABLE quotation_notes (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    quotation_id           UUID REFERENCES quotations(id) ON DELETE CASCADE,
    article_quotation_id   UUID REFERENCES article_quotations(id) ON DELETE CASCADE,
    body                   TEXT NOT NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_note_owner CHECK (
        (quotation_id IS NOT NULL AND article_quotation_id IS NULL)
        OR (quotation_id IS NULL AND article_quotation_id IS NOT NULL)
    )
);

CREATE INDEX idx_qnotes_quotation ON quotation_notes (quotation_id, created_at DESC)
    WHERE quotation_id IS NOT NULL;
CREATE INDEX idx_qnotes_article_quotation ON quotation_notes (article_quotation_id, created_at DESC)
    WHERE article_quotation_id IS NOT NULL;

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

-- ============================================================
-- FEEDBACK
-- ============================================================

-- Free transitions (any → any). Default 'todo' on submission;
-- admins flip to 'in_progress' / 'done' / 'cancelled' from the dashboard.
CREATE TYPE feedback_status AS ENUM ('todo', 'in_progress', 'done', 'cancelled');

-- User-submitted feedback (bug reports, suggestions, questions).
-- user_id is nullable so feedback survives user deletion (we render
-- "User deleted" in the admin UI). admin_notes is a single editable
-- scratchpad — not a thread; if we later want a comment trail, add a
-- separate feedback_comments table.
CREATE TABLE feedback (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID REFERENCES users(id) ON DELETE SET NULL,
    body          TEXT NOT NULL,
    -- Captured at submit time. Full URL with query string; useful when
    -- the bug depends on filter/route parameters.
    url           TEXT,
    user_agent    TEXT,
    viewport_w    INT,
    viewport_h    INT,
    status        feedback_status NOT NULL DEFAULT 'todo',
    admin_notes   TEXT,
    -- Last admin to edit status / notes. Nullable until first triage.
    handled_by    UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Dashboard default sort.
CREATE INDEX idx_feedback_created_at ON feedback (created_at DESC);
-- Status-filtered list ("active" = todo + in_progress) is the default
-- admin view; index status + created_at to keep that query cheap.
CREATE INDEX idx_feedback_status_created_at ON feedback (status, created_at DESC);
-- Rate-limit lookup: "submissions by this user in the last 24h".
CREATE INDEX idx_feedback_user_created_at ON feedback (user_id, created_at DESC)
    WHERE user_id IS NOT NULL;

-- ============================================================
-- BILLING (Stripe subscriptions)
-- ============================================================

-- Mirror of the user's current Stripe subscription. One row per
-- Stripe Subscription object; old canceled rows are kept for history.
-- The webhook handler is the only writer (refetches from Stripe API
-- before each upsert to defeat out-of-order delivery).
--
-- `tier` mirrors which Price the sub is on; mapped from the price ID
-- via env-var configuration (STRIPE_PRICE_BASE / _MID / _HIGH).
-- Backend currently treats all tiers identically (binary "active or
-- not"); the tier name only drives the public profile chip.
CREATE TABLE subscriptions (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                 UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stripe_subscription_id  TEXT NOT NULL UNIQUE,
    stripe_price_id         TEXT NOT NULL,
    -- 'base' | 'mid' | 'high' — derived from price ID at write time.
    tier                    TEXT NOT NULL,
    -- Mirrors Stripe: 'active', 'trialing', 'past_due', 'canceled',
    -- 'unpaid', 'incomplete', 'incomplete_expired', 'paused'.
    status                  TEXT NOT NULL,
    current_period_end      TIMESTAMPTZ NOT NULL,
    cancel_at_period_end    BOOLEAN NOT NULL DEFAULT false,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_subscriptions_user ON subscriptions (user_id);

-- Enforce: a user can have at most one "live" sub at any time.
-- Live = anything that grants access (active/trialing) or is in the
-- recoverable-failure window (past_due). Canceled/unpaid rows don't
-- count, so a user can re-subscribe freely after cancellation.
CREATE UNIQUE INDEX idx_subscriptions_user_live
    ON subscriptions (user_id)
    WHERE status IN ('active', 'trialing', 'past_due');

-- Webhook idempotency: every Stripe event ID we've already processed.
-- INSERT ... ON CONFLICT DO NOTHING is the dedup gate. Grows
-- unbounded for v1; revisit retention policy at >100k rows.
CREATE TABLE stripe_processed_events (
    event_id     TEXT PRIMARY KEY,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

