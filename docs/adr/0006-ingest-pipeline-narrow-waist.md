# 0006. Ingest pipeline: the narrow waist

**Status**: Accepted (implemented 2026-07-02)
**Date**: 2026-07-02

## Context

By mid-2026 the ingest pipeline had grown one corpus at a time: four Kant
parser crates forked from each other, two byte-identical Kant importers
differing only in hardcoded book blurbs, a generic poetry/drama importer
beside them, and a six-artifact orchestration fan-out per corpus (shell
script, npm alias, Dockerfile, job entrypoint, k8s manifest, CI filter) that
had already rotted (ibsen1 undeployable, reload chains missing corpora, a
flag-forwarding bug). A full review (see git history of
`PLAN_INGEST_REFACTOR.md` for the findings and per-phase verification
evidence) consolidated it.

## Decision

**The struct JSON (`text_struct`) is the pipeline's narrow waist. Above the
waist: one parser per *genre*. Below the waist: exactly one of everything.
A *corpus* crosses the waist as data — never as code.**

Corollaries, and how they are realized:

1. **One schema.** `text_struct::model::Output` is the only struct schema —
   the superset of every genre's needs (footnotes, verse indent, speaker
   blocks, sub-work `NodeSource` anchors), optional-by-default so each
   genre's JSON stays quiet. Shared parsing utilities (front matter, dir
   scan, marker resolution) live in `text_struct::parse`.
2. **One importer.** `struct_to_db` imports every struct JSON: fresh insert,
   reconcile-in-place (sentence UUIDs + anchored quotations survive edits),
   and footnote-aware sentence-locked translation mode
   (`--source-book-slug`).
3. **Genre = code, corpus = data.** Three genre parsers —
   `md_prose_to_struct` (kant1|kant3), `md_poetry_to_struct`
   (shakespeare1|milton1), `md_drama_to_struct` (ibsen1) — each driven by a
   `corpus.rs` builder over `common::<corpus>` data modules (TOC tables,
   filename rules, book/reference-system metadata).
4. **One orchestration path.** `scripts/ingest.sh` is the per-corpus import
   manifest, used verbatim by local dev (`just db <corpus>`) and the single
   `jobs/ingest` image (CORPUS env), so local and deploy cannot drift. The
   corpus roster lives once, in `scripts/lib.sh` `SCHOLIA_CORPORA`;
   `just db-reload` iterates `ingest.sh --list`. One CI filter, thin
   per-corpus k8s manifests over one image. Derived structs are uniformly
   `assets/<corpus>/derived/output.json` (+ `translation_output.json`).
5. **The new-text test.** Adding a text of an existing genre requires:
   curated MD, one `common::<corpus>` module, one `corpus.rs` builder arm,
   one word in `SCHOLIA_CORPORA` + case arms in `scripts/{ingest,struct}.sh`,
   and one thin k8s Job manifest (the only per-corpus deploy artifact — it
   points at the shared image). Zero new crates, Dockerfiles, CI filters, or
   front-door entries. A new *genre capability* lands once in `text_struct` +
   `struct_to_db` and every genre inherits it.

**Bible stays outside the waist, deliberately.** Its three special rules are
essential, not incidental: verse-ref alignment
(`cross_translation_alignments`) is a deliberately looser contract than
sentence-locking (verse counts legitimately drift across translations); its
input is fetched API JSON, not curated MD; canonical drift policing is Bible
domain logic. It shares the correctness-critical primitives
(`reconcile::{align,deps,hash}`) and the orchestration shell
(`bible_import.sh`, same job/CI shape). Its remaining hand-rolled reconcile
orchestrator (~640 lines mirroring `reconcile::orchestrate`) is known
duplication, kept because bible content is frozen — that reconcile path
effectively never runs. **Revisit triggers:** bible content starts changing;
the shared orchestrator gains a capability bible needs (e.g. the
`page_markers.sort_order` renumber it alone maintains); or a second
compilation-shaped text arrives.

## Verification

Every phase was proven behavior-preserving against throwaway databases: all
eight editions (kant1/kant3 source + translation, shakespeare1, milton1,
ibsen1 source + translation) imported by the pre-refactor code and by the
final code produce **byte-identical normalized dumps** (72,028 lines —
content hashes, natural keys, translation links, footnotes, markers; only
UUIDs/timestamps excluded), and the final importer's dry-run reconcile
against books written by the old importers short-circuits on root hash
(deploy-safe). Struct JSONs were byte-compared at every intermediate step.

## Consequences

- Net ≈ −4,700 lines; 18 crates → 13; five importers → two (`struct_to_db`,
  `bible_to_db`); the `md_<genre>_to_struct` naming scheme makes roles
  legible from names alone (see `packages/README.md` for the diagram).
- hegel1 (queued in `assets/hegel1/raw/`) is the standing acceptance test of
  the new-text rule.
- Two real bugs surfaced and fixed en route: the pnpm `--` separator broke
  flag-forwarding in three import scripts; an 1873 compositor error (missing
  period, `010_kj` p. 255) was flagged by the drama parity guard.
- Known deferred items: the `indent` field is not part of the sentence
  content hash (indent-only edits need `--full-rewrite`); the Kant markdown
  grammar does not HTML-escape (unlike `text_struct::html`) — deliberate
  for now, revisit if curated Kant MD ever carries user-visible `<`/`&`.
