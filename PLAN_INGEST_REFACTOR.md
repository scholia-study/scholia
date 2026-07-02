# Ingest pipeline review: making the pipeline DRY, structured, maintainable

**Status**: Proposal — for discussion, not yet accepted
**Date**: 2026-07-02
**Scope**: everything between curated assets and Postgres — parser crates,
struct schemas, importer crates, the reconcile toolkit, shell scripts, job
images, k8s manifests, CI filters.

---

## 1. The overriding principle: the narrow waist

Every text on Scholia, whatever its genre, flows through the same midpoint:
a **struct JSON** describing `book → reference_systems → toc_nodes → blocks →
sentences → markers`. That JSON is the pipeline's **narrow waist** — the same
role IP plays in networking: many things above, many things below, exactly one
thing in the middle.

> **The struct JSON is the narrow waist of the pipeline. Above the waist:
> one parser per *genre*. Below the waist: exactly one of everything
> (one schema, one importer, one reconciler, one job image, one manifest
> template). A *corpus* crosses the waist as data — never as code.**

Four corollaries, which are the actual design rules:

1. **One schema at the waist.** There is exactly one `Output` struct. A new
   capability (footnotes, verse indent, speaker blocks) is added to it once,
   optional-by-default, and every genre inherits it.
2. **Genre = code, corpus = data.** A genre (annotated prose, verse, drama,
   versified compilation) earns a parser crate, because genres genuinely read
   differently. A corpus (kant1, kant3, hegel1, ibsen1…) earns only a
   `common::<corpus>` data module + a corpus-builder arm + a manifest entry.
3. **Below the waist, corpus identity is a parameter.** The importer,
   reconciler, scripts, Dockerfile, and Job manifest never contain a corpus
   name except as an argument/env var/template token.
4. **The new-text test** (acceptance criterion for the whole refactor):
   *adding a new text of an existing genre requires curated MD, one
   `common::<corpus>` module, one corpus-builder arm, and one manifest entry —
   zero new crates, zero new shell scripts, zero new Dockerfiles, zero new k8s
   manifests, zero new CI filters.*

This principle is not invented — it's a generalization of what already works
here. The successes all follow it: poetry+drama sharing `struct_to_db`;
shakespeare1+milton1 as pure configs of one verse parser; kant3 reusing kant1's
markdown grammar as a library; `reconcile` as one shared toolkit; `dataduct`
as one connect/purge/seed layer. The failures are all places where it was
violated: `kant3_struct_to_db` is an 871-line copy of `kant1_struct_to_db`
differing in **two hardcoded `about_text` blurbs**; `bible_to_db` re-implements
the reconcile orchestrator by hand; each corpus fans out into **six** copied
orchestration artifacts.

**Why now:** `assets/hegel1/raw/` already exists. Under the current structure
hegel1 costs ~2 new parser crates + a third forked importer + 6 orchestration
artifacts. Under the target structure it costs a data module and a manifest
entry.

---

## 2. Current state — the map

```
                    PARSERS (curated MD → struct JSON)
  kant1_md_to_struct ─────────┐  (grammar lib + KrV binary, 2138)
  kant1_md_translation_… ─────┤  reuse kant1 grammar,        (880)
  kant3_md_to_struct ─────────┤  fork orchestration          (894)
  kant3_md_translation_… ─────┘                              (762)
  poetry_md_to_struct  — shakespeare1|milton via --corpus    (696)
  drama_md_to_struct   — ibsen1, --translation flag         (1247)
  (bible has NO parser — bible_to_db reads raw API JSON directly)

                    WAIST (struct schema)                 ← should be ONE
  kant1_md_to_struct::model   (footnotes; no about_text/indent/NodeSource)
  text_struct::model          (about_text, nodes_per_page, NodeSource,
                               indent; no footnotes)

                    IMPORTERS (struct JSON → Postgres)    ← should be ONE
  kant1_struct_to_db (871) ── kant3_struct_to_db (871): byte-identical
                              except about_text + CLI strings
  struct_to_db (800): poetry + drama, generic
  bible_to_db (1970): own schema, own IR, own reconcile orchestrator

                    SHARED (healthy)
  reconcile (align/deps/hash/keys/orchestrate, 2137)
  dataduct  (db connect, cache purge, system user, 139)
  common    (sentences + per-corpus TOC/config data, 6028)

                    ORCHESTRATION                          ← should be ONE
  6× scripts/db_<corpus>.sh   5× jobs/ingest-<c>/Dockerfile (~identical)
  5× jobs/ingest-<c>/*.sh     5× infra/k8s/jobs/ingest-<c>.yaml (~identical)
  5× CI path filters          6× copies of the rclone config block
```

---

## 3. Findings (the duplication + drift inventory)

### 3.1 Importer layer — the worst offender

- **`kant3_struct_to_db` ≡ `kant1_struct_to_db`.** `reconcile_input.rs` is
  byte-identical; `import.rs` differs only in two hardcoded `about_text`
  blurbs (`kant{1,3}_struct_to_db/src/import.rs:205-215`); `main.rs` only in
  help strings. Every bug fix must be hand-ported. The *only* reason the fork
  exists is that book copy is embedded in code — `text_struct::BookData`
  already solved this with an `about_text` **data** field.
- **`struct_to_db` vs the Kant pair: same skeleton, three deliberate deltas.**
  Kant adds footnotes end-to-end (insert loop `import.rs:516-580`, footnote
  translation maps, footnote parity in `validate_translation_parity`);
  struct_to_db adds `--replace`, node-level sub-work sources (`NodeSource`),
  `nodes_per_page`, `indent`. Their `validate_translation_parity` and
  `sort_name` are drifted copies of each other.
- **`bible_to_db` re-implements the reconcile orchestrator.**
  `bible_to_db/src/reconcile.rs` (642 lines) mirrors
  `reconcile::orchestrate::reconcile_book` step-for-step with its own IR,
  its own `ReconcileReport`, and its own `TEMP_SENTENCE_NUMBER_BASE`.
  **Real behavioral drift both ways**: Bible renumbers `page_markers.sort_order`
  after sentence renumber (`reconcile.rs:597-605`) — the shared orchestrator
  doesn't; the shared orchestrator supports additive growth (appended
  nodes/blocks/footnotes reconcile in place) — Bible aborts on any verse-set
  change.
- **Correctness footgun:** `indent` is not part of the sentence content hash
  (`struct_to_db/src/reconcile_input.rs:24-26`), so an indent-only edit to a
  poem is invisible to reconcile without `--full-rewrite`.

### 3.2 Parser layer — a genre parser trapped in a corpus crate

- `kant1_md_to_struct` is really the **annotated-prose genre parser** (markdown
  grammar, footnotes, figures, dual page systems, indented runs) — kant3
  already consumes it as a library. But the orchestration around it
  (`main.rs` + `structure.rs`) is forked 4× (~90% identical): kant1/kant3 ×
  source/translation. `find_parent_source_ref`/`build_path`/`entry_slug`/
  `build_block`/`rewrite_footnote_refs` exist in 4 near-identical copies.
- **Config leaked into code**, unlike poetry/drama which keep it in
  `common::<corpus>` + `corpus.rs`: book metadata + reference systems are
  string literals in each `structure.rs` (`kant1/structure.rs:59-84` etc.);
  figure label words ("Abbildung"/"Figure") at call sites; asset paths in CLI
  defaults.
- **Small utility duplication across families:** front-matter parsing (3
  hand-rolled copies — drama's is a near-verbatim copy of poetry's),
  `scan_dir` (6 copies), `strip_markers` (2), `resolve_marker_to_sentence`
  (5), `strip_indent` (2 byte-identical).
- **Divergent md→html:** `text_struct::html` escapes HTML metacharacters;
  the Kant `html.rs` does not (it has richer markup: footnote refs,
  sperrdruck, antiqua). Needs a deliberate decision, not silent divergence.
- **Asymmetric quality:** kant3_translation collects *all* parity mismatches
  and reports them at once (`main.rs:230-278`); kant1_translation panics on
  the first. The better behavior exists but only in one copy — the signature
  cost of forked code.
- **Healthy already:** all sentence splitting is centralized in
  `common::sentences`; per-corpus TOC data lives in `common::{kant1,kant3,
  shakespeare1,milton1,ibsen1}`.

### 3.3 Orchestration layer — six artifacts per corpus

- The **rclone remote config block** (~8 lines) is copied verbatim in **6
  files** (5 job entrypoints + `assets_sync.sh`).
- **5 Dockerfiles** are the same ~35-line cargo-chef template varying only in
  crate/binary name; **5 k8s Job manifests** are ~55 identical lines varying
  only in a corpus token in 5 spots; milton1/shakespeare1's entrypoints are
  near-byte-identical.
- The bible "KJV first, 4 parallel" block is duplicated between
  `scripts/db_bible.sh` and `jobs/ingest-bible/ingest_bible.sh`.
- `DATABASE_URL` default + sqlx-cli guard duplicated across `db_prepare.sh`/
  `db_migrate.sh`/`db_reset.sh`.
- **Consistency rot (the predictable result of copies):** ibsen1 has no job
  dir, no k8s manifest, no CI filter, and is missing from `db:reload`/
  `db:dev:reload` — it can only be ingested locally. Flag-forwarding differs
  per script (only `db_ibsen1.sh` strips a leading `--`; kant1/bible don't
  forward at all). Comments reference `dp:*` scripts that no longer exist.
  Derived-output layout splits into two camps (kant: nested
  `derived/md_to_struct/output.json`; poetry/drama: flat `derived/output.json`).

---

## 4. Proposals

Ordered so each phase is independently shippable and independently valuable.
Estimated net deletion across P1–P4: **~3,500–4,000 lines of Rust** plus ~4/5
of the orchestration boilerplate.

### P1 — One schema at the waist *(small, unlocks P2)*

Fold the Kant schema into `text_struct::model`:

- Add `SentenceData.footnotes: Vec<FootnoteData>` (+ `FootnoteData`,
  `FootnoteSentenceData`) — `#[serde(default, skip_serializing_if =
  "Vec::is_empty")]`, so poetry/drama JSON is unchanged.
- Add `BookData.publisher: Option<String>` (kant binds `book.source` into
  `sources.publisher`; the generic importer currently drops it).
- Kant parsers emit `text_struct::model`; delete
  `kant1_md_to_struct::model`. `about_text` moves from importer hardcode into
  the struct JSON (authored in the corpus config — see P3).

Result: one `Output`. The waist exists.

### P2 — One importer *(the big win; medium risk, well-guarded)*

Teach `struct_to_db` the two things only Kant's importer does, then delete
both Kant importers (−1,742 lines, +~250):

- Footnote insertion (footnote rows, footnote sentences,
  `footnote_natural_key`, book-global footnote numbering).
- Footnote-aware translation mode: source footnote-sentence maps + footnote
  parity in `validate_translation_parity` (single merged implementation).
- Bind `about_text`/`publisher`/`nodes_per_page` from the JSON (P1 fields).
- One `sort_name`, one parity validator, one `reconcile_input.rs` glue
  (footnotes + indent + `WorkSource` in a single copy).
- While here, fix the **indent hash gap**: include `indent` in
  `SentenceContent` hashing (document the one-time `--full-rewrite` needed for
  poetry books).

Guard-rails: `--dry-run` against a scratch DB per corpus before/after; the
reconcile crate itself (align/deps/hash/orchestrate) is untouched; kant
imports must produce identical row counts and identical content hashes.

### P3 — Kant parser family → genre parser + corpus data *(medium)*

`kant1_md_to_struct` is already the genre grammar; make its orchestration
corpus-driven, mirroring poetry/drama:

- Rename to **`prose_md_to_struct`** (the annotated-prose genre parser) with a
  `corpus.rs` builder: `--corpus kant1|kant3 [--translation]`. The 4 forked
  `main.rs`/`structure.rs` collapse into one driver + per-corpus config.
- Move the leaked config into `common::{kant1,kant3}`: book metadata,
  reference-system definitions (incl. kant3's Roman-or-Arabic `e1790` sort
  rule as a corpus flag, not a code branch), figure label word, `about_text`
  copy, asset paths.
- Adopt kant3_translation's collect-all-mismatches parity reporting as the
  single shared behavior.
- **hegel1 becomes the proof**: it should land as `common::hegel1` + one
  builder arm. If it needs a new crate, P3 failed.

Deletes ~2 crates outright (kant3_md_to_struct, kant3_md_translation_to_struct)
and most of two more.

### P4 — Shared parser utilities *(small, mechanical, any time)*

Into `text_struct` (or a thin `parse_core` module within it): front-matter
parsing (one impl, `aa_page` optional), `scan_dir`, `strip_markers` (marker
kinds as a parameter), `resolve_marker_to_sentence`, `strip_indent`.
Decide the HTML-escaping question deliberately: either the Kant grammar adopts
escape-first (verify curated MD contains no intentional raw HTML outside
`<figure>` blocks) or the divergence is documented in the module header.

### P5 — Orchestration: corpus manifest, one of everything *(independent; can ship first)*

Replace the 6-artifact-per-corpus fan-out with parameterization:

- **One `scripts/lib.sh`** holding the rclone config block, the
  `DATABASE_URL` default + sqlx guard, and a `run_importer <crate> <args…>`
  helper (build + flag-forward with the `--`-strip shim — one consistent
  behavior everywhere).
- **One `scripts/ingest.sh <corpus>`** driven by a small manifest (per corpus:
  struct path(s), `--source-book-slug`, import order). `db:<corpus>` npm
  aliases stay as ergonomic shims; `db:reload` iterates the manifest so a
  corpus can never silently fall out again.
- **One job image** (`jobs/ingest/`): a single Dockerfile building the (post-P2
  single) importer binary; entrypoint takes the corpus as arg/env, syncs
  `scholia-assets/<corpus>/derived`, runs the manifest entry. One k8s Job
  template (kustomize patch or `envsubst`) instead of 5 manifests; one CI
  filter.
- Normalize the derived layout to flat `derived/output.json` (+
  `translation_output.json`) for kant too, so sync paths are uniform.
- Immediate rot fixes regardless of the above: wire **ibsen1** into deploy
  (job, manifest, CI, reload chains), delete stale `dp:*` comment references.

### P6 — Bible convergence *(largest, last, two stages)*

Bible is genuinely different in three ways that must survive: compilation
shape (per-book sub-work sources — which `NodeSource` already models),
verse-ref–based cross-translation alignment (verse counts legitimately drift,
so translations are *not* sentence-locked like Kant/Ibsen), and canonical
drift checks. Everything else is the shared pipeline re-implemented.

- **Stage 1 (recommended now-ish):** lift `bible_to_db/reconcile.rs` onto
  `reconcile::orchestrate::reconcile_book`. Teach the shared orchestrator the
  one invariant Bible adds (repoint `page_markers.sort_order` after renumber —
  arguably a latent bug for the others anyway) and let Bible build
  `ReconcileInput`. Deletes ~600 lines and one whole drifted orchestrator;
  Bible gains additive-growth reconcile for free.
- **Stage 2 (only if it earns itself):** a `bible_to_struct` parser emitting
  waist JSON (book nodes with `NodeSource`, chapter nodes, verse markers), so
  fresh inserts go through the one importer. `cross_translation_alignments`
  seeding and drift checks remain bible-specific post-import steps — they are
  domain logic, not pipeline.

### What NOT to do

- **No mega-framework.** Genre parsers stay separate crates with plain code —
  prose, verse, and drama genuinely read differently (ADR 0002 / clarity over
  line count). The unification target is the *waist and below*, plus data-vs-
  code hygiene above it.
- **Don't force Bible's alignment model into the generic translation mode.**
  Sentence-locked (`source_sentence_start_id`) and verse-ref-mapped
  (`cross_translation_alignments`) are different contracts; keep both.
- **Don't abstract the pre-curation one-offs** (`kant1_ocr_to_lines`,
  `ibsen1_xml_to_md`, …). They run once per corpus, produce reviewed artifacts,
  and their value is over; DRYing them buys nothing.

---

## 5. Sequencing & effort

| Phase | Depends on | Size | Risk | Deletes |
|---|---|---|---|---|
| P5 orchestration manifest | — | S–M | low (ops only) | 4×Dockerfile, 4×entrypoint, 4×manifest, script boilerplate |
| P1 one schema | — | S | low (serde-compatible) | kant model.rs |
| P2 one importer | P1 | M | medium (guarded by dry-run + hash parity) | 2×871-line crates |
| P4 parser utils | P1 helps | S | low | ~15 duplicated fns |
| P3 prose genre parser | P1 | M | medium | ~2 crates + 2 half-crates |
| P6a bible on shared orchestrator | — | M | medium-high (data-integrity path) | 642-line orchestrator copy |
| P6b bible through the waist | P1–P3, P6a | L | high | bible fresh-insert path |

Suggested order: **P5 → P1 → P2 → P4 → P3 → P6a**, with hegel1 onboarding as
the live acceptance test of P3, and P6b deferred until something (e.g. a new
compilation-shaped text) makes it pay.

## 6. Open questions for sign-off

1. `prose_md_to_struct` as the genre name for the Kant family — or
   `scholarly_md_to_struct` / keep `kant1_` and accept the misnomer?
2. HTML escaping in the Kant grammar: adopt escape-first, or document the
   divergence?
3. `page_markers.sort_order` renumber (Bible's extra invariant): adopt in the
   shared orchestrator for everyone, or confirm it's Bible-only and document
   why?
4. Normalizing kant's derived layout to flat `output.json` breaks the two
   existing job sync paths — fold into P5's single job image, or do
   separately first?
5. Does P2 land before or after the current ibsen1 review/import work
   completes, to avoid churning the importer mid-corpus?
