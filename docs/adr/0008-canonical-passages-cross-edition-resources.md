# 0008. Canonical passages: resources project across editions

**Status**: Accepted
**Date**: 2026-07-27

## Context

Resources (verbatim / paraphrase / allusion commentary) anchor to a
sentence range in **one book** and are listed with `WHERE r.book_id =
$1`. A verbatim source pointed at English Kant sentence 200 is invisible
on the German page — and on every sibling Bible translation. With the
community-submission flow now widening who catalogues resources, that
edition-siloing becomes systematic incompleteness: the same intellectual
reference lands on whichever edition the cataloguer happened to be
reading, and readers of every other edition silently miss it.

The corpus already carries two cross-edition alignment mechanisms, at
**different granularities**:

- **Sentence-locked works** (Kant, poetry, drama): the translation's
  sentences carry exact 1:1 `source_sentence_start_id` links to the
  source edition (verified complete: e.g. 4652/4652 body + 198/198
  footnote sentences on `critique-of-pure-reason-b`). The source edition
  *is* the canonical layer.
- **Bibles** (5 translations, no source edition among them): zero
  sentence links; alignment is by **verse coordinate** via
  `reference_systems('verse')` + `cross_translation_alignments`
  (see `docs/architecture/cross-translation-alignment.md`). Sentences
  split differently per translation *within* a verse, so verse is the
  finest shared unit.

Both families share one work-identity key:
`work_root = COALESCE(sources.translation_of_id, sources.id)` (English
Kant → German source; all 5 Bibles → a common root). Quotation
projection exists only for the verse mechanism; nothing projects for
sentence-locked works, and resources project nowhere.

Some resources are genuinely *about one edition's text* — a verbatim
quote captures one edition's exact wording; translation-commentary
("Kemp Smith renders *Anschauung* as 'intuition' here") is a claim about
the English rendering, not the passage. And the edition axis is not the
only one: a work may grow **multiple editions per language** (a second
English Kant). Those editions want to be "on the same page" — a resource
about the English translation layer applies to all of them — while still
differing from the German. A reader must be able to tell *this is from
another language*, not merely *another edition*.

User articles quoting passages already have cross-edition reach:
`list_article_references` matches same-book by overlap, sentence-locked
works by `natural_key` equality, and Bibles by canonicalized verse
intersection — so an article quoting the German Kant already surfaces in
the English edition's Articles pane. What articles lack is *indication*
(the pane doesn't say the references point at another edition/language).

## Decision

1. **New table `canonical_passages`** — an edition-independent passage
   identity per work, plus `sentences.canonical_passage_id`. Sibling
   editions' sentences that carry the same content share one row.
   Resources keep anchoring to the edition sentence they were created on;
   projection is resolved **at read time** through the shared passage.

2. **Granularity is dictated by the family's data, not chosen.** A
   `basis` column records which mechanism minted the row:
   - `'sentence'` — member sentences carry `source_sentence_start_id`.
     One passage per **source-edition sentence**; translations stamp the
     linked passage. Sentence-exact projection.
   - `'verse'` — no sentence links, but a shared `verse` reference
     system. One passage per **canonical verse**, resolved through
     `cross_translation_alignments` (identity when no row; mapped rows
     follow the row; translation-only rows — NULL canonical — stay
     unstamped and never project). Verse-granular projection.

   Sentence links win when both signals exist. Standalone works stamp
   nothing and never project.

3. **Everything projects; nothing is suppressed.** A new
   `resources.scope` enum — `'work' | 'language' | 'edition'`, default
   `'work'` — is the cataloguer's **claim** about what the resource is
   about: the passage in any form; this language's translation layer
   ("all English editions on the same page"); or this one edition's
   actual text. It is a label and filter key only; it never stops
   projection — a German reader still learns that an English
   edition-specific resource exists, marked as such.

   Display classification needs **three inputs**, of which the enum is
   one: `scope` (stored claim), the **origin book** (the existing
   `resources.book_id`, surfaced as origin slug + `books.language`), and
   the **viewer's book** (request context). The frontend derives:
   `origin.language ≠ viewer.language` → prominent language badge
   ("EN"/"DE"); `origin.book ≠ viewer.book` → edition note; `scope` →
   printed claim. The enum's `language` value never names a language —
   the origin anchor supplies it. No provenance or language column is
   added anywhere.

4. **Lifecycle stays on the origin edition.** Edit/delete only where the
   anchor lives (projected entries are read-only, deep-linking to the
   origin). Pending submissions never project; only `approved` resources
   do.

5. **Importers seed the mapping** (narrow-waist convention): the
   struct importer stamps sentence-basis passages in source and
   translation modes; the Bible importer mints verse-basis passages
   after `seed_cross_translation_alignments`. Existing rows backfill via
   routine re-ingest, not a data migration. Future editions of any
   language import with `--source-book-slug` pointing at the source
   edition, so N editions per language share the same root-minted
   passages with no schema or seeder change; language clusters are
   `books.language` grouped within `work_root`, derived at read time.

6. **Articles: deferred, noted.** Their matching already spans editions;
   what remains is surfacing origin edition/language per matched article
   in the Articles pane (same badge model as resources). Revisit after
   resource projection lands. Longer term, the three-branch article
   query — and quotation projection — can collapse onto the
   `canonical_passages` rail as the single projection mechanism.

## Reasoning

**Materialize vs compute.** Read-time computation (branching per corpus
inside `list_resources` — a source-link join for Kant, the verse CTEs
for Bibles) needs no migration but bakes two divergent mechanisms into
every read forever — the exact thing a canonical layer should kill. The
table gives one uniform rail: the seeder absorbs per-family complexity
once, at import time, where each mechanism already lives. Quotations
could later ride the same rail (finally giving sentence-locked works
quotation projection, which the verse-keyed query cannot).

**Why not anchor resources to canonical passages directly?** Re-homing
the anchor would lose provenance — *which edition's wording* a verbatim
captured — and force every existing anchor through a rewrite. Keeping
the edition anchor + read-time resolution preserves wording, provenance,
and the entire existing create/reconcile path (canonical stamping rides
on sentence rows, which reconcile-in-place already keeps stable).

**Label, not suppression.** An earlier design made the edition claim
suppress projection. Rejected: hiding the most common type (verbatim)
from sibling editions re-creates the incompleteness this exists to fix.
The reference ("source X engages this passage") is valuable to every
edition's reader; the wording caveat is handled by the origin badge.
Noise is a display concern → frontend filter (which can key on language
as the coarse switch), not data loss.

**Scope as enum, not boolean.** A bare `edition_specific` flag collapses
the language layer: with multiple English editions, "about the English
translation" (applies to every English edition) and "about this English
edition's text" are different claims, and a German reader cares about
the distinction. Three values cover the hierarchy work → language →
edition; anything finer belongs in the resource's note text.

**Accepted asymmetry.** Bible projection is verse-granular: a resource
pinned to one sentence inside a multi-sentence verse projects onto the
whole corresponding verse elsewhere — verse is the finest thing Bibles
share. Kant stays sentence-exact. Inherent, not a flaw. Likewise
DARBY's 47 translation-only title verses correctly keep their resources
edition-local (no peer content exists).

## Schema deltas

One append-only migration (next is `0016`):

```sql
CREATE TYPE canonical_basis AS ENUM ('sentence', 'verse');

CREATE TABLE canonical_passages (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_root        UUID NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    basis            canonical_basis NOT NULL,
    sentence_kind    sentence_kind NOT NULL DEFAULT 'body',
    ordinal          INT NOT NULL,
    root_sentence_id UUID REFERENCES sentences(id) ON DELETE CASCADE,
    source_ref       TEXT,
    ref_value        TEXT,
    CHECK ((basis = 'sentence') = (root_sentence_id IS NOT NULL)),
    CHECK ((basis = 'verse') = (source_ref IS NOT NULL AND ref_value IS NOT NULL))
);

ALTER TABLE sentences ADD COLUMN canonical_passage_id UUID
    REFERENCES canonical_passages(id) ON DELETE SET NULL;

CREATE TYPE resource_scope AS ENUM ('work', 'language', 'edition');
ALTER TABLE resources ADD COLUMN scope resource_scope NOT NULL DEFAULT 'work';
```

`ordinal` totally orders passages within `(work_root, sentence_kind)`
so anchor ranges can be overlap-tested: sentence basis reuses the source
edition's `sentence_number`; verse basis is minted in canonical (KJV)
traversal order. Partial unique indexes on `root_sentence_id`
(sentence basis) and `(work_root, source_ref, ref_value)` (verse basis)
make seeding idempotent. A NULL `canonical_passage_id` simply never
projects — the same-book read path doesn't touch it.

The read path (`list_resources`) keeps its same-book branch byte-for-byte
(including own-pending visibility) and adds a peer branch keyed on
canonical-ordinal overlap, returning `is_projected`, origin book slug +
language, `scope`, and target-local placement coordinates.
