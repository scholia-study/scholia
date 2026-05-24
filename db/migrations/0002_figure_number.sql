-- Sequential per-book figure number, used to build a stable human-readable
-- selection key (e.g. "fig1") instead of exposing the anchor sentence UUID.
-- Set only for block_type = 'figure'.

ALTER TABLE content_blocks ADD COLUMN figure_number INT;

CREATE UNIQUE INDEX idx_blocks_figure_num
    ON content_blocks (book_id, figure_number)
    WHERE figure_number IS NOT NULL;
