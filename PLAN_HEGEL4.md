# hegel4 — Hegel, Enzyklopädie der philosophischen Wissenschaften (1830)

Phase 1 of adding the 1830 *Encyclopaedia* (GW 20) as corpus `hegel4`:
source → curated German layers → struct → **throwaway-DB** import check.
Phase 1 ends when the curated files are ready for the user's inspection.
Out of scope for phase 1: the English translation, reader styling of
Remarks, roster/CI/deploy wiring, any import into the local or dev DB.

## Hard rules

1. **Never touch the local DB** (`DATABASE_URL` in `.env`, `prospero` on
   `localhost:5433`) or dev/prod. No `db_reset.sh`, no `just db*`, no
   `pnpm db:*`, no `sqlx migrate run` without an explicit scratch URL.
   Every importer invocation passes `--database-url <scratch>` explicitly.
   (A safety dump exists at `~/backups/scholia/prospero-local-20261124-1619.dump`.)
2. **Never `git commit`** (the user commits). Leave the work as unstaged
   changes in the working tree.
3. Work **step by step**: finish each step's gate before starting the
   next. If a gate fails and the fix isn't clear, stop and report rather
   than improvising around it.
4. Converters **extract, don't infer**: emit only what the source carries.
   Anything doubtful goes into the review queue (Step 9), not into a guess.
5. Follow CLAUDE.md: comments only for non-obvious rationale, no divider or
   section-label comments, `cargo fmt`, enums for closed value sets.
6. Existing corpora must stay **byte-identical**: parser changes may not alter
   any other corpus's struct output (gate in Step 6).
7. Do not add hegel4 to `SCHOLIA_CORPORA`, `ingest.sh`, `struct.sh`, CI,
   or the ingest job image in phase 1. Wiring hegel4 into those can trigger
   automatic ingest/deploys, so it waits until after the user's inspection.

## Source

Giuliano Infantino, *Digitale Hegel-Edition* (hegel.idealismedition.com,
formerly hegeledition.com — the same edition hegel2 was built from, see
`assets/hegel2/curated/README.md`). Encoding CC BY 4.0; Hegel's text is
public domain; the GW apparatus is not included.

- `GW20.xml` — DTA-Basisformat TEI export:
  `https://hegel.idealismedition.com/assets/dtabf/GW20.xml`. **Primary
  source.**
- `work-gw-20.js` — the edition's JSON (same shape hegel2's converter
  reads): `https://hegel.idealismedition.com/assets/work-gw-20.js`. Used
  **only as an oracle** for line-end hyphenation and as a consistency
  cross-check. It is not an independent witness, because it comes from the
  same editor.

Both are already downloaded to
`/tmp/claude-1000/-home-filip-dev-play-scholia/7c472ed1-2469-48fa-b90d-34f2b79dbc3f/scratchpad/gw20/`.

Facts about the XML, established by inspection:

- `<body>` is captured page by page with flat `<div n="1">` elements. Each
  heading is its own div holding only a `<head>`, and multi-line headings
  are split across consecutive divs (`<head>A.</head>` then
  `<head>QUALITÄT.</head>`). All 577 §§ appear as `<head>§. N.</head>`.
- A paragraph crossing a page break is **one `<p>`** with `<pb>` and
  `<fw type="header">` (running head) inside it. The editor has already
  decided every paragraph join, so there is no page_joins table as in hegel2.
- Remarks (Anmerkungen) are unheaded small-type paragraphs:
  `<p rendition="#et #smaller">` (425 of them). Main text is plain `<p>`.
- `<pb n="63" .../>` gives the GW 20 page number. `<lb n="…"/>` gives GW
  line numbers (dropped). `<hi rendition="#g">` marks Sperrsatz (render as
  `_…_`, following hegel2's emphasis convention).
  `<milestone unit="section" rendition="#vr"/>` marks the unnumbered
  page/leaf changes of the 1830 print (dropped in phase 1; count them in
  the report).
- 31 `<note place="foot">` notes, located at their call sites. Notes that
  run across pages are split into parts linked by `next`/`prev` and must be
  rejoined into one footnote. The call mark is in `@n` (`∗)`, `∗`). Also:
  13 `<foreign>`, 5 `<space dim="vertical"/>`. No `<figure>` was found;
  if one appears, refuse to emit.
- Known heading defects to resolve in the curated TOC, not in code. There
  is a bare `c.` heading under *Die Idee*, apparently where "Die absolute
  Idee" should be. A stray `II.` / `NATURPHILOSOPHIE.` pair precedes the
  real `ZWEITER THEIL. NATURPHILOSOPHIE.` Find any others by diffing the
  `<head>` stream against the JSON's `toc_printed`.
- The layout of main text (M) and Remark (R) paragraphs per §: `M` 340,
  `MR` 148, `MR+` 85. Four §§ break the pattern: **§ 24, 95, 335, 552**
  have main text resuming after a Remark, and two of them have two separate
  Remark runs. Do not decide these. Emit them as the source has them and
  put each on the review queue.

## Target design (agreed with the user)

### Layers

`assets/hegel4/curated/`:
- `md_reviewed/` — the GW text exactly as captured (spelling, punctuation,
  `[…]` editorial supplements, print errors all kept).
- `md_modernized/` — the reading text: spelling modernized via a
  `modernize_rulings.tsv`, mirroring hegel2's mechanism (read
  `packages/hegel2_gw_to_md` and `assets/hegel2/curated/README.md` first).
- `README.md` — mirrors hegel2's README: sources, licence (CC BY 4.0 for the
  German layers, attribution to Infantino), what's included or excluded,
  and the tables.

`md_prose_to_struct` pairs them (modernized = text, reviewed =
original_text), as for hegel2.

### Book identity (tentative; the user confirms at inspection)

- corpus `hegel4`, book slug `enzyklopaedie`, title *Enzyklopädie der
  philosophischen Wissenschaften im Grundrisse*, `ORIGINAL_YEAR` 1830,
  author as hegel2. SOURCE/ABOUT/LICENCE prose modelled on
  `common::hegel2::meta`, crediting the Digitale Hegel-Edition only (no DTA
  here). Modest, factual copy, with no superlatives.

### Scope of the text

- Included: the three Vorreden (1827 2nd ed., 1817 1st ed., 1830 Vorwort),
  the Einleitung, and §§ 1–577 through the end of the work.
- Excluded: the title page and the printed Inhaltsanzeige, if captured.
  List anything else you exclude in the README.

### Nodes

`common::hegel4` with `toc.rs` / `filenames.rs` / `meta.rs` / `mod.rs`
(plus `toc_mod.rs` if the modernized layer needs modernized labels, as in
hegel2), in the shape of `common::hegel2`. The TOC is **hand-curated** from
the `<head>` stream and the JSON's `toc_printed`:
- One node per structural heading down to the lowest headed level (Theil /
  Abtheilung / A. / a. / α). The §§ are **not** nodes.
- Labels join the split heads ("A." + "QUALITÄT." → `A. Qualität`) and
  case-normalize all-caps heads the way hegel2 does. GW orthography stays
  in labels.
- Depths form a strict tree. Each node records its starting GW page
  (`"20.63"`).
- The converter refuses to emit unless every TOC node anchors to exactly
  one heading run on its page.

### § headings — headings, never sentences

Each § is written as a **`## ` heading line** inside its node, so it
produces a heading sentence with `sentence_number = NULL` and never counts
in the sentence enumeration. Check this in Step 6.

### Citation: two reference systems

| System | Markdown | Marker value | Template | Priority | Margin prefix |
|---|---|---|---|---|---|
| `gw` (primary, `{{{ }}}`) | `{{{ 20.63 }}}` at each GW page start | `20.63` | `GW {ref}` | 1 | none (value is already `20.63`) |
| `paragraph` (secondary, `{{ }}`) | `{{ 20 }}` on each § heading; `{{ 20 A }}` at each Remark start | `20`, `20 A` | `§ {ref}` | 0 | `§ ` |

The resulting citations look like `§ 20 · GW 20.63` and
`§ 20 A–21 · GW 20.63–64`. A quote from the Vorreden, which come before
any § marker, gets only `GW 20.9`. The citation query (last marker at or
before the quote, per system) lives in
`apps/api/src/modules/writing/articles/db.rs`. It needs no change; just
confirm it works in Step 8.

Pick the `ref_type` for each system by reading how hegel1 (page + gw9) and
hegel2 (single `gw`) set theirs and what `ref_type` affects downstream.
Explain the choice in the report.

**Marker value grammar and sort.** `20 A` must parse and sort as § 20
followed by its Remark (e.g. § × 10 + 1, the same idea as plato2's
`subpage_letter_sort` and hobbes1's `247b`). Find where marker values are
validated and sorted for md_prose corpora and extend that behind a corpus
knob. Add a unit test. Value format: number, a single space, then `A`. If
the reviewed §§ (24/95/335/552) need a second-Remark value, do not invent
one. Leave a review-queue item.

Placement:

```
## {{ 20 }} §. 20.

Nehmen wir das Denken in seiner am nächsten liegenden Vorstellung auf, …

::: remark
{{ 20 A }} Die hier und in den nächstfolgenden §§. angegebenen Bestimmungen …

Zweiter Absatz der Anmerkung …
:::
```

Check that the heading-line marker attaches to the heading sentence and
that the citation query resolves it (Step 8). If attaching a marker to a
heading turns out to be unsupported, stop and report. Do not move the
marker into body text without asking.

### Remarks: the `::: remark` fence

New `md_prose_to_struct` syntax. A line `::: remark` opens a fence and a
line `:::` closes it. Every paragraph inside becomes its own content block
with `block_type: "remark"`. One block per paragraph keeps quotation
anchoring and translation 1:1 alignment working unchanged. The fence exists
so the Remark reads as one unit in the markdown. Requirements:
- Paragraph numbering (`counters.paragraph`) continues through remark
  blocks exactly as it does for paragraphs.
- Page markers, footnote refs and Sperrsatz work inside remark blocks.
- Parse errors are hard errors: an unclosed fence, nested fences, a heading
  or footnote definition inside a fence, or an unknown fence kind.
- `block_type` is a pass-through string (ADR, memory), so no importer
  change should be needed. Verify that.
- Opt-in per corpus. Other corpora must be unaffected (byte-identical,
  Step 6). Add unit tests in `parse.rs` / `structure.rs`.
- Reader styling is out of scope. Do check what the reader does with an
  unknown `block_type` (grep `apps/web/src/modules/reader`) and report it.

### Footnotes

Emit `[^N]` at the call site and `[^N]: …` definitions at the end of the
node file, numbered per book (follow hegel2/hegel3). Drop the `∗)` call
mark text. Join parts linked by `next`/`prev` into one definition.

### Hyphenation

Remove line-end hyphens (`-<lb/>`) by checking the JSON's
paragraph `text`, which the editor already dehyphenated. Where the JSON
keeps the hyphen (`Experimental-Physik`, `Sich-Aufheben`), keep it. Any XML
line-end hyphen whose join the JSON can't settle refuses to emit until a
ruling covers it (a small `hyphen_rulings.tsv`, only if needed).

### Modernized layer

Mirror hegel2: a `modernize_rulings.tsv` (surface → modern) applied
token-wise. **Seed** it from `assets/hegel2/curated/modernize_rulings.tsv`
and `assets/hegel3/curated/modernize_rulings.tsv`, the same author, era and
orthography. For words not covered (natural-science vocabulary in the
Naturphilosophie, for example), add rulings in the same style. Log
uncertain ones (`uncertain` column or comment, matching hegel2's format) so
the user can review them. Keep `[…]` supplements and the `§.` heading form
unchanged in both layers.

### Sentence splitting

Use hegel2's settings (`paren_protected_splits` etc.) as the starting
point. The Encyclopaedia cites itself constantly (`§. 261. Anm.`,
`§§. 20–23`, `vgl.`, `S. 13.`, `u.s.f.`), so those must never end a
sentence. Audit the struct output in Step 7.

## Steps and gates

**Step 0 — orient.** Read: this plan; CLAUDE.md; `.agents/skills/add-corpus/`
(for reference only, since phase 1 excludes the wiring); `common::hegel2`;
`packages/hegel2_gw_to_md`; the hegel2 arm in
`packages/md_prose_to_struct/src/corpus.rs`; `parse.rs` and `structure.rs`;
ADR 0006 and 0011; `assets/hegel2/curated/README.md`.
*Gate:* none. Move on once oriented.

**Step 1 — raw archive.** Put `GW20.xml` and `work-gw-20.js` in
`assets/hegel4/raw/`, and copy them into `assets/_unassigned/hegel/` (the
private archive; gw-20 is missing there). *Gate:* confirm `raw/` is
gitignored there (`git check-ignore`), and that `git status` shows no raw
files.

**Step 2 — source survey.** Write a small throwaway script in the
scratchpad, not in the repo. Inventory every element and attribute in the
body; diff the `<head>` stream against `toc_printed`; list every
non-`§` heading with its page; compare the XML word stream to the JSON word
stream after joining hyphens. *Gate:* the inventory contains no unhandled
element types. The XML/JSON word-stream agreement is above 99.9%, with
every disagreement listed (they go into the review queue, or are explained
as export differences).

**Step 3 — `common::hegel4`.** Curated TOC, filenames, meta (both systems,
templates, priorities, margin prefixes as tabled above). *Gate:*
`cargo test -p common` passes; there's a test that depths form a strict
tree and positions are 1..N.

**Step 4 — converter `packages/hegel4_dtabf_to_md`** (new crate, added to
the Cargo workspace). Streaming XML parse (use whatever XML crate the
workspace already depends on, e.g. the one `hegel1_tei_to_md` uses).
Emits `md_reviewed` (default) and `md_modernized` (`--layer modernized`).
Front matter as in hegel2 (`position`, `label`, `depth`, `page_gw`).
Refuse to emit on anything unexpected. Short module doc comment. Unit tests
for the tricky parts: head joining, footnote-part joining, hyphen oracle,
remark runs, and markers on § headings. *Gate:* `cargo test -p
hegel4_dtabf_to_md` passes; running it twice gives byte-identical output.

**Step 5 — generate `md_reviewed`.** *Gate:* 577 `## {{ N }} §. N.` heading
lines with N = 1..577 in order; `{{ N A }}` count equals the number of
remark fences (excluding review-queue §§); 556-ish `{{{ }}}` page markers
with no gaps; 31 footnotes; no `<`, `&lt;`, `rendition`, or stray `-` at
joins; every fence closed.

**Step 6 — parser support.** In `md_prose_to_struct`: the `::: remark`
fence, the `20 A` marker grammar and sort, the `hegel4` corpus arm (source
edition only; no `--translation` yet). *Gates:*
- `cargo test -p md_prose_to_struct` passes, including the new tests.
- **Regression:** stash-free before/after comparison. Build the struct for
  kant1, hegel1, hegel2, hegel3, hobbes1, peirce1, plato1 at the starting
  revision first (copy their `derived/*.json` aside to the scratchpad),
  then again after your changes. `cmp` every output; all must be identical.
- In the hegel4 struct, every § heading sentence has `sentence_number:
  null`, and the remark blocks carry `block_type: "remark"`.

**Step 7 — modernized layer + sentence audit.** Generate `md_modernized`,
build the hegel4 struct. *Gates:* the sentence counts of reviewed and
modernized match one-for-one in every block, which the parser enforces.
Then a scan of sentence starts and ends: no sentence ends on `§.`, `§§.`,
`Anm.`, `vgl.`, `S.`, `u.s.f.`, or a bare `N.` §-reference, and no
sentence starts with a lowercase letter or a digit, unless it's listed with
a reason in the report.

**Step 8 — throwaway-DB import.**
```
docker exec prospero-pg createdb -U prospero hegel4_scratch
SCRATCH=postgres://prospero:<pw from .env>@localhost:5433/hegel4_scratch
sqlx migrate run --source db/migrations --database-url "$SCRATCH"
cargo run -p struct_to_db -- --input-file assets/hegel4/derived/output.json --database-url "$SCRATCH"
```
(Check `struct_to_db --help` for the exact flags. Never omit
`--database-url`.) *Gates:*
- The import succeeds, and running it a second time (reconcile) is a no-op.
- SQL checks: number of nodes = TOC length; page markers per system
  (`gw` ≈ page count, `paragraph` = 577 + remark count); heading sentences
  have NULL `sentence_number`; remark blocks are present.
- Run the citation query from `articles/db.rs` by hand for four positions:
  inside § 20's main text, inside its Remark, a range from § 20 A into
  § 21, and a Vorrede sentence. Expected: `§ 20 · GW 20.x`,
  `§ 20 A · …`, `§ 20 A–21 · …`, and GW-only.
- Then **drop the scratch DB** (`dropdb`) and confirm it's gone.

**Step 9 — review queue + README + report.** Write
`assets/hegel4/curated/review_queue.md`. It is a working checklist for the
user's inspection, not history: the four anomalous §§ (24, 95, 335, 552)
with GW page and line, quoting the paragraph openings; TOC heading defects
and how you resolved them; XML/JSON disagreements; uncertain modernize
rulings (count plus where they're flagged); anything excluded; the `#vr`
milestone count. Write the README. Remove scratch scripts and temp files
that aren't deliverables. `git status` should show only the intended
changes. Run `cargo fmt` and `cargo clippy` on the touched crates.

## Final report (back to the orchestrator)

For each step: done or not, with gate results (numbers, not adjectives).
Then: the files changed or added; decisions made and why (ref_type, value
format, label normalization, anything not spelled out here); open questions
for the user; and how the reader treats an unknown `block_type`.
