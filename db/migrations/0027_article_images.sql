-- Images uploaded by users for embedding in articles via the ::figure{}
-- directive. Rows are per-user (not per-article): uploads happen mid-edit
-- before any save, and an image may be reused across a user's articles.
-- storage_key is the logical object key (facsimile_pages precedent),
-- content-addressed so a duplicate upload resolves to the existing object.
CREATE TABLE article_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    storage_key TEXT NOT NULL UNIQUE,
    original_filename TEXT,
    content_type TEXT NOT NULL,
    byte_size INTEGER NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX article_images_user_id_idx ON article_images (user_id);
