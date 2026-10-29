-- Per-book body-text length (characters) for the library index's shelf
-- spines. Aggregating LENGTH(text) over content_blocks on every library
-- request detoasted the whole table; recompute_node_sizes — which every
-- importer already calls before committing — now maintains it instead.
-- This saves on heavy computations on uncached loads.

ALTER TABLE books
    ADD COLUMN text_length BIGINT NOT NULL DEFAULT 0;

CREATE OR REPLACE FUNCTION recompute_node_sizes(p_book_id UUID) RETURNS void
LANGUAGE sql AS $$
    UPDATE toc_nodes tn
    SET content_size = (COALESCE(b.bytes, 0) + COALESCE(s.bytes, 0))::INTEGER,
        original_size = (COALESCE(b.original_bytes, 0) + COALESCE(s.original_bytes, 0))::INTEGER
    FROM toc_nodes t
    LEFT JOIN (
        SELECT node_id,
               SUM(octet_length(html) + 150) AS bytes,
               SUM(COALESCE(octet_length(original_html), 0)) AS original_bytes
        FROM content_blocks
        WHERE book_id = p_book_id
        GROUP BY node_id
    ) b ON b.node_id = t.id
    LEFT JOIN (
        SELECT node_id,
               SUM(octet_length(text) + octet_length(html) + 200) AS bytes,
               SUM(COALESCE(octet_length(original_text), 0)
                   + COALESCE(octet_length(original_html), 0)) AS original_bytes
        FROM sentences
        WHERE book_id = p_book_id
        GROUP BY node_id
    ) s ON s.node_id = t.id
    WHERE tn.id = t.id AND t.book_id = p_book_id;

    UPDATE books
    SET text_length = (
        SELECT COALESCE(SUM(LENGTH(text)), 0)
        FROM content_blocks
        WHERE book_id = p_book_id
    )
    WHERE id = p_book_id;
$$;

SELECT recompute_node_sizes(id) FROM books;
