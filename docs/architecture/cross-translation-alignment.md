# Cross-translation alignment

## Purpose

Quotation projection across translations needs to know **where the
same content lives** in each translation. For most chapters that's
trivial — KJV Romans 1:5 ≡ WEB Romans 1:5 ≡ ASV Romans 1:5 by chapter
and verse. But translations occasionally disagree on numbering, and
when they do, projection has to follow the *content*, not the
coordinates.

The `cross_translation_alignments` table maps each translation's
local `(source_ref, ref_value)` coordinates to the work's canonical
coordinates. Identity is the implicit default; only deviations are
stored.

## Data model

```sql
CREATE TABLE cross_translation_alignments (
    book_id              UUID NOT NULL REFERENCES books(id),
    system_slug          TEXT NOT NULL,         -- 'verse' | 'page' | …
    source_ref           TEXT NOT NULL,
    local_ref_value      TEXT NOT NULL,
    canonical_source_ref TEXT,
    canonical_ref_value  TEXT,
    PRIMARY KEY (book_id, system_slug, source_ref, local_ref_value),
    CHECK ((canonical_source_ref IS NULL) = (canonical_ref_value IS NULL))
);
```

A row can mean one of three things:

| Lookup result | Meaning |
| --- | --- |
| No row | **Identity** — peer's coords ARE canonical. Most rows in the corpus. |
| Row, both canonical fields filled | **Mapped** — coords map elsewhere (`WEB Rom 14:24 → canonical Rom 16:25`). |
| Row, both canonical fields NULL | **Translation-only** — verse exists in this translation but has no canonical equivalent (DARBY's Hebrew superscriptions). |

The CHECK constraint forbids the half-state.

`system_slug` parallels the per-book `reference_systems.slug` (e.g.
`'verse'` for Bible, `'page'` for paginated editions). It's a free-text
column rather than an FK because reference_systems are scoped per
book; the slug alone is enough to disambiguate.

## Current Bible drift cases

Five public-domain English Bibles (KJV, WEB, ASV, BBE, DARBY) agree on
1142 of 1189 chapters verse-for-verse. Two outliers produce all the
drift:

### WEB — Romans doxology (3 alignment rows)

WEB places the Romans doxology ("to him that is of power to stablish
you…") at the end of chapter 14 instead of chapter 16. Net content is
identical; KJV's Rom 16:25–27 lives at WEB Rom 14:24–26.

```text
WEB romans:14 / 14:24 → canonical romans:16 / 16:25
WEB romans:14 / 14:25 → canonical romans:16 / 16:26
WEB romans:14 / 14:26 → canonical romans:16 / 16:27
```

### DARBY — Hebrew Psalm superscriptions + 2 Kings 11 (~833 alignment rows)

DARBY treats the Hebrew superscriptions ("To the chief Musician. A
Psalm of David…") as numbered verses, shifting every subsequent verse
by +1 (or +2 in the rare case of two-line titles, e.g. Ps 50, Ps 51).
Affected chapters: 44 Psalms (`2, 5, 6, 7, 8, 17, 18, 20, 21, 30, 33,
35, 37, 38, 40, 41, 44, 45, 46, 47, 48, 50, 51, 52, 55, 56, 57, 58,
60, 62, 63, 64, 66, 68, 74, 75, 76, 80, 83, 84, 88, 91, 107, 141`)
plus `2kings:11`.

For each affected chapter, rows are generated programmatically:

- The leading title verse(s) — DARBY `local_ref_value = "{ch}:1"` and
  optionally `"{ch}:2"` — get rows with NULL canonical (translation-only).
- The remaining verses — DARBY `local_ref_value = "{ch}:{m + shift}"`
  for `m in 1..N` — map to canonical `"{ch}:{m}"` where `N` is the
  canonical chapter's verse count and `shift = darby_count -
  canonical_count`.

Total: 836 rows for DARBY (varies by ±1 if title-verse counts change).

### Clean clusters

KJV, ASV, and BBE align perfectly with each other and with canonical
on every chapter — they have **zero** alignment rows. WEB joins this
cluster except for the 3 Romans rows. DARBY joins it except for the
~833 Hebrew-superscription rows.

## Where alignment is applied

`packages/api/src/db/quotations.rs` — `list_quotations_for_node`. The
query has two CTEs and joins through alignments to compute
target-local projection coordinates for each peer quote:

1. **`target_verses`** — for each verse marker in the target chapter,
   resolve to canonical coords. Identity if no alignment row; the
   row's canonical coords otherwise.
2. **Per-quote canonical resolution** — for the quote's start/end
   verses, resolve to canonical via the peer book's alignment row (or
   identity).
3. **Match** — peer quote projects onto target if its canonical
   coords match any target verse's canonical coords. The matching
   target verse's `local_ref` becomes `projected_verse_start/end`.

For same-book quotes the query short-circuits to identity (no
alignment lookup). This keeps Kant — which has no verse markers and
no alignment rows — working through its existing source_ref equality
path.

The response carries both the original anchor coords (`anchor_*`,
used for badges and source links) and the target-local projection
coords (`projected_*`, used to place markers). For non-drifting
chapters they coincide.

## Importer seeding

`packages/bible_to_db/src/main.rs` — `seed_cross_translation_alignments`
runs after each translation imports, before commit. The dispatch is
explicit per slug: KJV/ASV/BBE return zero rows; WEB inserts the 3
Romans rows; DARBY enumerates the Hebrew-title chapters and computes
the shift dynamically from each chapter's verse counts.

The shift is computed (not assumed `+1`) because some chapters have
two-line superscriptions (Ps 50, Ps 51).

## Verification

The full per-translation drift report:

```bash
psql "$DATABASE_URL" -f scripts/bible_drift_check.sql
```

Spot-check a single chapter's alignments:

```sql
SELECT cta.local_ref_value, cta.canonical_source_ref, cta.canonical_ref_value
FROM cross_translation_alignments cta
JOIN books b ON b.id = cta.book_id
WHERE b.slug = 'darby-bible' AND cta.source_ref = 'psalms:51'
ORDER BY cta.local_ref_value;
```

## Adding a new translation

1. Add the translation to `TRANSLATIONS` in `bible_to_db/src/main.rs`.
2. Cache its chapters via `scripts/bible_fetch.sh`.
3. Import. The parity-warnings count at the end of import will tell
   you which chapters drift relative to canonical (KJV).
4. If any chapters drift, write a new alignment rule (analogous to
   `seed_web_romans_doxology` or `seed_darby_hebrew_titles`) and add
   a match arm in `seed_cross_translation_alignments`. Re-import.
5. Verify via the drift-check SQL — drift count should match number
   of chapters covered by the new rule.

## Adding a new reference system

Currently the seed and query both use `system_slug = 'verse'`. To
support page-level alignment (e.g. Kant A↔B editions if you ever
wanted it), you'd:

1. Add the system to `reference_systems` per book (already supported).
2. Insert alignment rows with `system_slug = 'page'` (or whatever the
   slug is).
3. The query in `list_quotations_for_node` is currently hardcoded to
   filter on `'verse'` — generalize it to use the target's available
   reference systems, or run separate queries per system. Out of
   scope for v1.
