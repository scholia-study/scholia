---
name: peirce1-curation
description: Hard-won pitfalls from building peirce1 (Essays in Pragmaticism) out of digitized journal transcriptions — transcription arbitration, sentence-splitter traps, venue pagination, per-essay bibliography, figure quoting, and the frontend shape heuristics per-node sources can trip. Use when working on peirce1 (new papers, emendations, splitter tweaks), when building any edition from digitized/transcribed text (Wikisource, Gutenberg, etc.), or when adding per-node sub-work sources to a corpus.
---

peirce1 was NOT "take the digitized text and import it." The transcriptions
were faithful-looking but error-carrying, the prose defeated the sentence
splitter in a dozen ways, and per-essay sources tripped frontend heuristics
written for the Bible. Everything below was learned the hard way in one
session cycle (2026-08). ADRs 0011 (single-layer editions), 0012 (mid-book
insertion), 0013 (per-essay sources) carry the design rationale; this file
carries the traps.

## Standing rules (non-negotiable)

- `assets/peirce1/curated/md_reviewed/` is THE editorial surface. The
  converter (`peirce1_wikisource_to_md`) is FROZEN — never re-run it, never
  "fix" it. Raw inputs under `assets/peirce1/raw/` are reference only.
- One defect found ⇒ scan and fix the whole class across all 16 files.
- Emendations are witness-gated: arbitrate against the original printing's
  scan OCR before changing a reading (below), and log every change in
  `assets/peirce1/curated/CURATION_NOTES.md` under **Emendations** with the
  witness page.
- Positions are spaced 1000 apart (5-digit filenames `01000`–`16000`) so
  papers can be inserted in date order. Numbers are permanent once
  published; never renumber, never reuse. Mid-book insertion works ONLY via
  `--allow-insertion`, already set in peirce1's `scripts/ingest.sh` arm
  (ADR 0012). Test imports on a throwaway scratch DB, never local/dev.

## 1. Transcriptions lie — arbitrate against the printing

Wikisource-derived text is proofread but not clean. Real errors found:
stray quotation marks with no printed counterpart, dropped/moved periods
("about Therefore. bleeding"), silently reworded phrases ("Although it
appear" for the printed "although it would appear"), Latin corruption
("confugere as eam"), `--` for em dashes, and one hallucinated word: "we
dust" for the printed "we must".

The arbitration witnesses are the Internet Archive scan OCRs of the actual
issues (e.g. `sim_journal-of-speculative-philosophy_1868_2_3` →
`https://archive.org/download/<id>/<id>_djvu.txt`; curl it and grep — the
files are ~300KB and too big for a WebFetch summary to find a phrase in).
peirce.org is a second witness where it covers the text. Never emend from
memory or from the Collected Papers (copyrighted; also a different text).

What is NOT an error: spaced punctuation (`reaction ; as when`), spaced
ellipses (`. . . .`) — the printings' own typography stands (diplomatic
layer).

## 2. Sentence-splitter traps in 19th-century journal prose

All fixed in `common::sentences` with regression tests (~156 in crate);
when a new paper surfaces a new miss, follow the same pattern: exhaustive
corpus audit first, then a guarded rule, then tests.

- **Enumerators attach to the wrong sentence**: "…as follows: 1. That …"
  must split so "1." STARTS the next sentence, not ends the previous.
  Guarded against citation forms (`cap.`, `p.`, `Vol.`, year-like numbers).
- **Abbreviation matching needs word boundaries**: "intellect." must not
  match an abbreviation list entry "lect." — check the char before is
  non-alphabetic (`ends_with` alone is wrong).
- **Single-letter variables end sentences**: "…the premise G. For …" — but
  name initials must NOT split ("M. Vacherot", "L. Le Conte"); NAME_GUARD +
  initial-chain regexes.
- **Colon before quotation splits** (`: "You mean…`), and Early-Modern
  "herald" colons are corpus-knob-gated — peirce uses `enum_label_splits`,
  NOT hobbes' `strong_colon_splits`.
- **Headings need their own splitter** (`heading_splitter` in BlockCtx):
  prose rules mangle title-page/heading text.
- **`|||` is the authored forced split** for cases no rule can know
  ("F and G.|||" — G. genuinely ends the sentence).
- **Regex crate traps**: `"_$1_"` parses as group `1_` (write `${1}`); raw
  strings keep trailing `\` literal; no lookahead exists — restructure.

## 3. Venue pagination and citation resolution

- One reference system, venue-qualified block markers: `{{{ PSM 12:1 }}}`,
  `{{{ PAAAS 7:287 }}}` — five periodicals with colliding volume numbers,
  so the venue prefix is load-bearing. `cite_template` `"{self} · {ref}"`.
- **The opening page marker of each paper lives INSIDE the title heading**
  (`## {{{ PSM 12:1 }}} The Fixation of Belief`). Heading sentences have no
  `sentence_number`, which is exactly why citation resolution is
  position-based (node sort → block position → sentence position), finding
  the last marker **at-or-before** the quoted range — a page marker marks
  where a page *begins*; in-range scans miss almost every quotation in a
  sparse-marker book. Don't regress `batch_get_sentences` to range scans.
- A quotation citing `s. N` instead of a venue page means marker/heading
  drift — check the marker is present and precedes the anchor, and that
  the article/api process actually restarted.

## 4. Per-essay bibliography (ADR 0013) — the sharp edges

- Imprints live in `common::peirce1::meta::PAPER_IMPRINTS` (16 rows,
  parallel to the TOC; printing year, journal, volume — e.g. Categories is
  1868, the PAAAS volume's printing year, not 1867). The parser emits them
  as `NodeSource`; the importer makes protected 'chapter' sources.
- **Node sources are NOT in the reconcile content hash.** New/corrected
  imprints reach an already-imported book only because a sync pass in the
  importer's reconcile branch upserts + links them before the root-hash
  short-circuit. If imprints don't appear after re-import, that pass is the
  suspect.
- Chapter-source identity is `(title, source_type, publication_year)` —
  upserts reclaim on conflict. A title/year change mints a NEW source row.
- `--replace` must delete chapter children BEFORE the parent source
  (`chk_chapter_has_parent` rejects the FK's SET NULL).
- Article-bibliography resolution walks toc ancestors for a chapter source
  **gated on `journal_name IS NOT NULL`** — per-Bible-book chapter sources
  have no imprint and must keep falling back to the whole translation.
  Removing that gate silently rewrites every Bible bibliography.

## 5. Per-node sources trip Bible-shape frontend heuristics

Anchoring `toc_nodes.source_id` on every top-level node made two heuristics
fire wrongly; both are fixed but the pattern generalizes — **grep for
`source_id` consumers in `apps/web` before giving any corpus per-node
sources**:

- TOC "Bible-shape" detection (`PanelToc`, `books.$bookSlug.index`) now
  requires `source_id && children.length > 0` — essays are leaf nodes.
- Library pills fire only for author-less "self" groups; peirce's authored
  book source keeps it a normal shelf card. Keep it authored (ADR 0004:
  standalone works, not compilations).

## 6. Figures and unnumbered sentences

- Figures are verbatim `<figure>` HTML; the figcaption is the block's one
  anchor sentence; empty caption ⇒ parser auto-labels "Figure N.".
  Figure/heading sentences have NO `sentence_number` — anything that
  addresses passages by number silently breaks on them:
  - Quotation-list responses `COALESCE(sentence_number, figure_number)` —
    a figure quote's "start number" is the FIGURE number. Never feed it to
    a sentence-number range.
  - Article embeds address these by sentence UUID (`sid=` directive attr →
    `start_id` on the batch endpoint), which also returns `figure_html` +
    `figure_number` so the card renders the whole figure.
  - Reader deep-link key for a figure is `fig{N}` (`modules/reader/keys.ts`).
- Embed rendering context matters: article pages wrap content in Tailwind
  `prose` (re-adds list bullets — figure markup needs `not-prose`), and the
  MDX editor's contentEditable is `white-space: pre-wrap` (pretty-printed
  figure HTML shows its newlines — force `whitespace-normal`).
- Wiki leftovers `<poem>`/`<nowiki>` in curated MD cause React
  unknown-element warnings — strip them in curation, not in code.

## 7. Growing the selection (the deferred 1905–06 papers, etc.)

1. Licensing check (PD), transcription source, scan witness located.
2. New file at the free position in date order (e.g. `15500`); TOC row in
   `common::peirce1::toc`, filename row, `PAPER_IMPRINTS` row — all three,
   same commit. Venue marker in the title heading.
3. Full curation pass per sections 1–2 (assume the transcription lies).
4. `just struct peirce1`; scratch-DB import + insertion rehearsal (ADR
   0012's verification recipe); confirm existing sentence UUIDs survive.
5. Byte-identity check on the other corpora's structs if any shared parser
   code was touched.
6. Ship: dev bump runs the ingest automatically; prod needs `just promote`
   PLUS the hand-curated prod `ingest-jobs/kustomization.yaml` entry —
   promote copies manifests but never the resource list.
