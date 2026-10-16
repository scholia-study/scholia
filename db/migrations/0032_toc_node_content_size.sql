-- Per-node payload weight for the reader's byte-budgeted page window
-- (GET /api/books/{slug}/nodes). The page endpoint fills a window up to a
-- byte budget instead of a fixed node count, so a book of whole-essay nodes
-- (peirce1, Hegel) no longer ships its entire text in one response.

ALTER TABLE toc_nodes
    ADD COLUMN content_size INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN original_size INTEGER NOT NULL DEFAULT 0;

CREATE FUNCTION recompute_node_sizes(p_book_id UUID) RETURNS void
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
$$;

SELECT recompute_node_sizes(id) FROM books;
