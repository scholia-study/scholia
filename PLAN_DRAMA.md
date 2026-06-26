# Drama support — Ibsen's *Emperor and Galilean* (schema, ingest, API, frontend)

> Status: **planned, not yet implemented.** Pick this up once the `ibsen1`
> modernization pass (`assets/ibsen1/curated/md_modernized/`) is complete.

## Context

Ibsen's *Kejser og Galilæer* is the first **drama** in Scholia. The curated
two-layer markdown is in flight (`assets/ibsen1/curated/md_reviewed/` +
`md_modernized/`, acts being modernized one per turn). Drama was deliberately
deferred by **ADR 0003** (poetry), which says to *re-open it when drama enters
scope* and sketches the seam: "speaker / stage-direction become future block
types (or a sentence attribute); act/scene/line becomes a reference system or
composite ref_value." That moment is now.

Drama differs from prose/poetry in three ways the reader must honour:
- **Character names are non-clickable**, exactly like headings in other works.
- **Dialogue is selectable sentence-by-sentence**, exactly like elsewhere.
- A single speech can **mix prose and verse** (confirmed: `@ Kejser Julian` in
  part 2, act 3), and stage directions appear at scene level, as a speaker-owned
  opener, mid-speech, and inline.

The investigation found the existing model already fits drama with a *tiny,
additive* change set, because `block_type` is a pass-through string end to end
(DB enum → importer bind → API `block_type::TEXT` → frontend `string`) and
headings already render non-clickable via `HeadingSentence`.

## Decisions (from review)

1. **Speaker = its own non-clickable `speaker` block.** A speech is the implicit
   run: `speaker` block + following `paragraph`/`verse`/`stage` blocks until the
   next `speaker`/`heading`. No `speaker` column (a first-class speaker→lines
   link is deferred; additive later if character-filtering is wanted).
2. **Stage directions = non-selectable italic chrome** (`stage` block_type),
   rendered like headings/speakers (no `data-sentence-key`, no `onClick`).
3. **Citation by page** as the default — the `{{{ N }}}` markers become a
   "1873 page" reference system. Denote **`p. N`** for page and **`s. N`** for
   sentence ordinal (not `§`, which is a paragraph mark). Page is the default;
   sentence ordinal is the secondary label.

## Schema changes — one migration, two enum values, zero new columns

`db/migrations/0010_drama_blocks.sql` (mirrors how `figure` (0001) and `verse`
(0006) were added):

```sql
ALTER TYPE block_type ADD VALUE IF NOT EXISTS 'speaker';
ALTER TYPE block_type ADD VALUE IF NOT EXISTS 'stage';
```

Nothing else. Speaker name lives as the `speaker` block's single sentence; page
numbers reuse `reference_systems` + `page_markers` (data, not schema), as Kant's
A/B pages and poetry line numbers already do.

## Content model — how each markup token maps

| MD token | block_type | sentences | selectable? | enumerated? |
|---|---|---|---|---|
| `## HEADING` (act title) | `heading` | 1 | no | no |
| `@ NAME *(opener)* SEP` | `speaker` | 1 (name + italic opener) | **no** | no |
| `@stage (…)` / own-line `*(…)*` | `stage` | 1 (whole direction) | **no** | no |
| flush prose lines | `paragraph` | split on `.!?` | **yes** | `sentence_number` set |
| `| verse line` | `verse` | one per line | **yes** | `sentence_number` set |
| `*word*` / inline `*(…)*` | — | inline emphasis in html | (within sentence) | — |
| `{{{ N }}}` | — | `page_markers` row on the sentence where the page begins | — | — |

Enumeration rule (parser-controlled, matches existing `heading`/`figure`):
`speaker`/`stage`/`heading` sentences get `sentence_number = NULL` (non-clickable,
excluded from the selection key); only `paragraph`/`verse` dialogue sentences are
numbered. A mixed prose+verse speech = `speaker` + `paragraph` + `verse` blocks
(the speaker label renders once, on the `speaker` block).

## Verse / chorus inside a play — already supported, no migration

Plays often embed verse (hymns, chants, chorus). This reuses the **exact**
Milton/Shakespeare machinery — **no new migration**:

- The `verse` block_type and `sentences.indent` shipped in **migration 0006**.
  `BlockRenderer`'s `verse` case already renders one line per `Sentence`,
  line-by-line selectable.
- **Line-by-line selection + shift-click ranges ride entirely on
  `sentence_number`** (`useRangeSelection` computes `[start,end]` from it;
  `keys.ts` keys on it). `poetry_md_to_struct/src/parse.rs:69,178‑194` already
  assigns a `sentence_number` to **every verse line** from a single global
  counter shared with prose sentences (`:243‑254`). The drama parser does the
  same.
- Consequence: prose dialogue selects sentence-by-sentence (Kant/Bible-style)
  and embedded verse selects line-by-line (Milton/Sonnets-style) with **no
  branching** — and because both share the counter, a shift-click range can span
  a prose→verse boundary inside one speech.
- The `line` reference system is **optional** (citation labels only, e.g.
  "Sonnet 18, 1–4") and is **not** required for selection. Drama omits it and
  cites by page (`p. N`); a line-numbered chorus could add it later as pure data
  (no migration).

## Ingest — new parser, reused importer (ADR 0003 pattern)

- **New `packages/drama_md_to_struct`** (sibling of `poetry_md_to_struct`):
  tokenizes the drama markup above into the *existing* shared struct
  (`Output`/`ContentBlockData`/`SentenceData` — `block_type` is a free string,
  so `speaker`/`stage` need no struct change). Reuses `common::sentences`
  (splitter), `common::content` (md→html), and the two-layer pairing approach
  poetry uses. Runs the **same parity check** done by hand while modernizing:
  modernized vs reviewed must share block sequence, block types, per-block
  sentence/line counts, and page-marker sequence.
- **New `common::ibsen1`** module (like `common/src/milton1.rs`,
  `shakespeare1.rs`): canonical TOC — book + 2 parts (depth-0 title-page nodes
  `cf`/`kj`) each parenting cast + 5 acts (depth 1) — filenames, labels, source
  metadata, and the `1873` page reference system config
  (`cite_priority = 0`, `cite_template = "p. {ref}"`).
- **Reuse `packages/poetry_struct_to_db`** unchanged — `import.rs:276` binds
  `block.block_type` to `::block_type` and takes `sentence_number` from the
  struct, so it imports `speaker`/`stage` with no code change.
- Reference systems (data): the `1873` page system (default citation, `p. N`).
  No act/scene/line system needed — acts are toc nodes, there are no scenes, and
  the prose drama isn't line-numbered. Sentence ordinal (`s. N`) rides on the
  existing `sentence_number`.
- Add `scripts/db_ibsen1.sh` + a `dp:ibsen1` package.json script (mirror
  `dp:poetry`/`scripts/db_*`).

Representative files: `packages/poetry_md_to_struct/src/{parse.rs,corpus.rs,model.rs}`
(template), `packages/poetry_struct_to_db/src/import.rs` (reused),
`packages/common/src/{sentences.rs,content.rs,milton1.rs}`.

## API — no code changes

`block_type::TEXT` (`apps/api/src/modules/corpus/reading/page/db.rs:170,482`)
already forwards any block_type string; `SentenceResponse`/`ContentBlockResponse`
already carry everything (`page_markers`, `sentence_number`, two-layer
`original_*`). Citation/label is driven by the `reference_systems` data rows
(0008 `cite_priority`/`cite_template`). So **no handler/model edit and no
`pnpm codegen`** is required for speaker/stage — they arrive as data.

## Frontend — two render cases, selection untouched

In `apps/web/src/modules/reader/components/BlockRenderer.tsx`, add two cases to
the `Block` switch, both routed through the existing **non-clickable**
`HeadingSentence` path (no `data-sentence-key`, no `onClick`):

- `case "speaker"`: name on its **own line, flush left**, **UPPERCASE + bold**,
  **muted `text-stone-500`** (kept easy to tweak), with a small top margin
  (e.g. `mt-5 mb-1`) to separate speeches. The italic `*(opener)*` rides inline
  in the sentence html after the name, keeping the literal `.`/`:`.
- `case "stage"`: **italic, muted `text-stone-500`, flush left, full width**
  (not indented) — reads as scene/stage apparatus distinct from dialogue.

**Layout — name above, dialogue indented.** Dialogue blocks (`paragraph`,
`verse`) render with a **left indent** (e.g. `pl-6`) so speaker names form a
scannable flush-left column above their indented speech; `speaker`/`stage`/
`heading` stay flush. The indent is conditional (Kant/Milton must stay
un-indented): `Block` takes an `inDrama` flag from the node renderer, which
detects a drama node by the presence of `speaker` blocks (or a book genre flag)
and adds the left padding only to the `paragraph`/`verse` cases. Visual hierarchy:
dialogue is the dark primary text (`text-stone-700`); names + stage directions
recede as muted `text-stone-500` apparatus. Names are **inert** (clicking does
nothing, like a heading).

`paragraph` and `verse` dialogue already render selectable `Sentence`s — so
prose selects sentence-by-sentence and embedded verse selects line-by-line,
shift-click ranges and all, unchanged via `useRangeSelection` + `keys.ts` (which
only ever sees the numbered dialogue sentences; `speaker`/`stage` never call
`sentenceKey`). Page markers render in the margin through the existing
`MarginNotes`/`page_markers` path, and the modernized/original toggle swaps the
text — both unchanged. No `keys.ts`/selection/quotation changes.

## Front-matter nodes (title page 001, cast list 002)

- **Title page** (`## CÆSARS FRAFALL,` + subtitle line): heading + a
  non-selectable line — already expressible.
- **Cast list** (`## DE OPPTREDENDE:` + `- ` bullets + trailing `@stage`):
  render the dramatis personae as **one non-selectable block** carrying the
  `<ul>` html (reuse the non-clickable rendering path; no third enum value).
  Decide at implementation whether to lean on `stage`-style rendering or a thin
  `list` helper — non-blocking, low risk.

## ADR

Write **`docs/adr/0005-drama-speaker-and-stage-blocks.md`** recording: speaker as
a non-clickable `speaker` block (vs. a `speaker` column — deferred), stage
directions as non-selectable `stage` chrome, page-based citation (`p. N` /
`s. N`), and that this re-opens ADR 0003 §"Re-open this ADR if … Drama enters
scope." Note the one tradeoff: no first-class speaker→lines association yet.

## Verification (end to end)

1. **Throwaway DB only** (per project rule — never local/dev): run
   `drama_md_to_struct` then `poetry_struct_to_db --database-url <scratch>
   --dry-run` first, then for real against the scratch DB.
2. Confirm: each act node has `heading` → `stage`/`speaker`/`paragraph`/`verse`
   blocks in order; speaker/stage sentences have `sentence_number = NULL`;
   dialogue sentences are numbered; `page_markers` land on the right sentences;
   modernized↔reviewed parity holds (block + sentence + marker counts equal).
3. Run the app; in the reader verify: **character names are not clickable**
   (like headings), **dialogue selects sentence-by-sentence + shift-click range**,
   stage directions show italic and non-selectable, the two-layer toggle works,
   and a quotation labels as `Cæsars Frafall, Første handling · p. 12`.
4. Clean up: drop the scratch DB, remove temp files (per project rule).

## Out of scope / future (additive)

- First-class **speaker→lines** association (a `speaker` column) for character
  filtering / "all of Julian's speeches".
- Act/scene/line numbering system (not needed for this prose-drama text).
- Importer generalization/rename off the `poetry_` prefix if a third genre lands.
