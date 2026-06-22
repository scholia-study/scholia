-- Per-book reader pagination directive. `nodes_per_page` is how many reading
-- nodes the paginated node-page endpoint returns per next/prev fetch. NULL =
-- default (20), which suits texts made of many small nodes (Bible chapters,
-- sonnets, Kant sections). Texts made of a few enormous nodes — Paradise Lost,
-- where one node is a whole Book (~640–1290 lines) — set a small value so the
-- reader loads one Book at a time instead of the entire work at once.

ALTER TABLE books ADD COLUMN nodes_per_page SMALLINT;
