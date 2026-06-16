# Shakespeare as a compilation (Bible-shape)

**Status:** IMPLEMENTED for the Sonnets (2026-06-16). Plays remain future work
(the work→act→scene depth, drama block types, and TLN below are still TODO).
**Date:** 2026-06-16
**Relates to:** `docs/adr/0003-poetry-line-as-sentence-row.md`, `[[project_shakespeare1_sources]]`

> Implemented: one book (slug `shakespeare`, compilation source titled "William
> Shakespeare"); a depth-0 source-anchored "Sonnets" work node (source
> "Shakespeare's Sonnets (1609 Quarto)") with the 154 sonnets as children; the
> author is attached to the **work** source so the
> compilation is an author-less Bible-shape "self" group. The library's
> `populate_book_pills` was relaxed to emit pills for single-edition compilations
> (not just 2+-translation ones). The old standalone `shakespeares-sonnets` book
> was surgically deleted from local. The play-specific sections below are the
> remaining design for when plays land.

This captures an agreed-but-deferred restructure: presenting Shakespeare's works
as a single **compilation** (the way the Bible is modelled) so the library shows
work **pills** + a **grand table of contents**, and so **plays** slot in later
without a redesign. Pick this up when we're ready to move the sonnets onto the
compilation structure (and/or start on plays).

---

## Why (the goal)

Today the sonnets are a **standalone book** (`books.slug = "shakespeares-sonnets"`)
with **154 flat depth-1 `toc_nodes`** and no `source_id` on them. That renders as
a long nested list — no pills, no grand TOC, and it's not plays-ready.

We want the **Bible's presentation**: a "Sonnets" pill up front in the library →
a grand TOC of Shakespeare's works → the sonnet grid; with plays (Hamlet, …)
appearing later as sibling pills. In the codebase that UI is "Bible-shape", and
it is triggered by **one book whose depth-0 `toc_nodes` are source-anchored
(`source_id` set)** — see `apps/web/src/modules/reader/components/PanelToc.tsx`
(`BibleShapeFullToc` / `BibleShapeToc`) and the `book_pills` query in
`apps/api/src/modules/corpus/reading/library/db.rs`.

## Agreed decisions

1. **One compilation book** — `"The Works of Shakespeare"`. Each work (Sonnets,
   later each play) is a **depth-0, source-anchored `toc_node`** under it. This is
   what turns on the pill + grand-TOC UI. (Not a book-per-work author group —
   that gives work-cards, not the Bible UX.)

2. **Each work is self-standing and edition-capable.** Provenance lives on the
   **per-work source**, and any future alternate text is an edition/translation
   of **that work**, not of the whole corpus. This is where Shakespeare DIVERGES
   from the Bible: the Bible's translations span the whole corpus (KJV = all 66
   books), but Shakespeare's editions vary per work (Sonnets = 1609 Quarto;
   Hamlet = 1623 First Folio). So `translation_of_id` would point work→work, not
   corpus→corpus.

3. **Play hierarchy is work → act → scene (3 layers).** The **scene is the
   readable leaf**; navigation is by act then scene (Act 1 · Scene 1, Act 1 ·
   Scene 2…). Works with no scenes **fall back to act-as-leaf** (work → act).
   Sonnets stay 2 layers (Sonnets → sonnet leaf). `toc_nodes` is ltree, so
   heterogeneous depth across works is fine in the data.

4. **Citation = act.scene.line, composed from the model we already have:**
   - act + scene come from the **`toc_node`** (the scene leaf),
   - the line number comes from the existing **`line` page-marker** — scene-local
     for plays, exactly as it is sonnet-local (1–14) for sonnets today.
   - Unifying rule: **line numbers reset per leaf node** (per sonnet / per scene /
     per act when sceneless). So `3.1.56` = `{node act.scene} + {line marker}` —
     no new mechanism.
   - **TLN** (Through-Line-Number: one play-absolute count, Folio-based, used by
     ISE) is optional and would be a **second** reference system added later for
     scholars. Not shipped from ISE; TLN is a derivable standard.

5. **Drama content is additive, later (not now):** add `speaker` and
   `stage_direction` block types (`ALTER TYPE block_type ADD VALUE …`, as we did
   for `verse`); prose speeches reuse `paragraph`; verse lines reuse `verse`. No
   model change required until plays arrive.

## Target data shape (sonnets, first cut)

```
books
  slug = "shakespeare", title = "William Shakespeare",
  language = "en", source_id → (A) compilation source

sources
  (A) "William Shakespeare"               source_type='book'      (compilation root)
  (B) "Shakespeare's Sonnets (1609 Quarto)" source_type='chapter' parent_source_id=(A)
      # future: (C) "Hamlet (First Folio, 1623)" source_type='chapter' parent=(A) …

toc_nodes
  "Sonnets"   depth=0  source_id=(B)  source_ref="sonnets"  slug="sonnets"  path="sonnets"
    "Sonnet 1"  depth=1  parent=Sonnets  source_ref="sonnets:1"  slug="sonnet-1"  path="sonnets.s1"
    … 154 children …
  # future: "Hamlet" depth=0 source_id=(C); acts depth=1; scenes depth=2 (leaves)

source_persons
  (A) ↔ William Shakespeare, role='author'
```

The depth-0 "Sonnets" node with `source_id` set is the pill; its children are the
grand-TOC grid. Plays become more depth-0 source-anchored nodes — zero
restructure.

## The Bible template to mirror

`packages/bible_to_db/src/main.rs` is the working example of every mechanic above:
- canonical root source via SELECT-or-INSERT (shared, no dupes);
- per-sub-work source `source_type='chapter'` with `parent_source_id` → root;
- depth-0 `toc_node` INSERT **with `source_id`** (the anchor) + `depth=0`,
  `path=<slug>`;
- depth-1 child `toc_node` INSERT **without `source_id`**, `parent_id`=depth-0
  node, `path="<parent>.<child>"`, monotonic global `sort_order`.

The biblical-book list is a hardcoded `BIBLE_BOOKS` array; our analogue is the
canonical TOC in `common::shakespeare1` (today: 154 flat sonnets — would gain a
"Sonnets" work parent + nesting, generalised toward a "works" concept).

## Implementation checklist (when un-parked)

- [ ] `common::shakespeare1`: introduce a "works" layer — a "Sonnets" work
      (depth 0, source-anchored) with the 154 sonnets as children; give nodes
      source_ref / slug / path / depth / parent.
- [ ] Struct model (`shakespeare1_md_to_struct::model`): let a `TocNodeData`
      declare a source to create (e.g. `source: Option<{ title, source_type }>`)
      so the importer can make the per-work source + set `toc_nodes.source_id`;
      and carry the compilation book source.
- [ ] Producer (`shakespeare1_md_to_struct`): emit the "Sonnets" depth-0 work
      node + 154 children; book metadata = the compilation.
- [ ] Importer (`shakespeare1_struct_to_db`): create compilation root source +
      per-work `chapter` source (`parent_source_id`) + set `toc_nodes.source_id`;
      parent_id linkage already exists. Mirror `bible_to_db`.
- [ ] Retire the old standalone `shakespeares-sonnets` book — a SCOPED one-book
      delete; per the no-reset rule ([[feedback_never_db_reset]]) this needs
      explicit permission, but it's one book with no user content.
- [ ] Frontend: pill labels — `chapterPillLabel` strips `"Chapter "`; extend for
      `"Sonnet "` (or label sonnet nodes just `"N"`). Verify Bible-shape
      detection fires for a single-work compilation.
- [ ] Update `scripts/db_shakespeare1.sh` / any `shakespeares-sonnets` slug refs.

## Open questions / risks

- **Pill UI depth.** Bible-shape pills handle work→child (2 levels). Plays need
  work→act→scene (3) — the per-work TOC renderer will need to go deeper (or plays
  use a nested/tree view). Frontend work, due when plays land; the *data* already
  supports it.
- **Single-work compilation detection.** With only "Sonnets" present, confirm the
  library still classifies the book as a compilation group (driven by the source
  having a `parent_source_id` child).
- **Sonnet node label vs pill label** — `"Sonnet 3"` (banner-friendly) vs `"3"`
  (pill-friendly). Decide where the stripping happens.
- **Depth-0 work node content.** The Bible gives its depth-0 book node a heading
  block; we may skip content on the "Sonnets" node (it's pure navigation) to
  respect the single-source-of-truth rule ([[feedback_reader_single_source_of_truth]]).

## Already done (context, NOT parked)

- Verse lines are clickable/quotable in the reader (`BlockRenderer` `verse` case).
- Sonnet number heading is authored as `## N` in the curated MD (both layers) and
  renders on the page via the existing `heading` block — single source of truth.
- Long-s baked into `md_reviewed`; two-layer model (modern / 1609 old spelling).
These carry forward unchanged into the compilation restructure.
