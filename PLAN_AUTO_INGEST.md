# PLAN: GitOps-triggered ingestion

Status: under review. The bucket groundwork is **done** —
`scholia-assets-auto` is provisioned with its expiry rule
(2026-07-13); the CI and Argo phases plus the open questions below are
still to be agreed. Extends the CI → git write-back → Argo pattern
from `PLAN_ARGOCD.md` to data ingestion: a merge touching a corpus's
curated MD should build its structs and run the DB import for that
corpus, with no manual steps.

Scope: struct-importer corpora only (`SCHOLIA_CORPORA` in
`scripts/lib.sh`). **Bible is excluded** — its source isn't in git, it
has its own importer + fetch flow, and KJV seeding order makes it a
rare, deliberate operation. Dev cluster first; prod gated later.

## Bottom line

The repo is already most of the way there. CI path-filters per
component and commits write-backs to the Argo-managed overlay; the
ingest image is corpus-parameterized (`CORPUS` env → shared
`scripts/ingest.sh` manifest); `struct_to_db` reconciles in place, so
re-running an import is safe by design. Two gaps:

1. Struct building is a manual local step (`just struct <corpus>` +
   `just assets-sync`).
2. Ingest Jobs are deliberately outside Argo (`infra/k8s/jobs/`, run
   via `kubectl create` — decision 3 in `PLAN_ARGOCD.md`, which this
   plan revisits).

Nothing in the Rust or app code changes at all.

## The core idea: content hash as the deploy "tag"

For app code, the GitOps contract is "overlay says image `main-<sha>`
→ cluster runs it." The ingest equivalent:

> Overlay says corpus content-state `<hash>` has been ingested →
> cluster has run that Job.

Git provides the hash for free: `git rev-parse HEAD:assets/kant1/curated`
is the **tree hash** of the curated directory. It changes only when
curated content actually changes, and is identical across unrelated
commits. Preferred over the commit SHA:

- No spurious re-ingests when unrelated files change in the same commit.
- A revert to previous content reproduces the old hash → no-op, which
  is correct since reconcile makes ingest idempotent.
- Identical content can never be ingested "twice" — same hash, same
  Job name, no new resource.

## Phase 1 — CI builds structs (close the manual gap)

Add per-corpus path filters to `.github/workflows/build.yml`:
`assets/kant1/curated/**` → `kant1`, etc. For each affected corpus, a
CI job:

1. Runs `just struct <corpus>` (cargo-builds the `md_*_to_struct`
   parser — cacheable, same as the existing Rust builds).
2. Uploads the derived JSON to the **dedicated auto-ingest bucket**
   `scholia-assets-auto` under a **hash-keyed path**,
   `scholia-assets-auto/kant1/derived@<treehash>/`.

Immutable artifacts, matching the `main-<sha>` image-tag philosophy: a
running Job can never have its input swapped underneath it, and any
past content state stays re-runnable.

### Why a second bucket (not `scholia-assets`)

`scripts/assets_sync.sh` is a **mirror with delete semantics** — local
is canonical, and remote files that don't exist locally get deleted.
CI-written `derived@<hash>/` prefixes exist only in the bucket, so the
next manual `just assets-sync` would wipe them all. Options:

- Shared bucket + rclone excludes in the sync script: breaks the
  script's clean "bucket matches local" contract and is fragile — one
  forgotten exclude silently deletes CI artifacts. Rejected.
- **Second bucket, CI-owned (chosen): `scholia-assets-auto`.**
  `scholia-assets` stays exactly as it is: manually mirrored, feeds
  manual ingest runs. The new bucket is written only by CI and read
  only by auto-ingest Jobs. The manual workflow doesn't change at all.
  Created via Terraform alongside the existing `aws_s3_bucket.assets`
  in `infra/terraform/shared/main.tf` (not by hand — keeps both
  buckets in one place).

Credentials caveat: Hetzner S3 keys are **project-wide** — every key
pair can read/write every bucket in the project; there are no scoped
keys. So a dedicated CI key pair buys independent revocation, not
isolation: leaked, it can still write `scholia-assets`. Optional
hardening (worth it — the repo is public): a bucket policy on
`scholia-assets` that denies the CI key. The Jobs' existing
`assets-bucket` Secret already reads the new bucket — no new k8s
Secret needed.

**Provisioned 2026-07-13.** The bucket is Terraform
(`infra/terraform/shared/main.tf`); its 30-day expiry rule (old hashes
are re-buildable from git) is `just assets-lifecycle` — the aws
provider's lifecycle resource can't converge against Hetzner (its
post-PUT read-back poll expects fields Hetzner never echoes; times out
in every rule shape, tested on v5.100.0 — aws/aws-sdk-go-v2#3285).

Rejected alternative: baking curated MD into the ingest image and
building structs inside the Job. It makes the "one image, CORPUS env"
design content-versioned and couples image rebuilds to text edits. The
bucket flow is the right narrow waist; CI just automates the upload.

## Phase 2 — Argo runs the Jobs (close the trigger gap)

Move the per-corpus Job manifests from manual-kubectl territory into
the Argo-managed dev overlay, with two changes:

- **Job name embeds the content hash**: `ingest-kant1-<treehash7>`.
  Jobs are immutable, but a new name is a new resource — Argo creates
  it; the old completed one ages out via the existing
  `ttlSecondsAfterFinished` plus Argo prune.
- **Entrypoint pulls from the hash-keyed path in the auto-ingest
  bucket** (`jobs/ingest/entrypoint.sh`), hash passed as env, set in
  the same manifest. When the hash env is unset the entrypoint keeps
  its current behavior (pull `scholia-assets/<corpus>/derived`), so
  the manual `kubectl create` flow and local `just db <corpus>` stay
  usable as-is.

CI's existing `bump` job — which already commits image-tag bumps to
`overlays/dev/kustomization.yaml` — additionally patches the ingest
Job manifest for each affected corpus (name + hash env), in the same
commit as any image bumps.

Day-to-day becomes: edit curated MD → merge to `main` → CI builds
structs + uploads → CI commits the hash bump → Argo syncs → Job runs →
`struct_to_db` reconciles → cache purge fires (already wired via
`CACHE_PURGE_URL`). `git log` on the Job manifest is the ingest audit
trail.

### Sequencing

- Sync waves: postgres/api (migrations via the `api migrate` init
  container) at wave 0, ingest Jobs at wave 1 — so a commit shipping
  both a migration and content changes migrates before importing.
- Jobs need hook/health config so Argo waits for completion rather
  than marking Synced immediately. Decide: should sync **block** on
  ingest success (failed reconcile → `scholia-dev` shows Degraded) or
  run fire-and-forget with separate alerting? Leaning toward blocking
  — a red app is exactly the visibility a reconcile abort needs.
- Source-before-translation ordering within a corpus is already inside
  `scripts/ingest.sh`; unchanged.

## Open questions (resolve in review)

1. **Failure surfacing is a first-class requirement.** Reconcile
   aborts are a feature — the aligner intentionally bails on ambiguous
   sentence rewrites (sim < 0.90) and expects the edit to land as two
   passes. Automating the trigger means a human must reliably notice
   the failure (Argo Degraded state, plus maybe a notification hook).
2. **Concurrent ingests of one corpus.** Two quick merges to the same
   corpus create two Jobs that could overlap mid-reconcile. Options:
   (a) verify `struct_to_db` wraps each book's reconcile in a
   transaction and accept the overlap; (b) single-slot Job per corpus
   replaced on change (`Replace` + `Force` sync options) — but Replace
   kills a running Job, so the transactional guarantee needs checking
   either way. The one real design question in this plan.
3. **Prod later.** Same pattern per overlay, but prod ingest should be
   gated (promote the hash bump to the prod overlay manually or via a
   release PR) rather than auto-on-merge — quotation anchoring is live
   user data there.
4. **paths-filter base caveat.** `build.yml` already notes paths-filter
   has no useful base on push-to-main; the same caveat applies to the
   new corpus filters (worst case: a spurious rebuild/re-upload, which
   the hash-keyed paths + hash-named Jobs make harmless).

## Rejected alternative: Argo Events + Argo Workflows

Webhook → build → import as a real DAG, with retries and a UI. More
powerful, but a whole new subsystem on a small cluster — and the
CI-writes-back-to-git pattern already gives the trigger, audit trail,
and ordering for free. Revisit if ingest orchestration outgrows
"one Job per corpus."

## Rough sizing

- Phase 1: CI (path filters, a struct-build job, an rclone upload
  step) + a dedicated CI key pair (generated in the Hetzner console —
  keys aren't a Terraform resource) stored as Actions secrets;
  optionally a deny-CI-key bucket policy on `scholia-assets`. Jobs
  reuse the existing `assets-bucket` Secret. *(Already done: the
  bucket in Terraform + `scripts/assets_lifecycle.sh` for the
  expiry rule.)*
- Phase 2: Job manifests + overlay, `jobs/ingest/entrypoint.sh`
  (hash-keyed pull path from the auto-ingest bucket, falling back to
  current behavior when unset), the bump script in `build.yml`, and
  updating `infra/argo/README.md` (Argo now owns ingest Jobs — revises
  the "stays manual" table).
