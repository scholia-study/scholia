# Bible — Romans doxology placement drift (KJV vs WEB)

## Summary

The KJV and WEB translations of the Bible disagree on where the Romans
doxology (the "to him that is of power to stablish you…" passage) sits.
Of 1189 chapters across both translations, **2 chapters drift** in
verse count:

| Chapter      | KJV verses | WEB verses | Difference |
| ------------ | ---------- | ---------- | ---------- |
| `romans:14`  | 23         | 26         | WEB +3     |
| `romans:16`  | 27         | 25         | WEB −2     |

This is not a data error — it reflects the well-known difference between
the **Textus Receptus** (which KJV follows; doxology at the end of
Romans 16) and the **critical text** (which WEB follows; doxology
moved to the end of Romans 14). Net content is identical; only the
chapter/verse numbering differs for those six verses (KJV 16:25–27 ≡
WEB 14:24–26).

The remaining 1187 chapters (99.83%) align verse-for-verse.

## Why it matters: cross-translation projection

The reader projects quotations across translations using the
`(source_ref, verse_ref_value)` tuple as the match key (see
`web/src/modules/reader/context/Quotations.tsx`,
`isSentenceSaved`). When the same `(chapter, verse_number)` key
points to **different content** in the other translation, the
projection marker would land on the wrong verse — a *misleading*
hint, not a missing one.

Concretely, before the guard:

- A quote on **KJV Rom 16:25** (doxology start) viewed in WEB Rom 16
  would mark **WEB Rom 16:25** — but that's a benediction, not the
  doxology.
- The reverse case (WEB → KJV Rom 16) has the same problem in
  the other direction.
- Rom 14 cases mostly degraded silently (KJV Rom 14 has no verse 24,
  so a WEB Rom 14:24 quote simply finds no marker target in KJV).

## Short-term fix (in tree)

`packages/api/src/db/quotations.rs` — `list_quotations_for_node` adds
a guard that suppresses **cross-translation** projection rows when the
anchor's `source_ref` is one of `romans:14` or `romans:16`. Same-book
rows are always included — saving and viewing a quote within its own
translation is unaffected.

The trade-off is intentional: those two chapters now behave exactly
like the doxology already did in Rom 14 — a silent miss across
translations rather than a misleading hint. Quotations themselves
are unchanged; only the visual *projection* is suppressed for those
two chapters.

The importer (`packages/bible_to_db/src/main.rs`) emits a
`parity_warnings` count at the end of each translation import, so
any future drift surfaces during import rather than at runtime.

## Verification

The drift is reproducible against the live DB:

```sql
WITH chapter_verses AS (
    SELECT b.slug AS book_slug,
           tn.source_ref,
           COUNT(DISTINCT pm.ref_value) AS verse_count
    FROM toc_nodes tn
    JOIN books b ON b.id = tn.book_id
    JOIN sentences s ON s.node_id = tn.id
    JOIN page_markers pm ON pm.sentence_id = s.id
    JOIN reference_systems rs ON rs.id = pm.system_id
    WHERE rs.slug = 'verse' AND tn.depth = 1
    GROUP BY b.slug, tn.source_ref
)
SELECT k.source_ref AS chapter,
       k.verse_count AS kjv,
       w.verse_count AS web,
       (w.verse_count - k.verse_count) AS diff
FROM chapter_verses k
JOIN chapter_verses w ON w.source_ref = k.source_ref
WHERE k.book_slug = 'kjv-bible'
  AND w.book_slug = 'web-bible'
  AND k.verse_count <> w.verse_count;
```

If a future translation is imported and produces drift in *new*
chapters, the SQL above will show them and the importer's
`parity_warnings` count will be non-zero. Add the new chapters to the
guard's `IN (...)` list and update the table at the top of this doc.

## Long-term solution

The hardcoded `IN ('romans:14', 'romans:16')` guard scales poorly:
adding a third translation (e.g. an LXX-derived OT or a
modern-language NT) will likely expose additional drift, and at some
point the guard list becomes unwieldy and *still* doesn't help users
who *want* the projection to follow the content rather than the
chapter:verse coordinates.

The proper fix is a **verse alignment table**, e.g.:

```sql
CREATE TABLE verse_alignments (
    work_root_id    UUID NOT NULL,                   -- the canonical "Bible" source
    a_translation   UUID NOT NULL,                   -- e.g. KJV source
    a_source_ref    TEXT NOT NULL,                   -- e.g. 'romans:16'
    a_verse         TEXT NOT NULL,                   -- e.g. '25'
    b_translation   UUID NOT NULL,                   -- e.g. WEB source
    b_source_ref    TEXT NOT NULL,                   -- e.g. 'romans:14'
    b_verse         TEXT NOT NULL,                   -- e.g. '24'
    PRIMARY KEY (work_root_id, a_translation, a_source_ref, a_verse,
                 b_translation, b_source_ref, b_verse)
);
```

`list_quotations_for_node` would consult this table to translate the
target's `(source_ref, verse)` into the peer translation's coordinates
before matching. Six rows would cover the doxology bidirectionally.
The same mechanism handles any future textual-tradition divergence.

This becomes worth building when one of:

- A third translation is imported and exposes more drift cases.
- A user reports the missing projection on the doxology specifically.
- A broader projection rework is on the roadmap (e.g. the deferred
  Q9 selection-carry, or per-translation quotation re-anchoring).

Until then, the guard + this doc is the sanctioned trade-off:
honest silent-miss behavior for the two known-drift chapters,
zero risk of misleading hints anywhere.
