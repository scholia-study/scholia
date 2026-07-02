# 0003. Poetry: a verse line is a sentence row

**Status**: Accepted
**Date**: 2026-06-14

## Context

Scholia hosts prose primary sources (Bible translations, Kant's first
Critique). We want to add poetry — starting with Shakespeare's Sonnets
(lyric) and Milton's *Paradise Lost* (narrative/epic). Verse *drama*
(plays — speakers, stage directions, act/scene/line) is **not** in this
pass, but the model must not have to be torn up when drama arrives.

Scholia's atomic, addressable unit is the **sentence**: quotation,
`page_markers`, `footnotes`, `cross_translation_alignments`, and
`natural_key` all anchor to a `sentences` row. Poetry, however, is read,
cited, and quoted by the **line** — and a line is not a sentence. Milton's
syntactic sentences run across many enjambed lines; a single (later,
dramatic) line can hold two sentences. Canonical line numbers also reset
per poem/canto and don't always equal a raw row count.

The existing hierarchy is `toc_nodes` (ltree) → `content_blocks`
(`block_type` enum: paragraph/heading/separator/figure) → `sentences`.
Numbering/citation is handled generically by `reference_systems` +
`page_markers` (this is how Bible verses and Kant's AA/B pages work).
Ingest for Kant is curated markdown → struct JSON (`packages/common`) →
DB (`kant1_struct_to_db`). A prior note already reserved `<br>` for
"literal line breaks (poetry, not yet wired)".

## Decision

**Model a verse line as a `sentences` row.** Reuse the existing
block→sentence machinery rather than introducing a parallel line layer.
Specifically:

1. **Line = `sentences` row.** Lines are delimited by **newline** in the
   curated source (`<br>` is the escape hatch for a forced break inside
   one counted line). The `.!?` sentence splitter is **bypassed** for
   verse — each delimited line becomes exactly one row regardless of
   internal punctuation. A row is a *line*, not a grammatical sentence.

2. **Stanza / verse-paragraph = `content_block`** with a new `verse`
   `block_type`. A blank line in source starts a new block.

3. **Indentation = new `indent SMALLINT` on `sentences`** (0 = flush,
   1,2,… from leading whitespace), rendered via CSS padding. `segment`
   stays NULL for verse (it keeps its distinct Kant hanging-run meaning).

4. **Line numbering = a `line` reference system**, reusing
   `reference_systems` + `page_markers` exactly as Kant's AA/B pages do.
   **One marker per line** carries the canonical, poem-local number.
   Numbers **reset per leaf `toc_node`** (a sonnet; a *PL* book) by
   default, with a front-matter `line_numbering: continue` override. The
   display interval (every 5/10/off, which margin) is a **reader display
   setting** on the existing margin-settings surface — not baked into the
   data.

5. **Quotation** reuses sentence-range selection (ranges may cross stanza
   blocks) but is **keyed/labelled via the `line` system**, Bible-style →
   "Sonnet 18, 1–4" / "Paradise Lost I.12–21".

6. **Ingest** is a new **`md_poetry_to_struct`** parser emitting the
   existing `packages/common` struct model, fed into the **reused**
   struct→DB importer. **One shared parser core + per-text config/hooks**
   (front matter + small per-work config; pluggable pre/post hooks for
   genuine oddities) — *not* a binary per text. Mirrors
   `bible_to_db --translation`.

7. **Search** uses a **per-language stored `tsv` column** on `sentences`
   (and `content_blocks`) computed with the book's `language`, replacing
   the hard-coded `'german'` expression index. This also fixes English
   Bible search. Per-line FTS granularity is inherent and matches Bible's
   verse-level search.

## Reasoning

The line-as-sentence-row choice is load-bearing: it inherits quotation,
footnotes, page-markers, cross-translation alignment, and `natural_key`
without new addressing code, and it gives **prosody a real row to attach
to later** (a `line_prosody` side table or `metadata JSONB`, purely
additive) — which is also exactly what citation and line-range quotation
need. The two alternatives were rejected:

- *A separate `lines` table alongside grammatical sentences* doubles the
  addressing model and complicates quotation, alignment, and rendering
  everywhere, for fidelity (grammatical-sentence boundaries) that the
  product doesn't use.
- *A line as its own `content_block`* explodes block counts, breaks
  block-as-stanza grouping, and pushes sentence-level features up a layer.

A distinct `verse` block_type (vs. overloading `paragraph` with a flag)
is justified because verse needs the *opposite* behaviour from
`paragraph` on the two code paths that already switch on `block_type`:
the renderer (preserve line breaks, don't reflow) and the ingest splitter
(split on newline, not `.!?`). Migration 0001 set the precedent by adding
`figure` the same way.

Storing **one marker per line** (not every Nth) decouples the canonical
number from physical row count — so it survives editorial line splits,
shared lines, and intervening blocks — and makes the display interval a
free render-time setting. Explicit canonical numbers are the same lesson
`cross_translation_alignments` already encodes: reference ≠ position.

Per-text *config/hooks* over per-text *binaries* follows ADR 0002's
cross-cutting-duplication seam in reverse: near-identical parsers would
drift, so the shared core stays authoritative and per-work difference
lives in data.

## Schema deltas

Two append-only migrations (`db/migrations/NNNN_*.sql`, next is `0006`):

1. **`0006` — poetry schema tweak** (trivial, instant, poetry-scoped):
   `ALTER TYPE block_type ADD VALUE 'verse';` and
   `ALTER TABLE sentences ADD COLUMN indent SMALLINT;` together. Safe in
   one transaction — neither statement *uses* the new `verse` value, so the
   PG12+ "can't use an added enum value in the same transaction" rule
   doesn't bite.
2. **`0007` — per-language FTS** (heavy, shared blast radius): stored `tsv`
   column + generic GIN index on `sentences` and `content_blocks`,
   replacing the `to_tsvector('german', …)` expression indexes; populated
   from the owning book's `language`.

These are kept apart not by size but by **lifecycle**: (1) is a 1ms,
poetry-only change; (2) rebuilds GIN indexes over the whole corpus,
changes Kant/Bible search behavior, and is independently shippable.
Everything else *could* legally collapse into one file (nothing forces a
split), but coupling a multi-second reindex to a trivial column add gives
two risk profiles one failure story — so the FTS overhaul earns its own
migration.

No new tables. The `line` reference system is **data**
(`reference_systems` + `page_markers` rows per poetry book), not schema.

## Implementation checklist

**DB**
- [ ] `0006_verse_block_and_indent.sql` — `ALTER TYPE block_type ADD VALUE
      'verse'` + `ALTER TABLE sentences ADD COLUMN indent SMALLINT` (one
      transaction; neither uses the new enum value).
- [ ] `0007_per_language_fts.sql` — stored `tsv` column + GIN on
      `sentences`/`content_blocks`; backfill from book `language`; drop the
      old german expression indexes; re-index and verify Bible/Kant search.

**Rust — `packages/common`**
- [ ] Add `Verse` to `BlockType`; add `indent: Option<i16>` to the
      sentence struct model.
- [ ] Verse-aware splitter path: newline-delimited, `.!?` bypassed,
      `<br>` = forced break, leading whitespace → `indent`.

**Rust — ingest**
- [ ] New `packages/md_poetry_to_struct` (shared core + per-text
      config/hooks; front matter incl. `line_numbering: reset|continue`).
- [ ] Reuse struct→DB importer; consider renaming off the `kant1_` prefix.
- [ ] Emit per-line `line`-system page_markers (poem-local, reset per leaf
      node) and a `line` `reference_systems` row per book.
- [ ] `scripts/db_poetry.sh` + `dp:poetry` package.json script.

**API**
- [ ] Surface `block_type: "verse"` and `indent` through the node/page
      response models; `pnpm codegen`.

**Frontend — `apps/web`**
- [ ] `BlockRenderer` `verse` branch: one line per row (no reflow),
      `indent` → padding, per-line selection/quotation preserved, stanza
      spacing between verse blocks.
- [ ] `line`-marker margin rendering + reader display setting (interval
      5/10/off, margin side) on the existing margin-settings surface.
- [ ] `keys.ts`/citation: line-number projection via the `line` system
      (Bible-style), ranges crossing stanza blocks.

**Content**
- [ ] Curate one sonnet + one *PL* book as the proof corpus; verify
      numbering reset, indentation, quotation labels, search.

## Consequences

- A verse "sentence" row is a line, not a grammatical sentence. FTS and
  quotation operate per line; a phrase spanning a line break won't match a
  single-row query — accepted, and identical to Bible's verse-level search.
- `natural_key` for verse becomes `{node.source_ref}/b{stanza}/s{line}`;
  moving a stanza break during curation renumbers `b`/`s`. Watch this
  under incremental re-import / reconcile hashing.
- `ALTER TYPE … ADD VALUE` is irreversible and (pre-PG) can't run in a
  transaction — fine on PG18, but `verse` can never be removed.
- The FTS change is the one delta that reaches beyond poetry (Kant/Bible);
  it needs a re-index and a search verification pass.
- Per-text hooks can quietly become per-text forks — keep the shared core
  authoritative; a hook is config or a thin pre/post step, never a parallel
  parser.

## Deferred, but unblocked by this design

- **Prosody** (meter, rhyme, scansion) → `line_prosody(line_id, …)` side
  table or `metadata JSONB`, keyed on the line row. Additive.
- **Drama** → `verse` block stays generic; speaker / stage-direction
  become future block types (or a sentence attribute), and act/scene/line
  becomes a second reference system or composite `ref_value`.
- **Multi-edition line-number drift** → `cross_translation_alignments`
  already generalizes to it.

## Re-open this ADR if

- Drama (plays) enters scope — revisit speaker/stage-direction modeling
  and the act/scene/line hierarchy before building it ad hoc.
- Prosodic analysis becomes a product goal — settle the `line_prosody`
  shape vs. `metadata JSONB` then.
- A poetry text genuinely can't be expressed as
  newline-delimited verse blocks + per-text hooks — that's the signal a
  bespoke parser path (not another hook) is warranted.
