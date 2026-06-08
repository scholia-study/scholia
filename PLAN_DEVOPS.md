# Scholia — DevOps Plan

Design notes and decisions for taking Scholia to production. Distilled from
the design grill on 2026-05-04. This is the reference doc for execution;
no code yet, just decisions + rationale.

---

## 0. Pre-deployment refactor (do first)

Three coupled changes that must land **before any cluster bringup**, all on
one branch:

### ✅ 0.1 Move all backend routes under `/api/*`

Current state: routes live under `/auth/*`, `/billing/*`, `/api/*`, and
`/webhooks/stripe`. Production deploys behind a single hostname with
path-based routing — having a uniform `/api/*` prefix makes ingress rules
simpler and avoids per-prefix routing logic.

Touches:

- Every `#[utoipa::path(path = "...")]` annotation in `packages/api/src/handlers/`
- `GITHUB_REDIRECT_URI` env var (now `…/api/auth/github/callback`)
- The Stripe webhook URL (now `…/api/webhooks/stripe`) — must update
  Stripe Dashboard after deploy
- `openapi.json` regenerates → run `pnpm codegen` to refresh frontend client
- Frontend `fetcher.ts` BASE_URL no longer hardcoded (see 0.2)

### ✅ 0.2 Runtime config injection (profile-registry model)

Replace build-time Vite env vars with runtime injection so a single
container image works across all environments.

**Pattern — all profiles in TypeScript, only the active profile is injected:**

- `apps/web/src/config.ts` — profile registry: a typed map keyed by profile
  name (`local`, `local-proxy`, `dev`, `prod`) holding all
  environment-specific values (`API_BASE_URL`, `STRIPE_PUBLISHABLE_KEY`,
  …). The active profile is read from `window.__ENV__.APP_PROFILE` and
  selects the matching entry. Default is `"local"` when `__ENV__` is
  absent.
- The TanStack Start root route (`__root.tsx`) reads
  `process.env.APP_PROFILE` at SSR render time and emits an **inline**
  `<script>window.__ENV__ = { APP_PROFILE: "..." };</script>` in
  `<head>`. The browser runs it synchronously before the bundle, so
  `config.ts` sees the value immediately. No separate `/config.js`
  fetch.
- `APP_PROFILE` is set on the **web Deployment env** (the Node SSR
  pod), not on the proxy. One image per environment by env-var, same
  pattern as `API_BASE_URL` and `CACHE_PURGE_URL`. The proxy no longer
  has any role in runtime config rendering.
- Local-dev mode selection (root `package.json` scripts):
  - `pnpm dev` — api + web only, no `APP_PROFILE` set → web falls back
    to `"local"` profile → fetches go to `http://localhost:4000`.
    Browser hits Node SSR on `:3000` directly. Fastest iteration loop.
  - `pnpm dev:all` — api + web + proxy together, with
    `APP_PROFILE=local-proxy` baked into the script. Web injects
    `local-proxy` → fetches go same-origin → proxy routes `/api/*` to
    Rust. Browser hits the proxy on `:8000`. Use this to exercise the
    full cache + PURGE path.
  - `pnpm dev:proxy` — just the proxy (already-running api + web on
    host ports). Only useful if the web side was started with
    `APP_PROFILE=local-proxy` separately.

**Why this shape over per-key envsubst:**

- Settings live in TypeScript next to the rest of the code; no parallel
  `.env` / template duplication
- Type safety on profile names (`Profile = "local" | "dev" | "prod"`)
- One injected variable (`APP_PROFILE`) instead of N
- Adding a new profile is a code change, reviewed in PRs

**Sensitive-data rule:** the registry ships to the browser. Only public
values belong here (Stripe publishable keys, API base URL, analytics
IDs). Secret keys stay server-side.

**Migrates two existing values:**

- `import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY` → `config.STRIPE_PUBLISHABLE_KEY`
- Hardcoded `BASE_URL = "http://localhost:4000"` in `fetcher.ts` → `config.API_BASE_URL`

### ✅ 0.3 Migrations bootstrap (sqlx)

Replaced the destructive `db_reset.sh`-only flow with a proper migration
story before any prod data exists.

**Landed:**

- `db/001_schema.sql` → `db/migrations/0000_initial.sql`. Naming
  convention: `NNNN_<semantic-name>.sql`, sequential starting at `0000`.
- `sqlx-cli` installed locally:
  `cargo install sqlx-cli --no-default-features --features postgres,rustls`
- Migrations embedded into the API binary via `sqlx::migrate!("../../db/migrations")`
  in `apps/api/src/migrate.rs`.
- API binary subcommand `api migrate` runs migrations and exits (cluster
  init container path). `default-run = "api"` set in `Cargo.toml` so
  `cargo run -p api -- migrate` resolves the main binary.
- `scripts/db_reset.sh` rewritten: drops schema, then `sqlx migrate run`.
  sqlx-cli (not the api binary) is used here because the api crate's
  compile-time `sqlx::query!` macros need a live schema — dropping the
  schema first creates a chicken-and-egg if you try to rebuild the
  binary. In production the same problem doesn't arise because builds
  use committed `.sqlx` offline metadata. Full rationale in
  `docs/architecture/database-migrations.md`.
- `_sqlx_migrations` table tracks applied migrations + checksums; sqlx
  refuses to re-run an edited migration.

**Rule (non-negotiable post-launch):** migrations are **append-only**.
Every schema change is a new migration file. Never edit a previously-applied
file. Multi-step changes (add nullable col → backfill → make NOT NULL)
become multiple migrations, not one edit.

### ✅ 0.4 Update auto-memory note

The `reference-db-reset` auto-memory now points at `pnpm db:reset` →
sqlx migrations in `db/migrations/`, and a companion
`feedback-migration-naming` memory captures the `NNNN_<semantic-name>`
convention starting at `0000`. CLAUDE.md `Database — db/migrations/`
section rewritten to match.

---

## 1. Infrastructure architecture

### 1.1 Cloud + topology

- **Provider**: Hetzner Cloud
- **Region**: Falkenstein (closest to EU/Norway audience and Stripe EU endpoints)
- **Topology**: two single-node k3s clusters, one per environment

| | dev cluster | prod cluster |
|---|---|---|
| **VPS** | cx23 (2 vCPU, 4GB, 40GB disk) | cx43 (8 vCPU, 16GB, 160GB disk) |
| **OS** | Ubuntu 24.04 LTS | Ubuntu 24.04 LTS |
| **Hostname** | `dev.scholia.study` | `scholia.study` |

(Slugs are Hetzner's `cx_3` Intel line: cx23→cx33→cx43→cx53.)

**Why two clusters, not namespaces in one cluster:**

- Physical isolation — dev mistakes can't take prod down
- Independent k3s upgrades (test on dev first, schedule prod separately)
- Distinct Stripe webhook URLs by hostname (no routing-rule collisions)
- Doubles K8s muscle (two kubeconfig contexts, real cross-cluster practice)

**Why cx43 prod (8 vCPU / 16 GB / 160 GB):** sized for launch plus the
medium term rather than minimal-at-start. Committed RAM is only
~2.5-3.5 GB (k3s + Postgres + Rust API + Node SSR + nginx-cache +
Traefik + cert-manager), so 16 GB leaves wide headroom for traffic
growth, in-cluster ingest/build spikes, and a future light
observability stack without an early resize (cx53 is the in-place step
up if ever needed). **Reserved 2026-06-08** — the box runs idle (no
app) until the prod app-layer bootstrap. Dev stays on cx23 (2/4/40); it
only needs to validate the chain.

**No snapshots** — disaster recovery comes from:
- Terraform-reproducible cluster (test by destroying + recreating dev once during bringup)
- DB backups in Hetzner Object Storage
- Git for code + manifests + encrypted secrets

### 1.2 Networking

**Firewall (Hetzner Cloud Firewall, per VPS):**

| Port | Source | Purpose |
|---|---|---|
| 22 (SSH) | Tailnet only | Cluster admin |
| 6443 (k3s API) | Tailnet only | `kubectl` access |
| 80 (HTTP) | 0.0.0.0/0 | cert-manager HTTP-01 + 301-redirect to HTTPS |
| 443 (HTTPS) | 0.0.0.0/0 | All real traffic (web + API + webhook) |
| ICMP | 0.0.0.0/0 | Standard debugging |

Everything else dropped.

**Tailscale on every node:**

- `tailscaled` installed via cloud-init at first boot
- Joins your tailnet automatically using a pre-auth key
- SSH and kubectl reachable only over the tailnet — random scanners can't
  see the ports at all
- Solves the "my home IP rotates" problem permanently
- Free for personal use

### 1.3 DNS

**Registrar**: Porkbun. Managed declaratively via the Porkbun Terraform
provider in the same stack as the VPS.

**Records:**

```
scholia.study      A   <prod VPS IPv4>
www.scholia.study  CNAME scholia.study
dev.scholia.study  A   <dev VPS IPv4>
```

The Terraform graph orders: VPS up → IPv4 known → DNS records created → wait for propagation → cert-manager issues certs.

### 1.4 Kubernetes

- **Distribution**: k3s (lightweight, single binary, bundled Traefik + local-path storage)
- **Ingress**: Traefik (ships with k3s, kept default)
- **TLS**: cert-manager + Let's Encrypt (production ACME issuer)
- **Storage class**: local-path (k3s default) — backed by `/var/lib/rancher/k3s/storage` on the host

**Routing pattern (single hostname, three-tier behind nginx-cache):**

Ingress sends everything to `nginx-cache:80`; the proxy fans out to
upstreams internally per its location blocks (see
`apps/proxy/conf.d/` and `apps/proxy/templates/`):

```
                Traefik (TLS termination)
                          │
                          ▼
                  nginx-cache (proxy)
                  ├── /api/*        → scholia-api (Rust)
                  └── /*            → scholia-web  (Node SSR)
                          ▲
                          │  PURGE on :8080 (cluster-only)
                  scholia-api (Rust)
```

Same shape on `dev.scholia.study`.

**Why single hostname:** same-origin = no CORS plumbing, first-party
cookies for sessions, simple Stripe webhook URL, one cache layer for
both HTML and `/api/*`, fewer ingress rules to get wrong.

---

## 2. Application services in cluster

### 2.1 Rust API (`scholia-api`)

- **Image**: multi-stage Docker build (`rust:bookworm` builder, `debian:bookworm-slim` runtime)
- **Tag**: `ghcr.io/<you>/scholia-api:main-<sha7>` for dev, `:v1.2.3` for prod
- **Pod**: 1 replica per cluster (no horizontal scaling for v1)
- **Init container**: same image, runs `api migrate` to apply DB migrations
  before the main container starts. Migration failure prevents the API
  from starting (clean failure semantics).
- **Public**: never directly. Sits behind `scholia-proxy` for everything.
- **Required env**: in addition to existing keys (DATABASE_URL, session
  secret, Stripe, etc.), set `CACHE_PURGE_URL=http://scholia-proxy:8080`
  so `cache::invalidate` fires PURGE after writes. Unset = silent no-op
  (fine for local dev, a regression in cluster).

### 2.2 Web — Node SSR (`scholia-web`)

- **Image**: multi-stage build (`node:22-alpine` builder + runtime).
  Runs the TanStack Start Nitro output (`pnpm --filter @apps/web build`
  → `pnpm --filter @apps/web start`).
- **Tag**: same scheme as API
- **Pod**: 1 replica. Multi-replica would mean per-pod request caches
  drifting; the nginx-cache layer in front handles the public hit rate
  anyway.
- **Build output**: Nitro Node server (HTML on demand) + client assets.
  No prerender, no SPA shell.
- **Public**: never directly. Receives only HTML requests from the
  proxy (or `/api/*` requests that miss in the proxy cache go straight
  to Rust, not via Node).
- **Required env**:
  - `API_BASE_URL=http://scholia-api:4000` so SSR loaders reach Rust
    over the in-cluster Service.
  - `APP_PROFILE=prod` (or `dev`) — read by `__root.tsx` at render
    time and inlined into each HTML response as
    `window.__ENV__.APP_PROFILE`. This is what selects the right
    entry from `apps/web/src/config.ts`'s profile registry on the
    client.

### 2.3 Edge proxy / HTTP cache (`scholia-proxy`)

- **Image**: custom multi-stage build at `apps/proxy/Dockerfile`. Stage
  1 compiles `ngx_cache_purge` against the runtime image's nginx
  version; stage 2 is plain `nginx:1.27-alpine` with that .so dropped
  into `/usr/lib/nginx/modules/`.
- **Tag**: same scheme as API/Web.
- **Pod**: 1 replica. Multi-replica needs cache coherence
  (cross-pod PURGE fan-out) — deferred indefinitely.
- **Public**: yes, via Ingress. `:80` is the only port the Ingress
  routes to; `:8080` is `ClusterIP`-only for PURGE.
- **Required env**:
  - `UPSTREAM_WEB=scholia-web:3000`
  - `UPSTREAM_API=scholia-api:4000`
  - (`APP_PROFILE` is **not** needed here — runtime profile injection
    moved to the Node SSR layer; see § 0.2 and § 2.2.)
- **Storage**: PVC for cache. ~5GB on `local-path` storage class;
  sized to fit the cached working set (HTML + API ≈ 700MB at current
  content) with headroom. Backing cache with a PV (not `emptyDir`) so
  pod restarts don't cold-start the cache.
- **NetworkPolicy**: only `scholia-api` pods may reach
  `scholia-proxy:8080`. Defense-in-depth — the port is `ClusterIP`-only
  anyway, but the policy makes "no PURGE from outside Rust" an
  enforceable invariant rather than a deployment fact.

### 2.4 Postgres (`scholia-db`)

- **Pattern**: in-cluster, raw StatefulSet on local-path PVC (no operator)
- **Image**: official `postgres:16` (or 17 when stable)
- **Storage**: 20GB PVC initially, resizable
- **Backups**: see § 3
- **Why raw StatefulSet not CloudNativePG**: lower abstraction = better
  K8s learning. CNPG is a fine v2 target once you've felt the bare
  primitives.
- **Why in-cluster not host-installed**: K8s storage primitives (PVC,
  StatefulSet, Headless Service) are exactly the part of K8s most worth
  learning. Skipping them with on-host Postgres skips the hard part.

### 2.5 cert-manager + Traefik

- cert-manager installed via Helm (one-time, post-cluster-up)
- ClusterIssuer: Let's Encrypt prod ACME
- Certificate CRs declared in manifests; cert-manager auto-renews
- Traefik bundled with k3s; kept default

---

## 3. Backups

### 3.1 Target

**Hetzner Object Storage** (S3-compatible). Same provider as VPS — same-region traffic, single vendor, single bill, flat-rate 1TB pool incl. egress.

### 3.2 Cadence + retention

| Aspect | Decision |
|---|---|
| **Prod cadence** | Daily, 03:00 UTC, `pg_dump --format=custom`, gzipped |
| **Dev cadence** | Daily during bringup (validates the chain). Tear down once prod is stable; dev becomes restore-from-prod when needed. |
| **Retention** | Last 30 daily + 12 monthly via bucket lifecycle rules |
| **Encryption** | At-rest from Hetzner. No client-side encryption layer for v1 (no PCI/HIPAA scope). |
| **Restore test** | Manual, every 1-3 months. Pull latest dump, restore into dev cluster, smoke-test API. |

### 3.3 What also gets backed up

The DB is the obvious thing. **Don't forget:**

- **SOPS age keys** (per environment) — encrypt with passphrase, store in
  Hetzner Object Storage + duplicate in 1Password/Bitwarden. **Lose the
  key, lose the ability to decrypt your committed secrets.**
- **`.env`/Tailscale auth keys/Hetzner API token** — your laptop's
  credentials, in your password manager
- **Terraform state** — already in Hetzner Object Storage (different
  prefix from DB backups)

---

## 4. Tooling & workflow

### 4.1 IaC: Terraform

- **Provisioned**: Hetzner Cloud (VPS, firewall, Cloud Network), Porkbun (DNS records)
- **k3s install**: cloud-init script attached to the VPS; runs `curl -sfL https://get.k3s.io | sh -` on first boot, plus Tailscale install. Both `curl | sh` installs are wrapped in a **5× retry loop** (10s backoff) — a single transient GitHub/network blip on first boot otherwise leaves a half-provisioned box with no k3s. (Learned the hard way: the prod bringup hit `k3s "[ERROR] Download failed"` and needed a manual SSH re-run of the install before the retry wrapper was added.)
- **State**: stored in Hetzner Object Storage as the Terraform backend
- **Workspaces**: `dev` and `prod` for the two environments. Both provisioned (dev 2026-05, prod reserved 2026-06-08); per-workspace state at `env:/<ws>/...`.
- **Module structure**: one `modules/cluster/` instantiated twice with different vars (only `hostname` + `vps_type` differ: dev cx23, prod cx43)

### 4.2 Manifests: Kustomize → ArgoCD

> **Status (2026-06-08): ArgoCD is LIVE for dev.** The manual
> `kubectl apply -k` path below is **superseded** — dev now deploys via
> GitOps. Canonical docs: `PLAN_ARGOCD.md` (decisions) and
> `infra/argo/README.md` (bootstrap + day-to-day). Prod will follow the
> same pattern when its cluster exists.

**v0 (initial bringup — historical):**

- Plain Kustomize with `base/` + `overlays/{dev,prod}/`
- Manual `kubectl apply -k overlays/dev/` from your laptop
- CI pushes images + commits image-tag bump to dev overlay

**v1 (now — dev):**

- One self-contained ArgoCD **per cluster** (no cross-cluster creds)
- Argo `Application` CR per environment (`infra/argo/application-dev.yaml`),
  points at `infra/k8s/overlays/dev`
- Sync = `kustomize build` (KSOPS-enabled repo-server) + apply + drift
  detection, automated with prune + selfHeal
- Image flow = **CI git write-back**: the `bump` job rewrites the
  overlay's immutable `main-<sha>` tags and pushes to `main` (via PAT);
  Argo syncs the diff. (Chose this over Argo Image Updater — one fewer
  controller.)

The Kustomize bases + overlays from v0 are reused as-is by Argo.

### 4.3 Secrets: SOPS + age

- **Tool**: SOPS (mozilla/getsops) with age keys
- **Keys**: one age key per environment (dev, prod). Distinct, never shared.
- **Storage**: encrypted YAML files committed to git
- **Apply path (v0 — historical)**: `sops -d secret.yaml | kubectl apply -f -`
- **Apply path (v1, ArgoCD — live)**: **KSOPS** init-container on the
  repo-server decrypts during `kustomize build` (age key mounted from the
  `sops-age` Secret). The overlay references `secrets/` via a ksops
  generator; plaintext never leaves the repo-server. See
  `infra/argo/values.yaml` + `infra/k8s/overlays/dev/secret-generator.yaml`.
  (Chose KSOPS over argocd-vault-plugin — AVP targets external vaults,
  not SOPS-in-git.)
- **Why SOPS over Sealed Secrets**: Bitnami acquisition by Broadcom has
  eroded trust; SOPS is vendor-neutral, broadly used, and works for
  non-K8s secrets too (e.g. tfvars).

### 4.4 Migrations: sqlx-cli + sqlx::migrate!

- New migration: `sqlx migrate add <name>` creates `db/migrations/<ts>_<name>.sql`
- Embedded into the API binary via `sqlx::migrate!("../../db/migrations")`
- Run via init container at deploy time
- `_sqlx_migrations` table tracks applied + checksums
- **Append-only**: never edit applied migrations

### 4.5 CI/CD: GitHub Actions

| Trigger | Action |
|---|---|
| PR opened/updated | Build + test + lint + typecheck (no push) |
| Push to `main` | Build changed images (api, web, proxy, + ingest jobs), push `:main-<sha>` to ghcr.io, then the `bump` job **commits per-service image-tag bumps to `infra/k8s/overlays/dev/`** (via PAT) for Argo to sync |
| Tag `v*` push | Build all three images, push `:<version>` and `:latest` to ghcr.io |

The proxy image rebuild is slow on first build (it compiles
`ngx_cache_purge` from source) but well-cached after, since the source
versions and configure args rarely change.

- **Runner**: GitHub-hosted (free for public repos)
- **Auth**: `GITHUB_TOKEN` (built-in) for ghcr push; a `PAT` secret for
  the `bump` job's commit-back (bypasses branch protection on `main`)
- **No Hetzner / kubectl credentials in CI** — Terraform runs from
  laptop; deploys are pull-based (ArgoCD syncs from git), so CI never
  touches the cluster

### 4.6 Image registry: ghcr.io

- **Visibility**: public (image bytes aren't sensitive; secrets live in K8s Secrets)
- **Three images**: `ghcr.io/<you>/scholia-api`, `ghcr.io/<you>/scholia-web`,
  `ghcr.io/<you>/scholia-proxy`
- **Tag scheme**: `main-<sha7>` for main pushes, semver for releases, never `:latest` in Deployments

---

## 5. Repository layout

```
.
├── apps/
│   ├── api/                  # existing — Rust axum API
│   ├── web/                  # existing — React frontend, now Node SSR
│   └── proxy/                # existing — nginx-cache (Dockerfile + conf)
├── db/
│   └── migrations/           # NEW — sqlx migrations, append-only
├── infra/                    # NEW
│   ├── terraform/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── dev.tfvars
│   │   ├── prod.tfvars
│   │   ├── cloud-init/
│   │   │   └── k3s.yaml.tpl
│   │   └── modules/
│   │       └── cluster/
│   └── k8s/
│       ├── base/             # core manifests (api, web, proxy, postgres, ingress)
│       └── overlays/
│           ├── dev/
│           └── prod/
├── PLAN_DEVOPS.md            # this file
└── ...
```

---

## 6. Operational practices

### 6.1 Migration discipline

- Every schema change = new migration file
- Never edit applied migrations (`_sqlx_migrations` checksum will reject)
- Backfills are multi-step: nullable add → backfill → constrain
- Test migrations on dev before prod

### 6.2 Backup discipline

- Verify daily backup ran (CronJob exit code, object exists in bucket)
- Quarterly manual restore test into dev cluster
- Never let restore-test cadence slip — untested backup = no backup

### 6.3 Deploy discipline (dev — ArgoCD)

- Push to main → CI builds + `bump` job commits the dev overlay tag →
  **ArgoCD auto-syncs** (no manual apply). Watch
  `kubectl get applications -n argocd -w`.
- Verify on dev (smoke-test paid checkout flow, articles, profile)
- Manual hotfixes via `kubectl` get reverted by selfHeal — put fixes in
  git.
- Watch logs via `kubectl logs` for first ~10 minutes post-deploy
- **Prod (when it exists)**: same pattern — second Argo on the prod
  cluster, `application-prod.yaml` tracking `overlays/prod`. Tag a
  release (`vX.Y.Z`) → CI builds prod-tagged image → bump prod overlay.

### 6.4 Disaster recovery

The "no snapshots" path requires `terraform apply` to fully reconstruct a
cluster from zero. **During bringup, deliberately destroy + recreate dev
once** to verify this works. If it doesn't, you don't have DR.

Recovery scenarios:

| Failure | Recovery |
|---|---|
| App pod crash | k3s auto-restarts (Deployment) |
| Postgres pod crash | k3s restarts the StatefulSet pod, PVC reattaches |
| Local-path volume corrupt | Restore from latest pg_dump in HOS, ~30 min |
| VPS lost entirely | `terraform destroy && terraform apply` (or just `apply` for replacement) → SOPS-decrypt secrets → restore DB from backup → ~45 min |
| ghcr.io image lost | Push from CI again from a tagged commit |
| Forgot SOPS age key | Restore from offsite copy (HOS + password manager) |

### 6.5 Stripe operational notes

- **Webhook URL must change post-API-refactor**: `…/api/webhooks/stripe`
  (currently `/webhooks/stripe`). Update Stripe Dashboard for both test
  and live mode.
- **Test mode and live mode have separate Customer Portal configs.**
  Don't forget to enable plan switching + cancel-at-period-end in live
  mode at launch.
- **Webhook secret is per-endpoint per-mode.** Dev cluster uses test-mode
  signing secret; prod cluster uses live-mode secret.

---

## 7. Observability roadmap

| Phase | What | Why |
|---|---|---|
| **v0** (now) | `kubectl logs`, `kubectl top` | Bringup; you'll be at the terminal anyway |
| **v1** (pre-launch) | Sentry (errors only) | Best ROI per minute of integration; free tier covers our scale |
| **v2** (after launch + traffic) | Grafana Cloud free tier | Logs + metrics aggregation; 50GB logs, 10k metrics, 14-day retention |

**Don't self-host Grafana/Loki/Prometheus on the dev cx23.** The stack
alone eats ~1GB RAM, leaving no headroom on a 4GB box. (Prod's cx43 has
16GB, so a light self-hosted stack there is viable later — but Grafana
Cloud's free tier is still the lower-effort default.)

---

## 8. Roadmap

### v0 — Pre-deployment refactor

- [x] Move backend routes under `/api/*`
- [x] Implement runtime config injection (profile registry +
      `window.__ENV__.APP_PROFILE`; proxy serves `/config.js`)
- [x] Runtime SSR rewrite (drop prerender + SPA shell, TanStack Start
      Nitro output)
- [x] Proxy / nginx-cache scaffold in `apps/proxy/` (Dockerfile,
      envsubst templates, cache zones, cookie bypass, hosted-text
      route exception, PURGE on `:8080`)
- [x] Rust `cache::invalidate` helper + handler wiring (articles
      update/publish/archive, profile)
- [ ] Wire PURGE into ingest binaries (`bible_to_db`, `kant1_*`) so a
      re-ingest invalidates the affected book/chapter URLs
- [x] Migrations bootstrap (sqlx-cli, init container)
- [x] Update auto-memory note about `db_reset.sh` flow
- [ ] Regenerate openapi.json + frontend client (run as part of any
      handler change)
- [ ] Smoke-test the production build path end-to-end:
      `pnpm --filter @apps/web build && pnpm --filter @apps/web start`
      behind the proxy, then `curl` chapter URLs to confirm Nitro
      output paths match the `start` script
- [ ] Update Stripe Dashboard webhook URL post-deploy

### v0 — Cluster bringup

- [x] Hetzner API token, Porkbun API key, Tailscale auth key in laptop env
- [x] Write `infra/terraform/clusters/` (Hetzner + Porkbun providers, cloud-init for k3s + Tailscale)
- [x] State backend in Hetzner Object Storage (`scholia-tf-state` bucket, fsn1)
- [x] `terraform apply -var-file=dev.tfvars` — dev cluster live at `dev.scholia.study`
- [x] `terraform apply -var-file=prod.tfvars` — **prod box reserved 2026-06-08** (cx43, apex `scholia.study`). Box up + Ready; idle (no app yet). Gotcha: had to delete Porkbun's apex ALIAS + wildcard-parking records first, and the k3s install hit a transient download failure needing a manual SSH re-run (now mitigated by the cloud-init retry wrapper).
- [x] **Validate IaC**: `terraform destroy` + `terraform apply` on dev once
- [x] Install cert-manager (Helm chart v1.20.2 in `cert-manager` namespace)
- [x] Configure Let's Encrypt staging + prod `ClusterIssuer`s
      (`infra/k8s/base/cert-manager/cluster-issuer-{staging,prod}.yaml`,
      ACME contact `contact@filipniklas.com`)
- [x] SOPS dev age keypair generated + encrypted backups to Hetzner
      Object Storage (`scholia-key-backups` bucket) + Dropbox
- [ ] SOPS prod age keypair (deferred with prod cluster)
- [x] Write `infra/k8s/base/` covering all three Deployments + Services:
  - `scholia-api` Deployment + ClusterIP Service (port 4000), init
    container running `api migrate`, env including `CACHE_PURGE_URL`,
    DATABASE_URL composed from the `postgres` Secret at runtime via
    `$(VAR)` env substitution
  - `scholia-web` Deployment + ClusterIP Service (port 3000), env
    including `API_BASE_URL=http://scholia-api:4000`, `APP_PROFILE`
  - `scholia-proxy` Deployment + ClusterIP Service (ports 80 + 8080),
    env including `UPSTREAM_WEB`, `UPSTREAM_API`. **No `APP_PROFILE`
    here** — that env moved to the web Deployment after the inline-
    script profile-injection migration.
  - PVC for proxy cache (5 GB, `local-path`)
  - Postgres StatefulSet + headless Service + 20 GB PVC (`postgres:18`,
    single replica, local-path)
  - NetworkPolicy restricting `scholia-proxy:8080` to pods labelled
    `app.kubernetes.io/name: scholia-api`
  - Ingress (Traefik) terminating TLS, routing to `scholia-proxy:80`;
    hostname is a `PLACEHOLDER` patched by the env overlay
- [x] Write `infra/k8s/overlays/dev/`:
  - `kustomization.yaml` referencing the base
  - `ingress-patch.yaml` replacing the hostname with `dev.scholia.study`
  - `secrets/postgres.yaml` and `secrets/api.yaml` — SOPS-encrypted via
    `.sops.yaml` at the repo root. Filled with real values: Stripe test
    keys, Resend API key, GitHub OAuth (dev app registered with callback
    `https://dev.scholia.study/api/auth/github/callback`).
- [ ] Write `infra/k8s/overlays/prod/` (mirror of dev with hostname
      `scholia.study`, `letsencrypt-prod` issuer, prod SOPS recipient)
- [x] **Dockerfile for `scholia-api`** — `apps/api/Dockerfile`.
      Multi-stage cargo-chef build (Rust 1.91-bookworm builder →
      `debian:bookworm-slim` runtime, libssl3 + ca-certificates, non-
      root uid 10001). Reads committed `.sqlx/` offline metadata via
      `SQLX_OFFLINE=true`, so the build needs no live DB. Image is
      ~148 MB. `scripts/db_prepare.sh` + `pnpm db:prepare` regenerate
      `.sqlx/` against the local DB after any sqlx query change.
- [x] **Dockerfile for `scholia-web`** — `apps/web/Dockerfile`.
      Multi-stage node:22-alpine + corepack/pnpm. Separate `deps`
      (full install) and `prod-deps` (`--prod`) stages so the runtime
      only carries production deps. `--ignore-scripts` everywhere to
      sidestep the root `prepare: lefthook install` lifecycle hook.
      Runtime invokes srvx directly (no pnpm in the runtime image).
      Image is ~455 MB — mostly the React+MUI+@mdxeditor stack itself.
      Five deps moved to devDependencies as part of this work:
      `@tailwindcss/vite`, `@tanstack/react-devtools`,
      `@tanstack/react-router-devtools`, `@tanstack/router-plugin`,
      `nitro` (the last verified unreferenced at runtime).
- [x] **Dockerfile for `scholia-proxy`** — already at `apps/proxy/Dockerfile`
- [x] **`.sqlx/` offline metadata committed** (144 query files).
- [x] **`.dockerignore` at repo root** — keeps build context lean
      across api + web (excludes target, node_modules, infra, docs,
      assets, terraform state, env files).
- [x] **GitHub Actions workflow** — `.github/workflows/build.yml`
      builds api + web + proxy in a matrix, pushes to
      `ghcr.io/<owner>/scholia-{api,web,proxy}` on push-to-main and
      manual dispatch. Tags: mutable `main` + immutable
      `main-<short-sha>` (no semver — Scholia is a deployed app, not
      a library). Per-service build context (proxy uses
      `apps/proxy/`, api+web use repo root). Cache: `type=gha`,
      scoped per service. `fail-fast: true` so partial pushes don't
      leave the three images out of sync. Top-level `paths:` filter
      skips builds when nothing image-relevant changed (docs, infra,
      *.md). `concurrency.cancel-in-progress: true` so a second push
      to the same ref cancels the in-flight build.
- [x] Make the three GHCR packages public (or set retention on the
      private ones). Defaults to inherit-from-repo on first push.
- [x] Deploy to dev (originally manual; **now via ArgoCD** — see
      `infra/argo/README.md`. Historical manual steps:)
  - `source ~/.config/scholia-infra.env` (for `SOPS_AGE_KEY_FILE`)
  - `sops -d infra/k8s/overlays/dev/secrets/postgres.yaml | kubectl apply -f -`
  - `sops -d infra/k8s/overlays/dev/secrets/api.yaml | kubectl apply -f -`
  - `kubectl apply -f infra/k8s/base/cert-manager/` (ClusterIssuers — still manual)
  - ~~`kubectl apply -k infra/k8s/overlays/dev/`~~ → ArgoCD syncs this
  - `kubectl get pods -n scholia -w` until all are Ready
- [ ] Validate end-to-end on dev: anonymous chapter pageviews cache
      (X-Cache-Status: MISS then HIT), authenticated requests bypass,
      PURGE after an article publish invalidates the listing, Stripe
      test charge → role flips → cancellation flow
- [ ] Flip dev Ingress's `cert-manager.io/cluster-issuer` annotation
      from `letsencrypt-staging` to `letsencrypt-prod` once HTTP-01
      challenge is known-good
- [ ] `kubectl apply -k overlays/prod/`
- [ ] Update Stripe to live mode keys + production webhook URL
- [ ] Public soft launch

### First-deploy landed (2026-05-18)

Dev cluster is serving real traffic. All four pods Ready, Ingress
routing TLS to the proxy, cache layer producing `X-Cache-Status: MISS`
→ `HIT`, `/api/library` returns `{"groups":[],"stats":{"works":0,…}}`
(DB migrations applied, just no content ingested yet).

Two snags hit + fixed along the way:

- Base Deployments referenced the wrong org (`ghcr.io/filipniklas/…`
  vs the actual GHCR namespace `ghcr.io/scholia-study/…`). The
  workflow uses `${{ github.repository_owner }}` which resolves to
  the org. Now corrected.
- Init container's migration failed with `Configuration(InvalidPort)`.
  Root cause: postgres password contained URL-special characters, and
  k8s `$(VAR)` substitution is literal — the rendered `DATABASE_URL`
  had unencoded chars that confused sqlx's URL parser. Fixed
  structurally rather than by changing the password:
  `apps/api/src/config.rs::pg_connect_options_from_env()` now builds
  a `PgConnectOptions` from discrete `POSTGRES_{USER,PASSWORD,DB,
  HOST,PORT}` env vars (falls back to `DATABASE_URL` for local dev).
  Init container env updated to match.

### HTTPS landed (2026-05-18)

`https://dev.scholia.study/` now serves a browser-trusted Let's
Encrypt cert (issuer `R12`, 90 days), and plain HTTP returns 308 →
HTTPS via a Traefik `Middleware` CRD
(`infra/k8s/base/ingress/middleware-redirect-https.yaml`) referenced
from the Ingress through the
`traefik.ingress.kubernetes.io/router.middlewares` annotation.
Renewals are automatic — cert-manager checks every ~12h and re-issues
when a cert crosses 2/3 lifetime. The HTTP-01 solver creates a
sibling Ingress without the redirect annotation, so renewals keep
working.

### Network hardening landed (2026-05-18)

`postgres:5432` is now only reachable from pods labelled
`app.kubernetes.io/name: scholia-api`
(`infra/k8s/base/postgres/network-policy.yaml`). Verified by `nc`
from the web pod failing while `curl /api/library` keeps working.
Defense-in-depth — already ClusterIP-only externally, this closes
the "compromised web/proxy pod brute-forces Postgres" path.

Two more hardening steps deferred:

- **k3s `--secrets-encryption`** to encrypt the Secrets datastore on
  the node disk. Worth adding at prod cluster bringup so it's there
  from day one rather than requiring a Secret rewrap mid-life.
- **Separate app-level DB user** with only CONNECT/USAGE/CRUD on the
  app tables (no DDL, no DROP). Currently the api uses the Postgres
  superuser. Limits blast radius if api credentials leak.

### Dev lockdown landed (2026-05-18)

Public visitors can no longer accidentally find dev.scholia.study.
Three layers:

1. **HTTP Basic Auth** at the proxy (`scholia` / `loves2study`).
   Bcrypt-hashed htpasswd in
   `infra/k8s/overlays/dev/secrets/proxy-htpasswd.yaml`
   (SOPS-encrypted), mounted via overlay patch at
   `/etc/nginx/auth/htpasswd`.
2. **robots.txt** served inline by the proxy (no auth needed so
   crawlers can read the Disallow) + `X-Robots-Tag: noindex, nofollow,
   noarchive` on every other response.
3. **Stripe webhook carve-out**: `location ^~ /api/webhooks/` has
   `auth_basic off;` so Stripe's servers reach the handler. Stripe-
   Signature header still authenticates.

Structurally clean: the base proxy stays prod-ready (public, no auth).
Dev opts in via:

- `apps/proxy/templates/default.conf.template` —
  `include /etc/nginx/conf.d/security/*.conf;` at server scope (no-op
  in base; dir is created empty by the Dockerfile).
- `infra/k8s/overlays/dev/proxy-security.conf` — the lockdown
  fragment (auth + robots + webhook carve-out).
- `infra/k8s/overlays/dev/proxy-lockdown-patch.yaml` — strategic
  merge that mounts both the ConfigMap (generated from the fragment)
  and the htpasswd Secret onto the proxy pod.
- `configMapGenerator` in `overlays/dev/kustomization.yaml` —
  content-hashed ConfigMap name so editing the fragment auto-restarts
  the pod.

When prod overlay is built, no proxy changes needed — base is already
public-by-default.

### Dev DB content + ergonomics (2026-05-18)

- Kant ingested into dev DB via `pnpm db:dev:run pnpm db:kant1`
  through the kubectl port-forward tunnel.
- New scripts (root `package.json`):
  - `db:dev:forward` — opens `localhost:55432` → `postgres:5432`.
  - `db:dev:run <cmd>` — runs any command with `DATABASE_URL`
    pointed at the tunneled cluster DB. Wraps
    `scripts/db_dev_run.sh` which decrypts the password from SOPS,
    URL-encodes it, and `exec`s the command.
  - `db:dev:reset` — drops public schema + re-applies migrations on
    the dev cluster. Prompts for explicit `yes` confirmation; bypass
    with `--yes` for automation.
  - `db:dev:reload` — `db:dev:reset` + re-ingest of Kant + Bible.

### Cluster capacity snapshot (2026-05-18)

CX22 (2 vCPU, 4GB). Current allocations:

| Pod | CPU req | Mem req | Mem limit |
|---|---|---|---|
| proxy    | 50m  | 64Mi  | 256Mi |
| api      | 100m | 128Mi | 512Mi |
| web      | 100m | 128Mi | 512Mi |
| postgres | 100m | 256Mi | 1Gi   |

Idle usage: ~13m CPU, ~156Mi memory across all four pods. Node
allocations: 27% CPU requests, 18% mem requests, 64% mem limits.
Headroom for 5-10× current load before resize to CX32 is needed.

No CPU limits set (intentional — requests give priority, limits
just throttle bursty workloads). Memory limits set for OOM
protection.

### Redeploy verb

**Dev is GitOps now — the redeploy verb is `git push`.** ArgoCD
auto-syncs any change merged to `main` under `overlays/dev` (image-tag
bumps from CI, manifest edits, secret rotations). Immutable `main-<sha>`
tags mean a code change = CI bump = new tag = Argo rolls the Deployment.

Manual `kubectl` escape hatches (rarely needed; selfHeal reverts
out-of-band edits):
- **Force a re-sync**: `argocd app sync scholia-dev` (or the UI).
- **Inspect drift**: `argocd app diff scholia-dev`.
- The old `kubectl rollout restart` / `kubectl apply -k` flow is retired
  for dev; immutable tags make `rollout restart` a no-op anyway.

### Ingest-as-Jobs (next workstream, designed 2026-05-19)

**Problem.** Bible KJV alone didn't finish in 20 min over the
kubectl port-forward tunnel; full Bible would be hours. Tunnel
latency × millions of small INSERTs is the bottleneck. `pg_dump |
psql` would be faster but still tunnel-bound, and would require
wiping + restoring content tables (CASCADE-nuking any user
quotations/articles that reference them). Both are stopgaps. Post-
launch, we'll regularly add new books and translations against a
live prod DB — we need a pattern where each ingest is fast,
additive, and doesn't disturb user content.

**Solution.** Package each ingest binary into its own container
image (no assets baked in). Assets live in Hetzner Object Storage
and are pulled at Job start. Run each book/translation ingest as
a one-shot Kubernetes Job in-cluster. Postgres traffic stays on
the LAN. Same pattern for dev and prod.

```
local workstation                cluster
─────────────────                ─────────────────────────────────
                                ┌────────────────────────────────┐
edit assets/ locally            │ rclone sync s3:scholia-assets  │
pnpm assets:sync ──┐            │   /<scope> → /assets           │
                   ↓            │ binary (bible_to_db | ...)     │
       s3://scholia-assets ────►│   reads /assets                │
                                │   ↓ in-cluster                 │
                                │   postgres svc                 │
                                │   ↓                            │
                                │   curl PURGE                   │
kubectl apply ─────────────────►│ Job pod orchestrates           │
  ingest-bible.yaml             └────────────────────────────────┘
```

**Components.**

1. **One image per importer.** Built from `jobs/<name>/Dockerfile`:
   - `jobs/ingest-bible/` → `scholia-ingest-bible` image, binary
     `bible_to_db`, ENTRYPOINT `ingest_bible.sh` (runs KJV first,
     then WEB/ASV/BBE/DARBY in parallel — preserves existing
     `db_bible.sh` orchestration).
   - `jobs/ingest-kant1/` → `scholia-ingest-kant1` image, binary
     `kant1_struct_to_db`, ENTRYPOINT `ingest_kant1.sh` (runs DE
     then EN translation).
   - Future content sources (Aristotle, Augustine, …) drop in
     under `jobs/` without touching `apps/`.

   Rust source stays under `packages/` (workspace members). Each
   `jobs/<x>/Dockerfile` references its own crate as build input.
   `cargo-chef` caches workspace deps across the per-image builds
   so the marginal cost of N images is just N final-link steps,
   not N full workspace compiles.

   Convention: `apps/` houses long-running Deployments;
   `jobs/` houses one-shot Jobs. Mirrors the K8s primitives:
   `apps/<x>/Dockerfile` ↔ `infra/k8s/base/<x>/` for services;
   `jobs/<x>/Dockerfile` ↔ `infra/k8s/jobs/<x>.yaml` for batch.

2. **Bucket-based assets (`scholia-assets`, Hetzner Object
   Storage, fsn1).** Provisioned by
   `infra/terraform/shared/main.tf`. Local `assets/` is canonical;
   `pnpm assets:sync` mirrors it up (rclone sync, ~1.7GB total,
   ~5min cold push, ~1-2min re-run after edits). Each Job pulls
   only the scope it needs at pod start:
   - `ingest-bible` pulls `s3://scholia-assets/bible/`.
   - `ingest-kant1` pulls `s3://scholia-assets/kant1/derived/md_to_struct/`
     + `s3://scholia-assets/kant1/derived/md_translation_to_struct/`.

   Bucket creds: single Hetzner-issued S3 keypair, account-wide
   (Hetzner Object Storage doesn't support per-bucket IAM). Local
   reads/writes via `AWS_ACCESS_KEY_ID` + `AWS_SECRET_ACCESS_KEY`
   in `scholia-infra.env`. Cluster Jobs read via the same keypair
   exposed through a SOPS-encrypted Secret. Per-bucket split would
   require a separate Hetzner project — defer.

   Image size win: per-binary image is ~30MB (binary + rclone),
   not ~80-300MB (binary + assets baked in). Faster CI builds and
   faster Job cold-starts on first pull.

3. **Idempotency in ingest binaries.** Each binary previously
   `INSERT`ed assuming an empty schema; a re-run blew up on the
   `books.slug` UNIQUE constraint mid-transaction.
   - ✅ **Skip-if-exists landed (2026-05-20).** `bible_to_db` and
     `kant1_struct_to_db` now check `SELECT id FROM books WHERE
     slug = $1` before opening the transaction; on hit they log
     "already imported, skipping" and exit 0. Each translation
     has its own `books` row (`kjv-bible`, `web-bible`, …) so the
     guard is per-translation, not per-Bible-as-a-whole.
   - ⏳ Replace flow deferred. A `--force-replace` flag that
     deletes + re-inserts a given book is a separate design — has
     to account for FK chains to user content (quotations →
     sentences) and for `cross_translation_alignments` regen.
     Schema/source-data fixes go through `pnpm db:reset` for now.

4. **Job manifests (`infra/k8s/jobs/`).** One per content source
   (Bible is one Job, Kant is one Job — intra-source orchestration
   stays inside the runner script). Each Job:
   - `generateName: ingest-bible-` so re-applies create fresh
     pods; old ones linger as audit trail until TTL'd.
   - Reads `POSTGRES_*` env from the same `postgres-credentials`
     Secret the api uses (DB user, password, host, port, db).
   - Reads `AWS_ACCESS_KEY_ID` + `AWS_SECRET_ACCESS_KEY` from a
     SOPS-encrypted `assets-bucket` Secret for the rclone pull.
   - `restartPolicy: Never`, `backoffLimit: 0` — Jobs fail fast
     on bugs. Investigate, fix, re-apply rather than auto-retry.
   - `ttlSecondsAfterFinished: 86400` — auto-clean after 1 day.
   - `resources.requests` modest; `limits.memory` generous (ingest
     parses + holds book trees in memory).

5. **PURGE on success.** ✅ Landed 2026-05-20. Both binaries call
   `purge_cache(...)` after `tx.commit()`, reading
   `CACHE_PURGE_URL` from env (no-op if unset, so local dev
   without a proxy still works). Currently invalidates `/api/
   library`, `/api/books`, `/books` — enough for the new-book
   case (skip-if-exists). When `--force-replace` lands, the path
   list will need to expand to the per-book pages too.

6. **Workflow.**
   ```
   # 1. edit assets/ locally if content changed; sync to bucket
   pnpm assets:sync
   # 2. CI builds scholia-ingest-{bible,kant1}:main (if Dockerfile
   #    or crate sources changed)
   # 3. trigger:
   kubectl apply -f infra/k8s/jobs/ingest-bible.yaml -n scholia
   # 4. watch:
   kubectl wait --for=condition=complete \
     job -l ingest=bible --timeout=20m -n scholia
   kubectl logs -f -l ingest=bible -n scholia
   # 5. job terminates; PURGE has fired; content live
   ```

**First proof point.** Bible-on-dev becomes the first concrete use
of this pattern, replacing the killed tunneled ingest.

**Once landed.** The `scripts/db_dev_run.sh`, `db_dev_reset.sh`,
`db_dev_reload`, etc. stay useful for Beekeeper DB inspection and
emergency wipes, but the primary content path is Jobs. The old
`pnpm db:dev:run pnpm db:bible` workflow is retired.

### Ingest-as-Jobs prereqs landed (2026-05-19)

- ✅ **Terraform restructured** to `infra/terraform/{clusters,
  shared}/`. Cluster TF (per-env, workspaced) lives in
  `clusters/`. Project-wide TF (no workspaces, applied once)
  lives in `shared/`. Backend keys: `scholia/terraform.tfstate`
  for clusters, `scholia/shared.tfstate` for shared — same
  backing bucket (`scholia-tf-state`), different state files.
- ✅ **`scholia-assets` bucket provisioned** via
  `infra/terraform/shared/main.tf` (AWS provider pointed at
  Hetzner Object Storage S3 endpoint, `fsn1`).
- ✅ **`pnpm assets:sync`** wired up. `scripts/assets_sync.sh`
  configures rclone via env vars (no `rclone.conf` needed),
  mirrors `./assets/` → `s3://scholia-assets/`. Source-of-truth
  is local; sync deletes remote files no longer present locally.
  Pass-through flags (e.g. `--dry-run`) supported.
- ✅ **Initial 1.7GB upload complete.** Re-runs take ~1-2 min to
  list and compare 8300+ files; transfers only changes.

### Pickup for next session

1. **Land Ingest-as-Jobs** (see above). Order of operations:
   1. ✅ Provision `scholia-assets` bucket (Terraform).
   2. ✅ Seed via `pnpm assets:sync`.
   3. ✅ Skip-if-exists in `bible_to_db` and `kant1_struct_to_db`
      (2026-05-20). Each binary now checks `books.slug` before
      opening its transaction and exits cleanly if the book is
      already imported. `--force-replace` deferred (see § 3).
   4. ✅ PURGE wired into both binaries (2026-05-20). Reads
      `CACHE_PURGE_URL` from env; calls after `tx.commit()`;
      invalidates `/api/library`, `/api/books`, `/books`.
   5. ⏳ Build `jobs/ingest-bible/Dockerfile` and
      `jobs/ingest-kant1/Dockerfile`. Each image: binary + rclone
      + runner script (`ingest_bible.sh`, `ingest_kant1.sh`).
      Refactor `scripts/db_bible.sh` + `db_kant1.sh` so the binary
      path is `${INGEST_BIN:-target/release/<name>}` — same script
      works in dev (cargo-build path) and in-image (pre-built bin
      on PATH). Add to CI workflow alongside api/web/proxy.
   6. ⏳ Create SOPS-encrypted `assets-bucket` Secret in
      `infra/k8s/overlays/{dev,prod}/secrets/` for the rclone pull
      creds.
   7. ⏳ Write Job manifest templates under `infra/k8s/jobs/`
      (`ingest-bible.yaml`, `ingest-kant1.yaml`). Kustomized so
      dev/prod overlays don't fork the YAML.
   8. ⏳ Run the first Job against dev: Bible. Verify timing,
      logs, PURGE call, idempotency on re-run.
   9. ⏳ Run Kant ingest Job against dev as second proof.

2. **End-to-end validation tail** (per § 6.3 — partially done):
   - ✅ Anonymous chapter pageview: `X-Cache-Status: MISS` → `HIT`
     (also `EXPIRED`, normal).
   - ✅ `/api/*` cacheable when anonymous.
   - ✅ Authenticated requests bypass: `X-Cache-Status: BYPASS` on
     every request once `scholia_session` cookie is present
     (verified via DevTools while logged in via GitHub OAuth).
   - ✅ Authenticated write path: profile bio update via GitHub-
     authed session round-trips to the DB and renders fresh.
   - ⏳ PURGE after article publish invalidates the listing. Any
     authenticated user can do this — `ArticlesCreate` is a base
     permission and `create_article` only needs a title. Flow: create
     article → publish → GET `/articles` (MISS then HIT) → publish a
     second article → GET `/articles` again and expect MISS (PURGE
     fired). No editor role or ingested book required.
   - ⏳ Stripe test charge → role flips → cancellation. Requires
     pointing the Stripe Dashboard webhook at
     `https://dev.scholia.study/api/webhooks/stripe` and using the
     dev test-mode keys already in the api Secret. Carve-out
     already in place (`/api/webhooks/` bypasses Basic Auth).

3. **Deferred hardening** (from earlier security review):
   - NetworkPolicy on Postgres ✅ done.
   - k3s `--secrets-encryption` — defer to prod cluster bringup.
   - Separate app-level DB user (not superuser) — v1 maintenance.

4. **Eventually**: prod cluster, prod overlay, Stripe live keys,
   soft launch.

### v1 — ArgoCD + Sentry

- [x] Install ArgoCD (dev cluster; per-cluster model) — 2026-06-08
- [x] Migrate Kustomize apply → Argo Application sync (dev)
- [x] SOPS handling in Argo (KSOPS init-container on repo-server)
- [ ] Install ArgoCD on prod cluster when it exists
- [ ] Sentry integration in API + frontend
- [ ] Capacity-aware UX (warn at 80% of free-tier limits)

### v1 — SEO infrastructure

Orthogonal to bringup; lands once the cache + PURGE pipeline is live
in at least dev. Reuses the same invalidation plumbing so re-ingests
and content writes don't strand stale crawler payloads.

- [ ] Sitemap: Rust handler at `/sitemap.xml` (and shard files
      `/sitemap-bible-1.xml`, etc.) that enumerates canonical URLs
      from the DB. Cacheable like everything else; PURGEd via the
      ingest pipeline once that wiring lands.
- [ ] JSON-LD breadcrumbs: rendered in `__root.tsx` (or the route
      component) so they land in the SSR HTML. Reads a `breadcrumb`
      field on `NodeDetail` — add server-side alongside the existing
      `book_prefixed_label`.
- [ ] OG images: lazy per-URL generator. Rust endpoint producing a
      1200×630 PNG from book title + chapter label. Cache aggressively.
      Punt until crawler data shows it matters.

### v2 — Observability + scale

- [ ] Grafana Cloud Agent in clusters
- [ ] Dashboards for HTTP latency, error rate, DB connections, cache
      hit rate (surface `$upstream_cache_status` from access logs)
- [ ] Prod VPS upgrade to CX32 if RAM headroom drops below 1GB
- [ ] Postgres operator (CloudNativePG) migration when PITR / replicas matter

---

## 9. Open questions / deferred decisions

These were touched on during the grill but explicitly deferred:

- **Pending-update mirroring**: surfacing scheduled tier changes (e.g.
  "Your tier changes to Patron on May 31") in our own UI. Stripe's portal
  shows it; deferred until users ask.
- **Free-tier capacity limits in UI**: warning banners + dashboard usage
  bars when approaching limits. v1+.
- **Annual prices**: monthly only for v1. Add annual Prices to Stripe
  Product when revenue justifies the proration complexity.
- **B2B / institutional subscriptions**: VAT-ID field on checkout. Wait
  for actual demand.
- **Email change flow**: no endpoint exists yet; Stripe customer email
  sync becomes relevant when it does.
- **User account deletion**: same — no endpoint, but when added, must
  cancel Stripe sub + handle data retention question.
