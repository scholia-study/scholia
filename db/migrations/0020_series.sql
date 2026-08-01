-- Article series: editor-curated, ordered collections of published
-- articles — a third grouping mechanism beside author-picked topics and
-- editor-applied editorial labels. The Scholia Blog is simply a series;
-- pinned series surface as shelves on the articles landing page.
--
-- Membership is many-to-many with an app-managed position (adding an
-- article prepends it at position 1; display is always position-asc, so
-- the blog stays newest-first with zero maintenance). Attach is
-- published-only, but membership survives unpublish: public surfaces
-- filter to published members while the manage UI greys the rest.
-- Series deletion is refused in the app while members remain, mirroring
-- topics; the CASCADE FKs are the backstop, not the policy.

CREATE TABLE series (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL UNIQUE,
    slug        TEXT NOT NULL UNIQUE,
    description TEXT,
    pinned      BOOLEAN NOT NULL DEFAULT false,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE series_articles (
    series_id  UUID NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    article_id UUID NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    position   INTEGER NOT NULL,
    added_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (series_id, article_id)
);

CREATE INDEX idx_series_articles_position ON series_articles (series_id, position);
CREATE INDEX idx_series_articles_article  ON series_articles (article_id);

-- Seed the blog so the hardcoded frontend link resolves in every
-- environment, including fresh dev databases. Not pinned — promoting
-- it to a landing-page shelf is an editorial decision made in the UI.
INSERT INTO series (name, slug) VALUES ('Scholia Blog', 'scholia-blog');
