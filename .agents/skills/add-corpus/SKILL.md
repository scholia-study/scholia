---
name: add-corpus
description: Checklist for adding a new text corpus to Scholia's ingest pipeline — every file, roster entry, and deploy artifact a corpus needs, plus verification. Use when adding a new text/corpus/ingest (e.g. "add hegel1", "ingest a new work"), or when a corpus is half-wired and something (struct, import, CI, auto-ingest) can't find it.
---

Adding a corpus = **data, never code** (ADR 0006, the narrow waist).
One `common::<corpus>` module + one builder arm + roster entries + three
thin Job manifests (manual, dev, prod). Zero new crates, Dockerfiles, or CI filters. Read
`docs/architecture/overview.md` for the pipeline shape; hegel1
(queued in `assets/hegel1/raw/`) is the standing acceptance test.

## 0. Before any code

- **Licensing**: source must be public-domain or commercially licensed
  — Scholia is commercial; CC BY-NC is blocked. Record provenance.
- **Shape**: a normal authored work (like Kant), NOT a Bible-style
  compilation (ADR 0004 — only the Bible is one).
- **Genre**: prose | poetry | drama → reuse `md_{prose,poetry,drama}_to_struct`.
  A genuinely new genre capability lands once in `text_struct` +
  `struct_to_db` first (ADR 0006) — stop and plan with the user.
- **Layers**: single edition, or source + `--translation` edition?
  Translation editions import with `--source-book-slug` and are
  sentence-locked 1:1 to the source.

## 1. Assets (`assets/<corpus>/`)

- [ ] `raw/` — pre-curation inputs/outputs (gitignored)
- [ ] `curated/` — human-reviewed MD (tracked; the single source of
      truth — converters extract, never infer; missing data is the
      reviewer's call)
- [ ] `derived/` — struct JSONs (gitignored, regenerated)

## 2. Code (parser side)

- [ ] `packages/common/src/<corpus>/` — TOC tables, filename rules,
      book + reference-system metadata (incl. `cite_priority` +
      `cite_template` — the citation-systems standard)
- [ ] Builder arm in the genre parser's `corpus.rs`

## 3. Rosters + orchestration (all three, same commit)

- [ ] `scripts/lib.sh` — add to `SCHOLIA_CORPORA` (import order matters)
- [ ] `scripts/struct.sh` — case arm (both editions if two-layer)
- [ ] `scripts/ingest.sh` — case arm (source edition FIRST, then
      translation with `--source-book-slug <source-slug>`)

## 4. Deploy artifacts (same commit — see the CI trap below)

- [ ] `infra/k8s/jobs/ingest-<corpus>.yaml` — manual escape hatch
      (copy a sibling; `generateName:`, no `DERIVED_HASH`)
- [ ] `infra/k8s/overlays/dev/ingest-jobs/ingest-<corpus>.yaml` — the
      Argo-managed auto-ingest Job (copy a sibling, then rename every
      `<corpus>` occurrence; name suffix `-bootstrap`, `DERIVED_HASH`
      value `""`, keep the `NTFY_URL` env and BOTH annotations:
      sync-wave "1" and `Force=true,Replace=true`)
- [ ] Add it to `infra/k8s/overlays/dev/ingest-jobs/kustomization.yaml`
      resources

The prod overlay manifest is NOT created here — it lands at promotion
time (step 6) with the dev-validated hash.

**CI trap**: the `structs` job builds every corpus from
`struct.sh --list`, and `bump` then patches
`ingest-jobs/ingest-<corpus>.yaml` for every built corpus — if the
overlay manifest doesn't exist the bump job FAILS. Never land the
roster entry without the overlay manifest. (No workflow changes are
needed; that's the point.)

## 5. Verify locally (before merging)

- [ ] `just struct <corpus>` — parser runs clean, derived JSONs exist
- [ ] `bash scripts/derived_hash.sh <corpus>` — prints a hash
- [ ] Import against a THROWAWAY scratch DB only (never local/dev):
      `just db <corpus> --dry-run --database-url postgres://…/scratch`
      then without `--dry-run`; spot-check counts in the report
- [ ] `kubectl kustomize infra/k8s/overlays/dev/ingest-jobs` renders
- [ ] Drop the scratch DB + any temp files afterwards

## 6. Merge → watch it land

- [ ] CI: `structs` uploads `<corpus>/derived@<hash>` + `.complete`;
      `bump` commits the Job rename `-bootstrap` → real hash
- [ ] Argo: `ingest-<corpus>-<hash>` Job completes (wave 1)
- [ ] Run report appears: `scholia-assets-auto/<corpus>/reports/`
      (+ a low-priority ntfy ping)
- [ ] Book renders in the dev reader; cache purge fired
- [ ] Promote to prod once dev content is validated: `just promote`
      copies the image pins AND the dev ingest manifests into the prod
      overlay — but NOT the prod `ingest-jobs/kustomization.yaml`
      resource list, which is curated by hand. Add the new manifest
      there and check `kubectl kustomize infra/k8s/overlays/prod/ingest-jobs`
      renders. MIGRATION TRAP: the ingest Job must land with (or after)
      an api pin that contains any migrations the corpus needs — wave 1
      only orders the sync, it doesn't check the api image is new
      enough. A Job that failed for this reason won't re-run on its
      own (`backoffLimit` 0, name unchanged): after the api rolls out,
      `kubectl delete job` it and Argo selfHeal recreates it.

## 7. Docs

- [ ] `docs/architecture/overview.md` — add the corpus to its genre
      parser's `--corpus` list (both diagrams + prose)
- [ ] `CLAUDE.md` — corpus mentions in the packages section
- [ ] New ADR only if a genre capability or pipeline shape changed
