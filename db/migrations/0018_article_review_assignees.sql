-- Who on the editorial team owns a review request. Any editor may
-- assign themselves or a colleague; NULL = unclaimed.
ALTER TABLE article_review_requests
    ADD COLUMN assigned_to UUID REFERENCES users(id) ON DELETE SET NULL;
