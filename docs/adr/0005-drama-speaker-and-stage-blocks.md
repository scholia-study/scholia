# 0005. Drama: speaker and stage blocks; page-cited dialogue

**Status**: Accepted
**Date**: 2026-06-30

## Context

Scholia adds its first **drama** — Ibsen's *Emperor and Galilean* (*Kejser
og Galilæer*, 1873), a two-layer text (`md_modernized` modern Bokmål +
`md_reviewed` faithful 1873). ADR 0003 (poetry) deferred drama and left a
seam: "speaker / stage-direction become future block types (or a sentence
attribute); act/scene/line becomes a reference system or composite
ref_value. Re-open this ADR if … Drama enters scope." That moment is now.

Drama must honour three reader behaviours: character names are
**non-clickable** (like headings); dialogue is **selectable
sentence-by-sentence** (like prose); a single speech can **mix prose and
verse** (a hymn inside a speech). The existing model already fits with a
small, additive change set, because `block_type` is a pass-through string
end to end (DB enum → importer bind → API `block_type` string → frontend
`string`) and headings already render non-clickable.

## Decision

1. **Speaker = its own non-clickable `speaker` block.** A speech is the
   implicit run: `speaker` block + following `paragraph`/`verse`/`stage`
   blocks until the next `speaker`/`heading`. No `speaker` *column* — a
   first-class speaker→lines link (character filtering, "all of Julian's
   speeches") is deferred; it is additive later. The speaker label (name +
   inline `*(opener)*`) is the block's single sentence.

2. **Stage directions = `stage` chrome.** `@stage (…)`, own-line `*(…)*`,
   and the dramatis-personae `<ul>` are all `stage` blocks, rendered
   italic/muted. *(Amended 2026-07-01 — stage directions are now
   selectable/quotable; see the Amendment below. The cast-list `<ul>` stays
   inert.)*

3. **Non-clickable = `sentence_number = NULL`.** `speaker`/`heading`
   sentences (and the cast-list `stage` block) are unnumbered, so they are
   excluded from the selection key and shift-click ranges (which ride on
   `sentence_number`); `paragraph`/`verse` dialogue is numbered. The parser
   controls this. *(Amended 2026-07-01 — stage-direction `stage` blocks are
   now numbered too; see the Amendment.)*

4. **Citation by page.** The `{{{ N }}}` first-edition markers become a
   `1873` page **reference system** — drama's *default* citation
   (`cite_priority = 0`), templated `"{parent}, {self} · p. {ref}"` →
   "Cæsars Frafald, Første handling · p. 12". Sentence ordinal (`s. N`) is
   the secondary label, riding the existing `sentence_number`. No
   act/scene/line system: acts are `toc_nodes`, there are no scenes, and
   the prose drama is not line-numbered.

5. **A structural, layer-consistent sentence splitter.** Dialogue is split
   by `common::sentences::split_sentences_structural` — punctuation-only,
   **case-insensitive**, no abbreviation/initial heuristics — *not*
   `split_sentences_en`. The English splitter requires a capital after
   `.!?`, so the 1873 layer's lower-case-after-`?` (e.g. `nogen? eller`,
   the period orthography) yields no split where the modernized `noen?
   Eller` does — desyncing the two layers' sentence counts (66 of ~1773
   speeches). The structural splitter splits both layers identically
   because they share the punctuation skeleton, so they pair
   sentence-for-sentence **without editing the faithful text**.

## Reasoning

A distinct `speaker`/`stage` block_type (vs. overloading `paragraph` with
a flag) is justified for the same reason `verse` (0003) and `figure`
(0001) were: the renderer and the selection layer must switch behaviour on
it (non-clickable, no `sentence_number`), and `block_type` is already that
switch. Two enum values, **zero new columns**.

Page-as-default-citation (not a line/scene system) matches how this prose
drama is actually cited in scholarship — by the first edition's page —
and reuses the `reference_systems` + `page_markers` machinery unchanged
(Bible verses, Kant A/B pages, poetry lines all work this way).

The structural splitter is the load-bearing correctness decision. The
alternative — reconciling the curated content so both layers split alike
under `split_sentences_en` — would mean *editing the faithful 1873 layer*
(capitalising words after `?`), corrupting the very fidelity that layer
exists to preserve. Moving the variance into a deterministic splitter
keeps both layers untouched and is reusable by any future two-layer text.

## Schema deltas

One append-only migration, **`0010_drama_blocks.sql`** (mirrors 0001
`figure` / 0006 `verse`):

```sql
ALTER TYPE block_type ADD VALUE IF NOT EXISTS 'speaker';
ALTER TYPE block_type ADD VALUE IF NOT EXISTS 'stage';
```

Safe in one transaction — neither statement *uses* the new value. No new
columns. The `1873` page system is **data** (`reference_systems` +
`page_markers`), not schema.

## Ingest

- New **`packages/drama_md_to_struct`** parser (sibling of
  `poetry_md_to_struct`): tokenises the drama markup into the shared
  `text_struct` schema (`block_type` free string → `speaker`/`stage` need no
  struct change), reusing `common::sentences` + `text_struct::html`.
- New **`common::ibsen1`** module: canonical TOC (book + the `cf` part
  title-page parenting cast + five acts), filenames, labels, and the
  `1873` page system config.
- **Reuses `packages/struct_to_db` unchanged** — it binds
  `block.block_type` to `::block_type` and takes `sentence_number` from the
  struct, importing `speaker`/`stage` with no code change.
- Per-act parity guard: modernized vs reviewed must share block sequence,
  block types, verse-line counts, and prose sentence counts (the structural
  splitter makes the last hold); page-marker sequence is preserved.
- **English translation edition** (`md_modernized_translated`): the same
  `drama_md_to_struct --translation` parses it single-layer (English → text,
  no original) into a separate book (`emperor-and-galilean`, dated to the
  present so it doesn't collide with the 1873 source on the `sources` unique
  key). `struct_to_db --source-book-slug keiser-og-galileer` imports
  it in **translation mode** (mirroring kant1): it links each node
  (`source_node_id`) and sentence (`source_sentence_start_id`) 1:1 to the
  source book by natural key, sets `translation_of_id` on its source, reuses
  the source's `1873` reference system, and validates that every block carries
  the same sentence count as the source. This drives the side-by-side companion
  view and cross-edition quotation projection. `pnpm db:ibsen1` imports the
  source then the translation in one go.

## API / Frontend

- **API: no code change.** `block_type` forwards any string; citation is
  driven by `reference_systems` data. No `pnpm codegen` needed.
- **Frontend:** two cases in `BlockRenderer` — `speaker` (flush-left,
  bold/uppercase, muted; routed through the non-clickable `HeadingSentence`)
  and `stage` (italic/muted block, cast-list `<ul>` styled). Dialogue
  (`paragraph`/`verse`) gets a conditional left indent (`inDrama`, derived
  from a node containing `speaker` blocks) so names form a scannable
  flush-left column. Selection/quotation untouched — they only ever see the
  numbered dialogue sentences.

## Consequences

- **No first-class speaker→lines association yet** (deferred `speaker`
  column) — character filtering is a future additive change.
- Part Two (*Kejser Julian*) is not yet modernized; it slots in as a second
  depth-0 part in `common::ibsen1` when ready, no schema change.
- The English translation edition is wired through the *reused*
  `struct_to_db` (now translation-capable, like `kant1_struct_to_db`),
  so a future poetry translation (e.g. a Milton modernization) gets the same
  path for free.
- **Post-acceptance rename (2026-07-01):** the generic schema + md→html that
  this reuse leaned on were extracted from `poetry_md_to_struct` into a
  neutrally named **`packages/text_struct`**, and the generic importer was
  renamed `poetry_struct_to_db` → **`packages/struct_to_db`** — the "poetry"
  names lied once a second genre (drama) depended on them. `struct_to_db` and
  the still-separate `kant1_struct_to_db` remain a **known duplication** (two
  near-identical node→block→sentence importers with reconcile + translation);
  folding them into one generic importer is a future additive change, deferred
  until it earns itself.

## Amendment (2026-07-01): stage directions are quotable

Decisions 2–3 treated **all** stage directions as inert chrome. In use this
proved wrong on two counts:

1. **Inline directions were glued to dialogue.** A `*(…)*` sitting *between*
   two sentences of a speech (`…at the stake. *(draws aside.)* Oh, let us…`)
   is not lifted to its own block — it flows inside the prose. The structural
   splitter breaks on `. ` but not on `.)·`, so the direction rode at the head
   of the *next* dialogue sentence and was swept into any selection/quotation
   of that spoken line.
2. **Directions weren't quotable at all.** A stage direction is authored
   dramatic text a scholar may cite; making it inert (like a speaker label)
   was the wrong altitude.

**Amended decision.** Stage directions are **quotable dramatic text**: each is
its own numbered, clickable sentence rendered muted/italic. This applies to
inline directions *and* standalone `stage` blocks. **Still inert:** speaker
labels, headings, and the dramatis-personae cast list.

**Implementation (no schema change — `sentence_number` was already nullable):**

- **Splitter (`common::sentences`).** `split_sentences_structural` is now
  **paren-aware**: it never places a boundary inside a `(…)` run, so a
  direction carrying its own sentence punctuation
  (`(…the lamp-bowl. The lamp lights itself…)`) stays one unit. (Sole caller is
  drama, so this is safe.)
- **Parser (`drama_md_to_struct`).** `prose_block` tags parenthetical emphasis
  (`<i>(…)</i>` → `<i class="stage">…</i>`, leaving ordinary `*word*` emphasis
  alone) and **peels** any sentence that *opens* with a direction into a
  standalone numbered direction sentence + the remaining dialogue. A
  between-sentence direction opens a sentence after splitting (so it peels); a
  **mid-sentence** direction (`But behold, Basil, *(he grasps him by the arm)*
  they all lacked…`) never opens a sentence, so it stays woven in — quotable
  only as part of its line, which grammar demands. Standalone `stage` blocks
  (`label_block`) now receive a `sentence_number`; the cast list (`list_block`)
  stays `NULL`. Both edition layers peel identically (their markers are
  parallel), so the prose-sentence parity guard still holds.
- **Frontend.** A global `.stage` rule (alongside `.antiqua`/`.sperrdruck`)
  mutes the run everywhere it appears. The `BlockRenderer` `stage` case renders
  a numbered direction through the clickable `<Sentence>` path and the
  null-numbered cast list as inert block html. Selection/quotation needed no
  change — they already key on `sentence_number`, which directions now carry.

The migration `0010_drama_blocks.sql` comment ("stage sentences carry
sentence_number = NULL") is now stale for directions but is left byte-identical
(applied + checksummed); this Amendment is the authority.
