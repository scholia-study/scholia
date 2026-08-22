CREATE TYPE collegium_member_role AS ENUM ('steward', 'member');

CREATE TYPE collegium_join_request_status AS ENUM ('pending', 'approved', 'rejected');

CREATE TABLE collegia (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name         TEXT NOT NULL,
    slug         TEXT NOT NULL UNIQUE,
    description  TEXT,
    is_private   BOOLEAN NOT NULL DEFAULT false,
    invite_token TEXT UNIQUE,
    -- Attribution and quota ledger; kept on account deletion as NULL.
    created_by   UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at   TIMESTAMPTZ
);

CREATE INDEX idx_collegia_created_by ON collegia (created_by) WHERE created_by IS NOT NULL;
CREATE INDEX idx_collegia_discover ON collegia (name) WHERE deleted_at IS NULL AND NOT is_private;

CREATE TABLE collegium_members (
    collegium_id  UUID NOT NULL REFERENCES collegia(id) ON DELETE CASCADE,
    user_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role      collegium_member_role NOT NULL DEFAULT 'member',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (collegium_id, user_id)
);

CREATE INDEX idx_collegium_members_user ON collegium_members (user_id, joined_at DESC);

CREATE TABLE collegium_join_requests (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    collegium_id   UUID NOT NULL REFERENCES collegia(id) ON DELETE CASCADE,
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status     collegium_join_request_status NOT NULL DEFAULT 'pending',
    decided_by UUID REFERENCES users(id) ON DELETE SET NULL,
    decided_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uq_collegium_join_requests_one_pending
    ON collegium_join_requests (collegium_id, user_id) WHERE status = 'pending';
CREATE INDEX idx_collegium_join_requests_queue
    ON collegium_join_requests (collegium_id, created_at) WHERE status = 'pending';

-- Review audience: NULL = editorial team, otherwise the collegium whose
-- members act as reviewers. The one-pending-per-article unique index is
-- deliberately left global across audiences.
ALTER TABLE article_review_requests
    ADD COLUMN collegium_id UUID REFERENCES collegia(id);

ALTER TABLE article_review_requests
    ADD CONSTRAINT chk_collegium_review_feedback_only
        CHECK (collegium_id IS NULL OR intent = 'feedback');

CREATE INDEX idx_article_review_requests_collegium
    ON article_review_requests (collegium_id, submitted_at)
    WHERE collegium_id IS NOT NULL AND status = 'pending';

-- The message channel is scoped to the audience: one editorial channel
-- (collegium_id NULL) plus one channel per collegium, per article.
ALTER TABLE article_review_messages
    ADD COLUMN collegium_id UUID REFERENCES collegia(id);

CREATE INDEX idx_article_review_messages_collegium_channel
    ON article_review_messages (article_id, collegium_id, created_at);
