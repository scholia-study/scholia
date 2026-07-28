-- Article review workflow: authors submit an article (draft or published)
-- to the editorial team, either for feedback or as a publication hand-off.
-- Each request freezes a snapshot of the article; editor comments anchor to
-- sentence ranges within that immutable snapshot, so anchors never drift.
-- A per-article message channel is shared across review rounds.
--
-- Approving a publication-intent request publishes the article (if still a
-- draft) and applies the 'imprimatur' editorial label seeded below; the
-- existing revokes_on_edit machinery then handles badge revocation and
-- re-submission rounds.

CREATE TYPE article_review_intent AS ENUM ('feedback', 'publication');

CREATE TYPE article_review_request_status AS ENUM
    ('pending', 'approved', 'declined', 'resolved', 'withdrawn');

CREATE TABLE article_review_requests (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id        UUID NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    intent            article_review_intent NOT NULL,
    status            article_review_request_status NOT NULL DEFAULT 'pending',
    snapshot_markdown TEXT NOT NULL,
    snapshot_html     TEXT NOT NULL,
    submitted_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_by       UUID REFERENCES users(id) ON DELETE SET NULL,
    resolved_at       TIMESTAMPTZ,
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uq_article_review_requests_one_pending
    ON article_review_requests (article_id) WHERE status = 'pending';
CREATE INDEX idx_article_review_requests_queue
    ON article_review_requests (submitted_at) WHERE status = 'pending';
CREATE INDEX idx_article_review_requests_article
    ON article_review_requests (article_id, submitted_at DESC);

-- Per-article channel between the author and the editorial team, shared
-- across review rounds. Comes into existence with the first request.
CREATE TABLE article_review_messages (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    -- NULL sender = deleted account; the message itself is kept.
    sender_id  UUID REFERENCES users(id) ON DELETE SET NULL,
    body       TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_article_review_messages_channel
    ON article_review_messages (article_id, created_at);

-- Anchored comment threads on one request's snapshot. Top-level comments
-- carry the anchor (block index, optional sentence range within the
-- block); replies inherit it from their parent. A NULL sentence range
-- means a block-level comment (quotation embeds, figures).
CREATE TABLE article_review_comments (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id     UUID NOT NULL REFERENCES article_review_requests(id) ON DELETE CASCADE,
    parent_id      UUID REFERENCES article_review_comments(id) ON DELETE CASCADE,
    sender_id      UUID REFERENCES users(id) ON DELETE SET NULL,
    block_index    INT,
    sentence_start INT,
    sentence_end   INT,
    quoted_text    TEXT,
    body           TEXT NOT NULL,
    resolved_at    TIMESTAMPTZ,
    resolved_by    UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_article_review_comment_anchor
        CHECK ((parent_id IS NULL) = (block_index IS NOT NULL)),
    CONSTRAINT chk_article_review_comment_range
        CHECK ((sentence_start IS NULL) = (sentence_end IS NULL))
);

CREATE INDEX idx_article_review_comments_request
    ON article_review_comments (request_id, created_at);

INSERT INTO editorial_labels (name, slug, description, revokes_on_edit, sort_order)
VALUES ('Imprimatur', 'imprimatur',
        '"Let it be printed" — reviewed and approved by a Scholia editor. Revoked when the author edits the article.',
        true, 15);
