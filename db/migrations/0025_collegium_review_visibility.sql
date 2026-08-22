CREATE TYPE collegium_review_visibility AS ENUM ('members', 'stewards');

ALTER TABLE collegia
    ADD COLUMN review_visibility collegium_review_visibility NOT NULL DEFAULT 'members';

ALTER TABLE article_review_requests ADD COLUMN member_visible BOOLEAN;

UPDATE article_review_requests SET member_visible = true WHERE collegium_id IS NOT NULL;

ALTER TABLE article_review_requests
    ADD CONSTRAINT chk_member_visible_collegium_only
        CHECK ((collegium_id IS NULL) = (member_visible IS NULL));
