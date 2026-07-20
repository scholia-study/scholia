# 0007. Auto-ingest: content-hash GitOps

**Status**: Accepted (implemented 2026-07-17, verified in dev 2026-07-18)
**Date**: 2026-07-18

## Context

Struct imports were manual: `just struct <corpus>` + `just assets-sync`
locally, then `kubectl create` a Job on the cluster. Everything else
already deployed via CI → git write-back → Argo (decision record in
git history — PLAN_ARGOCD). This extends that pattern to data:
a merge touching a corpus's curated MD — or its parser — builds structs
and runs the DB import with no manual steps. Scope: struct-importer
corpora only; **Bible is excluded** (own importer + fetch flow, source
not in git, KJV seeding order makes it a rare deliberate operation).
Current flow: `docs/architecture/overview.md`. Ops: `infra/argo/README.md`.

## Decision

**The overlay pins a per-corpus content hash of the derived structs,
exactly as it pins image tags; CI converges the pins, Argo runs
hash-named Jobs.**

1. **Output-addressed, not input-addressed.** The hash
   (`scripts/derived_hash.sh`, 12 hex over the derived JSONs' bytes +
   paths) is computed from the *built artifact*, not the curated source
   tree — derived JSON is `f(curated MD, parser code)`, and an input
   tree hash misses the parser half: parser/meta changes would either
   never re-ingest or overwrite a hash key with different bytes. Output
   addressing changes the hash iff the import input changes, makes
   no-op parser refactors free, and requires parser determinism —
   verified: two local runs and CI produce byte-identical hashes.
2. **One workflow, convergent bump.** `build.yml` runs the image
   matrix and a `structs` job in parallel; `structs` is
   **unconditional** (every run builds all corpora, hashes, uploads
   missing keys) so the joining `bump` job can converge the overlay to
   the checkout regardless of which run got cancelled
   (`cancel-in-progress`) or what an event's path filter saw. One
   commit carries image tags + Job hashes — Argo never sees a new hash
   without the image bump that shipped alongside it. Structs build on
   the runner via `scripts/struct.sh` (same script as local — no
   drift); no struct-builder image exists because parsers have no
   cluster consumer.
3. **Immutable artifacts in a CI-owned second bucket.**
   `scholia-assets-auto/<corpus>/derived@<hash>/`, 30-day expiry (any
   hash is re-buildable via `workflow_dispatch`). Separate from
   `scholia-assets` because `assets_sync.sh` is a mirror with delete
   semantics — CI-only prefixes in a shared bucket would be wiped by
   the next manual sync.
4. **Argo-managed, hash-named Jobs** (`overlays/dev/ingest-jobs/`):
   `ingest-<corpus>-<hash12>`, sync-wave 1 (migrations at wave 0),
   `backoffLimit: 0`, **no TTL** (selfHeal would resurrect a
   TTL-deleted Job; prune-on-rename is the cleanup), image
   **unpinned** (`:main` + pull Always — the Job's identity is its
   content hash; pinning would mutate an immutable Job field on every
   importer rebuild or re-run all corpora for a code-only change).
   Concurrency is safe by construction: `struct_to_db` commits one
   transaction per edition, and Argo's prune of the renamed Job
   preempts a still-running predecessor; the replacement re-runs both
   editions, reconcile no-opping the current one.
5. **Per-overlay manifests are environment content pins.** Never
   shared via base — sharing would deploy dev bumps to prod on merge.
   Prod (when it exists) moves only by *promotion*: copy
   `overlays/dev/ingest-jobs/` into the prod overlay; prod then runs
   the byte-identical bucket artifacts dev validated. The 30-day
   expiry bounds the promotion window (check the key exists; recovery
   is the usual dispatch).

Failure surfacing is a feature: reconcile aborts (ambiguous rewrite,
sim < 0.90) are designed outcomes — red Job → app Degraded → land the
edit as two passes.

## Rejected alternatives

- **Bake curated MD + parsers into the ingest image** (image tag =
  content version). Deletes the bucket/hash machinery, but per-corpus
  granularity dies (any edit re-runs every corpus, or input-hashing
  returns with its parser blind spot), parse failures surface
  in-cluster after the image shipped instead of failing CI pre-merge,
  and every typo pays a docker build. Parser-binaries-only variant:
  worst of both.
- **Separate ingest workflow chained on `workflow_run: Build`.** Build
  doesn't trigger on `assets/**` (needs a relay), `workflow_run` has
  no diff base, and two workflows committing to `main` race.
- **Input tree-hash addressing** (`git rev-parse HEAD:assets/…`).
  Misses parser-code inputs — see decision 1.
- **Argo Events + Argo Workflows.** A real DAG with retries and a UI,
  but a whole new subsystem on a small cluster; CI-writes-back-to-git
  already gives trigger, audit trail, and ordering. Revisit if
  orchestration outgrows "one Job per corpus."

## Consequences

- Day-to-day: edit curated MD → merge → ingested; `git log` on a Job
  manifest is that corpus's ingest audit trail. Unchanged re-runs are
  free end to end (no upload, no commit, no Job).
- Escape hatches stay live: local `just struct`/`just db`, manual
  bucket + `kubectl create -f infra/k8s/jobs/…` (no `DERIVED_HASH` →
  manual bucket), `workflow_dispatch` (`images=false`, `corpora=…`).
- A one-line MD fix still costs a full O(book) reconcile — the
  tier-2 incremental spec
  (`docs/architecture/reconcile-incremental-hashing.md`) is the
  answer, its motivation now narrowed to changed-corpus cost.
- Hetzner S3 keys are project-wide: the dedicated CI key buys
  revocation, not isolation. **Open:** a deny-CI-key bucket policy on
  `scholia-assets` (repo is public), and a notification hook on
  Degraded so reconcile aborts are noticed without watching the Argo
  UI.
