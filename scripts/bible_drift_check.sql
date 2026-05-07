-- Cross-translation verse-count drift report.
-- Run after importing all Bible translations to surface chapters where
-- verse counts disagree across translations. Use the output to decide
-- whether to extend the doxology guard in list_quotations_for_node.
--
-- Usage:
--   psql "$DATABASE_URL" -f scripts/bible_drift_check.sql

WITH chapter_verses AS (
    SELECT
        b.slug AS book_slug,
        tn.source_ref,
        COUNT(DISTINCT pm.ref_value) AS verse_count
    FROM toc_nodes tn
    JOIN books b ON b.id = tn.book_id
    JOIN sentences s ON s.node_id = tn.id
    JOIN page_markers pm ON pm.sentence_id = s.id
    JOIN reference_systems rs ON rs.id = pm.system_id
    WHERE rs.slug = 'verse' AND tn.depth = 1
    GROUP BY b.slug, tn.source_ref
),
pivoted AS (
    SELECT
        source_ref,
        MAX(CASE WHEN book_slug = 'kjv-bible'   THEN verse_count END) AS kjv,
        MAX(CASE WHEN book_slug = 'web-bible'   THEN verse_count END) AS web,
        MAX(CASE WHEN book_slug = 'asv-bible'   THEN verse_count END) AS asv,
        MAX(CASE WHEN book_slug = 'bbe-bible'   THEN verse_count END) AS bbe,
        MAX(CASE WHEN book_slug = 'darby-bible' THEN verse_count END) AS darby
    FROM chapter_verses
    GROUP BY source_ref
)
SELECT
    source_ref AS chapter,
    kjv, web, asv, bbe, darby,
    -- min/max of present (non-null) counts
    LEAST(
        COALESCE(kjv, 99999), COALESCE(web, 99999), COALESCE(asv, 99999),
        COALESCE(bbe, 99999), COALESCE(darby, 99999)
    ) AS min_count,
    GREATEST(
        COALESCE(kjv, 0), COALESCE(web, 0), COALESCE(asv, 0),
        COALESCE(bbe, 0), COALESCE(darby, 0)
    ) AS max_count
FROM pivoted
WHERE
    -- a chapter "drifts" if any pair of present counts disagrees
    (kjv   IS NOT NULL AND web   IS NOT NULL AND kjv   <> web)
 OR (kjv   IS NOT NULL AND asv   IS NOT NULL AND kjv   <> asv)
 OR (kjv   IS NOT NULL AND bbe   IS NOT NULL AND kjv   <> bbe)
 OR (kjv   IS NOT NULL AND darby IS NOT NULL AND kjv   <> darby)
 OR (web   IS NOT NULL AND asv   IS NOT NULL AND web   <> asv)
 OR (web   IS NOT NULL AND bbe   IS NOT NULL AND web   <> bbe)
 OR (web   IS NOT NULL AND darby IS NOT NULL AND web   <> darby)
 OR (asv   IS NOT NULL AND bbe   IS NOT NULL AND asv   <> bbe)
 OR (asv   IS NOT NULL AND darby IS NOT NULL AND asv   <> darby)
 OR (bbe   IS NOT NULL AND darby IS NOT NULL AND bbe   <> darby)
ORDER BY source_ref;
