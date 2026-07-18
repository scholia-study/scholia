# PLAN: GitOps-triggered ingestion

Status: implemented 2026-07-17 (both phases, one branch) — remaining
before merge: create the CI key pair in the Hetzner console and store
it as Actions secrets `ASSETS_BUCKET_ACCESS_KEY_ID` /
`ASSETS_BUCKET_SECRET_ACCESS_KEY`. The bucket groundwork was already
**done** — `scholia-assets-auto` provisioned with its expiry rule
(2026-07-13). Extends the CI → git
write-back → Argo pattern from `PLAN_ARGOCD.md` to data ingestion: a
merge touching a corpus's curated MD (or its parser) should build its
structs and run the DB import for that corpus, with no manual steps.

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

The settled shape: **one workflow** (`build.yml` extended), a new
`structs` job running in parallel with the image matrix, and a single
**convergent** `bump` job joining both — one commit patches image tags
and ingest-Job hashes together, so Argo gets one atomic sync. Nothing
in the Rust or app code changes at all.

```
changes ─→ build (image matrix) ──┐
       └─→ structs (parse → hash → upload) ─┴─→ bump (one commit: tags + hashes)
```

## The core idea: derived-output hash as the deploy "tag"

For app code, the GitOps contract is "overlay says image `main-<sha>`
→ cluster runs it." The ingest equivalent:

> Overlay says corpus content-state `<hash>` has been ingested →
> cluster has run that Job.

The hash is a digest of the corpus's **derived struct JSON(s)** — the
built artifact, not the source tree. An earlier draft used the git
tree hash of `assets/<corpus>/curated`, but derived JSON is a function
of curated MD **and parser code** (TOC tables, ref-system meta, cite
templates all live in `common::<corpus>`): a parser change alters the
output without touching curated, which under input-hashing either
never re-ingests or overwrites a hash-keyed path with different bytes.
Output-addressing gives:

- The hash changes iff the import input actually changes — a parser
  refactor that emits identical JSON is a no-op, no Job fired.
- A revert to previous content reproduces the old hash → no-op, which
  is correct since reconcile makes ingest idempotent.
- Identical content can never be ingested "twice" — same hash, same
  Job name, no new resource.
- Requirement: parser output must be deterministic (verify item 3).

## Phase 1 — CI builds structs (close the manual gap)

Why one workflow and not a second one chained on `workflow_run`: Build
doesn't trigger on `assets/**` (the chain would need a trigger-relay
hack), `workflow_run` has no diff base, and two workflows committing
to `main` race on push. A single final bump commit avoids all three
and makes image-tag + Job-hash updates atomic (see rejected
alternatives).

**`structs` job** (parallel with `build`, not after it — struct
building uses the checkout's parsers via cargo, never the images):

1. Runs **unconditionally** on every workflow run — no path filter.
   Unchanged corpora hash the same and no-op, and unconditional runs
   are what make `bump` convergent: a run cancelled by
   `cancel-in-progress` loses nothing, the next run reconciles the
   overlay to the current checkout. (`assets/*/curated/**` and
   `scripts/**` added to `on.push.paths` so curated-only pushes
   trigger the workflow at all — the latter also fixes a pre-existing
   gap where `scripts/ingest.sh`-only changes never triggered it.)
   `workflow_dispatch` inputs give the manual controls: `images`
   (false = structs-only recovery run) and `corpora` (limit the
   sweep).
2. Runs `bash scripts/struct.sh <corpus>` for every corpus from
   `struct.sh --list` — the script stays the canonical per-corpus
   manifest, so CI and local builds can't drift. Plain cargo on the
   runner: `Swatinem/rust-cache`, toolchain pinned in the workflow
   (1.96.0, matching local) so hashes don't depend on the runner's
   rustc. No image: the parsers have no cluster consumer — images are
   for what the cluster runs.
3. Hashes each corpus's derived JSON(s) via `scripts/derived_hash.sh`
   — THE hash definition (12 hex chars over file bytes + paths);
   uploads to `scholia-assets-auto/<corpus>/derived@<hash>/` if that
   key is absent. Immutable artifacts, matching the `main-<sha>`
   image-tag philosophy: a running Job can never have its input
   swapped underneath it, and any past content state stays
   re-runnable.

**`bump` job** (join point, `needs: [build, structs]`):

- **Convergent, not event-driven**: it checks out `main` anyway, so it
  recomputes desired state (output hash per corpus, image tags from
  the matrix), diffs against the overlay, and patches whatever
  differs. A run cancelled by `cancel-in-progress` loses nothing —
  the next run reconciles it. This also absorbs the paths-filter
  no-base caveat; filters remain a build-cost optimization only.
- Skipped-`needs` gotcha: a curated-only push skips `build`, an
  app-only push skips `structs`, and a job whose `needs` includes a
  skipped job is itself skipped by default. `bump` needs
  `if: ${{ !cancelled() && needs.build.result != 'failure' &&
  needs.structs.result != 'failure' }}` — run when a parent was
  skipped, never when one failed. Test both push shapes.
- Fail-closed coupling, deliberate: a merge mixing curated edits with
  app code that fails to build skips `bump` — content doesn't move
  while tooling is broken; the ingest lands when the build is fixed.

A side benefit of CI-side struct building: the parsers are strict, so
a malformed curated edit fails the workflow — red X on the merge,
before anything ships.

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

## Phase 2 — Argo runs the Jobs (close the trigger gap)

New Argo-managed Job manifests live in
`infra/k8s/overlays/dev/ingest-jobs/` (the manual
`infra/k8s/jobs/*.yaml` stay as the kubectl escape hatch):

- **Job name embeds the content hash**: `ingest-kant1-<hash12>`. Jobs
  are immutable, but a new name is a new resource — Argo creates it
  and **prunes** the old one; no `ttlSecondsAfterFinished` (selfHeal
  would resurrect — i.e. re-run — a TTL-deleted Job; prune-on-rename
  is the cleanup). Manifests land with a `-bootstrap` name + empty
  hash: the adoption sync runs one reconcile from the manual bucket,
  then the first CI bump takes over.
- **The ingest image stays unpinned** (`:main` + pull `Always`),
  unlike the api/web/proxy tag pins: the Job's identity is its
  content hash. Pinning would mutate an immutable Job field on every
  importer rebuild — a sync error — or force all corpora to re-run on
  a code-only change. Tooling rides the latest built image; a
  content-triggered Job always runs with the importer that `main`
  last shipped. (Revisit with digest-pinning if this ever bites.)
- **Entrypoint pulls from the hash-keyed path in the auto-ingest
  bucket** (`jobs/ingest/entrypoint.sh`), hash passed as
  `DERIVED_HASH` env, set in the same manifest — with an existence
  check that names the expiry-recovery path on failure. When the hash
  env is unset/empty the entrypoint keeps its old behavior (pull
  `scholia-assets/<corpus>/derived`), so the manual `kubectl create`
  flow and local `just db <corpus>` stay usable as-is.

`bump` patches the ingest Job manifests in the **same commit** as any
image-tag bumps (it's the same job now, by construction).

Day-to-day becomes: edit curated MD → merge to `main` → CI builds
structs + uploads → CI commits the hash bump → Argo syncs → Job runs →
`struct_to_db` reconciles → cache purge fires (already wired via
`CACHE_PURGE_URL`). `git log` on the Job manifest is the ingest audit
trail.

### Sequencing

- Sync waves: postgres/api (migrations via the `api migrate` init
  container) at wave 0, ingest Jobs at wave 1 — so a commit shipping
  both a migration and content changes migrates before importing.
- Jobs get health config so Argo waits for completion: sync **blocks**
  on ingest success — a failed reconcile shows `scholia-dev` Degraded,
  which is exactly the visibility a reconcile abort needs (open
  item 1 adds notification on top).
- Source-before-translation ordering within a corpus is already inside
  `scripts/ingest.sh`; unchanged.

### Concurrent ingests of one corpus — resolved

`struct_to_db` opens **one transaction per invocation**
(`packages/struct_to_db/src/import.rs` — `pool.begin()` up front,
single commit on either the reconcile or fresh-insert path), so each
edition import is atomic. A corpus with a translation is two
sequential invocations in `ingest.sh`, so corpus-level atomicity does
not exist — but the bump renames the Job manifest, and Argo prunes the
old Job, killing it if still running: single-slot-with-preemption
semantics. The kill is safe (no partial book can commit), and the new
Job re-runs both editions, the already-current source import
no-opping via reconcile. Residual risk: a transient source/translation
mixed state in dev until the newer Job completes. Accepted.

## Open items

1. **Failure surfacing is a first-class requirement.** Reconcile
   aborts are a feature — the aligner intentionally bails on ambiguous
   sentence rewrites (sim < 0.90) and expects the edit to land as two
   passes. Automating the trigger means a human must reliably notice
   the failure: Argo Degraded state (sequencing above) plus a
   notification hook — nobody watches the Argo UI between deploys.
2. **Bucket expiry vs replayable manifests.** The overlay permanently
   references `derived@<hash>`, but the artifact expires after 30
   quiet days — a fresh cluster build or a re-created Job then pulls a
   missing key. Recovery is one `workflow_dispatch` (the `structs` job
   runs unconditionally and re-uploads current hashes). Document in
   `infra/argo/README.md`.
3. **Parser determinism — verified locally 2026-07-17**: two
   independent full runs of the prose (kant1) and drama (ibsen1)
   parsers produced byte-identical hashes. Residual risk is
   CI-vs-local platform variance; the first CI run's hashes can be
   compared against `scripts/derived_hash.sh` output locally.
4. **Prod later.** The ingest Job manifests stay **per-overlay** —
   deliberately NOT shared via base. A hash-named manifest is the
   environment's content *pin*, exactly like the overlay's `newTag:`
   image pins: sharing it would make every dev bump change prod's
   rendered state on the same merge, defeating the gate. Promotion =
   copy `overlays/dev/ingest-jobs/` over `overlays/prod/ingest-jobs/`
   (plus image pins if promoting the app in the same gesture) and
   commit — prod then runs the byte-identical `derived@<hash>`
   artifacts dev already validated, and `git log` on the prod dir is
   prod's own content audit trail. The five duplicated Job files per
   overlay are mechanical copies overwritten wholesale by promotion;
   a kustomize shared-template + name-patch scheme would fight the
   tool (the patch target changes every bump). Caveats: the 30-day
   bucket expiry bounds the promotion window (the promote step should
   check `derived@<hash>` still exists; recovery is the usual
   `workflow_dispatch`), quotation anchoring is live user data there
   (gate stays manual), and a book reconcile is one long transaction
   touching sentences that live quotation writes reference — promote
   in a quiet window.

## Rejected alternatives

- **Curated MD + parsers baked into the ingest image** (image tag =
  content version). Tempting — it deletes the bucket/hash machinery
  entirely — but: per-corpus granularity dies (one image serves all
  corpora, so any edit re-runs every Job or reintroduces input-side
  hashing and its parser-change blind spot); parse failures surface
  in-cluster after the image shipped instead of failing CI pre-merge;
  every typo pays a docker build + registry push; and "one image,
  CORPUS env" quietly becomes "importer code plus a snapshot of all
  texts." The parser-binaries-only variant is the worst of both: the
  image rebuilds on parser changes and the Job still needs a
  content-fetch path.
- **Separate ingest workflow chained on `workflow_run: Build`.**
  Build doesn't trigger on `assets/**`, so the chain needs a
  trigger-relay; `workflow_run` supplies no diff base; and two
  workflows committing to `main` race on push. The in-workflow join
  gets the same ordering with one atomic commit.
- **Argo Events + Argo Workflows.** Webhook → build → import as a real
  DAG, with retries and a UI. More powerful, but a whole new subsystem
  on a small cluster — and the CI-writes-back-to-git pattern already
  gives the trigger, audit trail, and ordering for free. Revisit if
  ingest orchestration outgrows "one Job per corpus."

## What landed (2026-07-17)

- `.github/workflows/build.yml` — curated/scripts trigger paths,
  unconditional `structs` job, convergent single-commit `bump`,
  `workflow_dispatch` inputs (`images`, `corpora`).
- `scripts/derived_hash.sh` — the shared hash definition.
- `jobs/ingest/entrypoint.sh` — `DERIVED_HASH` pull mode with
  existence check + expiry-recovery hint.
- `infra/k8s/overlays/dev/ingest-jobs/` — five Argo-managed Job
  manifests + kustomization (rationale documented there), wired into
  the dev overlay.
- `infra/k8s/jobs/*.yaml` headers — reframed as the manual escape
  hatch; `infra/argo/README.md` + `application-dev.yaml` — ownership
  table, day-to-day flows, re-run/recovery notes.

Still to do by hand: the CI key pair (Hetzner console — keys aren't a
Terraform resource) → Actions secrets `ASSETS_BUCKET_ACCESS_KEY_ID` /
`ASSETS_BUCKET_SECRET_ACCESS_KEY`; optionally a deny-CI-key bucket
policy on `scholia-assets` (the repo is public). Cluster Jobs reuse
the existing `assets-bucket` Secret. *(Already done earlier: the
bucket in Terraform + `scripts/assets_lifecycle.sh` for the expiry
rule.)*
