-- Community source submissions. Authenticated non-editors can suggest
-- resources (verbatim/paraphrase/allusion); editors review them before they
-- go live. Editor-created resources keep landing as 'approved' (the column
-- default), so existing rows and the editor flow are unchanged.

CREATE TYPE resource_review_status AS ENUM ('pending', 'approved', 'rejected');

ALTER TABLE resources
    ADD COLUMN review_status resource_review_status NOT NULL DEFAULT 'approved',
    ADD COLUMN submitted_by  UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN reviewed_by   UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN reviewed_at   TIMESTAMPTZ,
    ADD COLUMN review_note   TEXT;

-- The review queue reads only the pending, non-archived rows; index just those.
CREATE INDEX idx_resources_pending ON resources (created_at)
    WHERE review_status = 'pending' AND archived_at IS NULL;
