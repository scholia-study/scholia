-- Figure quotations: an article quotation may snapshot an uploaded figure
-- instead of a text selection. The figure fields are extracted server-side
-- from the quoted article's rendered HTML (never client-supplied), so the
-- snapshot is exactly what the article showed at quote time. Media objects
-- are never deleted, so figure_src stays resolvable even after the source
-- article drops or loses the figure.
CREATE TYPE article_quotation_kind AS ENUM ('text', 'figure');

ALTER TABLE article_quotations
    ADD COLUMN kind article_quotation_kind NOT NULL DEFAULT 'text',
    ADD COLUMN figure_src TEXT,
    ADD COLUMN figure_alt TEXT,
    ADD COLUMN figure_caption TEXT,
    ADD COLUMN figure_width INTEGER,
    ADD COLUMN figure_height INTEGER,
    ADD CONSTRAINT chk_figure_src_matches_kind
        CHECK ((kind = 'figure') = (figure_src IS NOT NULL));
