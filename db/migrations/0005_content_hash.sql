-- Tier-2 incremental reconcile: a two-level content hash (book root + per-node)
-- lets a reconciling re-import skip the content work for unchanged nodes and do
-- the positional renumber set-based instead of per-row. See
-- docs/architecture/reconcile-incremental-hashing.md.
--
-- Both nullable. NULL means "unknown — treat as changed", so the optimization is
-- fail-safe: a missing hash never causes a skip. The first reconcile after this
-- migration sees all-NULL and behaves like --full, backfilling every hash.
ALTER TABLE toc_nodes ADD COLUMN content_hash TEXT;
ALTER TABLE books     ADD COLUMN content_hash TEXT;
