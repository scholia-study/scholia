# Scholia — DevOps Plan

Design notes and decisions for taking Scholia to production. Distilled from
the design grill on 2026-05-04. This is the reference doc for execution;
no code yet, just decisions + rationale.

---

## 0. Pre-deployment refactor (do first)

Three coupled changes that must land **before any cluster bringup**, all on
one branch:

### 0.1 Move all backend routes under `/api/*`

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

### 0.2 Runtime config injection

Replace build-time Vite env vars with runtime injection so a single
container image works across all environments.

**Pattern:**

- `web/public/config.js.template` — committed source with placeholders
  ```js
  window.__SCHOLIA_CONFIG__ = {
      stripePublishableKey: "${STRIPE_PUBLISHABLE_KEY}",
      apiBaseUrl: "${API_BASE_URL}",
      environment: "${ENVIRONMENT}",
  };
  ```
- nginx Docker entrypoint script in `/docker-entrypoint.d/` runs `envsubst`
  at pod startup, writing the rendered file to `/usr/share/nginx/html/config.js`
- `index.html` loads `<script src="/config.js"></script>` before the app bundle
- Frontend code reads `window.__SCHOLIA_CONFIG__.stripePublishableKey` etc.
- For local `pnpm dev`: a hand-edited `web/public/config.js` exists with the
  dev publishable key (it's public anyway). nginx overwrites this file at
  startup in cluster pods.

**Migrates two existing variables:**

- `import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY` → `window.__SCHOLIA_WEB_CONFIG__.stripePublishableKey`
- Hardcoded `BASE_URL = "http://localhost:4000"` in `fetcher.ts` → `window.__SCHOLIA_WEB_CONFIG__.apiBaseUrl`

### 0.3 Migrations bootstrap (sqlx)

Replace the destructive `db_reset.sh`-only flow with a proper migration
story before any prod data exists.

**Setup:**

- Convert `db/001_schema.sql` → `db/migrations/<timestamp>_initial.sql`
- Add `sqlx-cli` to dev tooling (`cargo install sqlx-cli`)
- Embed migrations into the API binary via `sqlx::migrate!("../../db/migrations")`
- New API binary subcommand: `api migrate` runs migrations and exits
- Update `db_reset.sh`: drops schema, then runs migrations (instead of
  re-applying the legacy 001_schema.sql)
- `_sqlx_migrations` table tracks applied migrations + checksums; sqlx
  refuses to re-run an edited migration

**Rule (non-negotiable post-launch):** migrations are **append-only**.
Every schema change is a new migration file. Never edit a previously-applied
file. Multi-step changes (add nullable col → backfill → make NOT NULL)
become multiple migrations, not one edit.

### 0.4 Update auto-memory note

The existing memory note about editing `db/001_schema.sql` directly + using
`db_reset.sh` becomes stale once 0.3 lands. Update to reference the
migrations directory + `sqlx migrate add` flow.

---

## 1. Infrastructure architecture

### 1.1 Cloud + topology

- **Provider**: Hetzner Cloud
- **Region**: Falkenstein (closest to EU/Norway audience and Stripe EU endpoints)
- **Topology**: two single-node k3s clusters, one per environment

| | dev cluster | prod cluster |
|---|---|---|
| **VPS** | CX22 (2 vCPU, 4GB, 40GB disk) | CX22 (2 vCPU, 4GB, 40GB disk) |
| **Cost** | €4.90/mo | €4.90/mo |
| **OS** | Ubuntu 24.04 LTS | Ubuntu 24.04 LTS |
| **Hostname** | `dev.scholia.study` | `scholia.study` |

**Why two clusters, not namespaces in one cluster:**

- Physical isolation — dev mistakes can't take prod down
- Independent k3s upgrades (test on dev first, schedule prod separately)
- Distinct Stripe webhook URLs by hostname (no routing-rule collisions)
- Doubles K8s muscle (two kubeconfig contexts, real cross-cluster practice)
- Cost difference is ~€5/mo

**Why CX22 prod:** at your scale, RAM budget is ~2-3GB committed (k3s + Postgres + API + nginx + Traefik + cert-manager) on 4GB available. Comfortable for v1; resize to CX32 in-place when traffic warrants.

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

**Routing pattern (Pattern X — single hostname, path-based):**

```
scholia.study/                  → scholia-web (nginx pod, static files)
scholia.study/api/*             → scholia-api (Rust pod)
scholia.study/api/webhooks/*    → scholia-api (Rust pod, raw body, no session/CORS layers)
```

Same shape on `dev.scholia.study`.

**Why single hostname:** same-origin = no CORS plumbing, first-party
cookies for sessions, simple Stripe webhook URL, fewer ingress rules to
get wrong.

---

## 2. Application services in cluster

### 2.1 Rust API (`scholia-api`)

- **Image**: multi-stage Docker build (`rust:bookworm` builder, `debian:bookworm-slim` runtime)
- **Tag**: `ghcr.io/<you>/scholia-api:main-<sha7>` for dev, `:v1.2.3` for prod
- **Pod**: 1 replica per cluster (no horizontal scaling for v1)
- **Init container**: same image, runs `api migrate` to apply DB migrations
  before the main container starts. Migration failure prevents the API
  from starting (clean failure semantics).
- **Public**: through ingress, `/api/*` paths

### 2.2 Web (`scholia-web`)

- **Image**: multi-stage build (`node:22-alpine` builder, `nginx:alpine` runtime)
- **Tag**: same scheme as API
- **Pod**: 1 replica (static files; trivial)
- **Build output**: TanStack Start with `prerender` + `spa.enabled` → fully static
- **Runtime config**: `envsubst` at pod startup writes `/config.js` from env vars
- **Public**: through ingress, all paths not starting with `/api/`

### 2.3 Postgres (`scholia-db`)

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

### 2.4 cert-manager + Traefik

- cert-manager installed via Helm (one-time, post-cluster-up)
- ClusterIssuer: Let's Encrypt prod ACME
- Certificate CRs declared in manifests; cert-manager auto-renews
- Traefik bundled with k3s; kept default

---

## 3. Backups

### 3.1 Target

**Hetzner Object Storage** (S3-compatible). Same provider as VPS — same-region traffic, single vendor, single bill, flat ~€5.99/mo for 1TB incl. egress.

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
- **k3s install**: cloud-init script attached to the VPS; runs `curl -sfL https://get.k3s.io | sh -` on first boot, plus Tailscale install
- **State**: stored in Hetzner Object Storage as the Terraform backend
- **Workspaces**: `dev` and `prod` for the two environments
- **Module structure**: one `modules/cluster/` instantiated twice with different vars

### 4.2 Manifests: Kustomize → ArgoCD

**v0 (initial bringup):**

- Plain Kustomize with `base/` + `overlays/{dev,prod}/`
- Manual `kubectl apply -k overlays/dev/` from your laptop
- CI pushes images + commits image-tag bump to dev overlay
- You apply to prod manually after testing dev

**v1 (before public launch):**

- ArgoCD installed in each cluster (or one Argo managing both via cross-cluster Applications)
- Argo `Application` CR per environment, points at git path
- Sync = `kustomize build` + `kubectl apply` + drift detection
- Optional: Argo Image Updater for auto-bumping dev overlay on every push

The Kustomize bases + overlays from v0 are reused as-is by Argo.

### 4.3 Secrets: SOPS + age

- **Tool**: SOPS (mozilla/getsops) with age keys
- **Keys**: one age key per environment (dev, prod). Distinct, never shared.
- **Storage**: encrypted YAML files committed to git
- **Apply path (v0)**: `sops -d secret.yaml | kubectl apply -f -`
  (or a small `make secrets-dev` wrapper)
- **Apply path (v1, ArgoCD)**: `argocd-vault-plugin` or SOPS-aware plugin sidecar
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
| Push to `main` | Build images, push `:main-<sha>` to ghcr.io, **commit image-tag bump to `infra/k8s/overlays/dev/`** |
| Tag `v*` push | Build images, push `:<version>` and `:latest` to ghcr.io |

- **Runner**: GitHub-hosted (free for public repos)
- **Auth**: `GITHUB_TOKEN` (built-in) for ghcr push and contents:write commit-back
- **No Hetzner / kubectl credentials in CI for v1** — Terraform runs from
  laptop, manual `kubectl apply` from laptop

### 4.6 Image registry: ghcr.io

- **Visibility**: public (image bytes aren't sensitive; secrets live in K8s Secrets)
- **Two images**: `ghcr.io/<you>/scholia-api`, `ghcr.io/<you>/scholia-web`
- **Tag scheme**: `main-<sha7>` for main pushes, semver for releases, never `:latest` in Deployments

---

## 5. Repository layout

```
.
├── packages/api/             # existing — Rust API
├── web/                      # existing — React frontend
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
│       ├── base/             # core manifests (api, web, postgres, ingress)
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

### 6.3 Deploy discipline (v1)

- Push to main → CI builds + bumps dev overlay → manual `kubectl apply -k overlays/dev`
- Verify on dev (smoke-test paid checkout flow, articles, profile)
- Tag a release (`vX.Y.Z`) → CI builds prod-tagged image
- Manual: bump `overlays/prod/kustomization.yaml`, commit, manual `kubectl apply -k overlays/prod`
- Watch logs via `kubectl logs` for first ~10 minutes post-deploy

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

**Don't self-host Grafana/Loki/Prometheus on a CX22.** The stack alone
eats ~1GB RAM, leaving no headroom on a 4GB box.

---

## 8. Cost estimate (monthly)

| Item | Cost |
|---|---|
| Dev VPS (CX22) | €4.90 |
| Prod VPS (CX22) | €4.90 |
| Hetzner Object Storage (1TB pool) | €5.99 |
| Domain (scholia.study, Porkbun) | ~€0.80 (annualized) |
| Tailscale | €0 (personal use) |
| ghcr.io (public images) | €0 |
| GitHub Actions (public repo) | €0 |
| Sentry (free tier, v1) | €0 |
| Grafana Cloud (free tier, v2) | €0 |
| **Total v1** | **~€16.59/mo** |

Stripe takes 1.4% + €0.25 per EU card transaction (different in other
regions). Not infrastructure, but worth tracking.

---

## 9. Roadmap

### v0 — Pre-deployment refactor

- [ ] Move backend routes under `/api/*`
- [ ] Implement runtime config injection (`window.__SCHOLIA_CONFIG__`)
- [ ] Migrations bootstrap (sqlx-cli, init container)
- [ ] Update auto-memory note about `db_reset.sh` flow
- [ ] Regenerate openapi.json + frontend client
- [ ] Update Stripe Dashboard webhook URL post-deploy

### v0 — Cluster bringup

- [ ] Hetzner API token, Porkbun API key, Tailscale auth key in laptop env
- [ ] Write `infra/terraform/` (Hetzner + Porkbun providers, cloud-init for k3s + Tailscale)
- [ ] State backend in Hetzner Object Storage
- [ ] `terraform apply -var-file=dev.tfvars`
- [ ] `terraform apply -var-file=prod.tfvars`
- [ ] **Validate IaC**: `terraform destroy` + `terraform apply` on dev once
- [ ] Install cert-manager, configure Let's Encrypt prod issuer
- [ ] SOPS age keypairs (dev + prod), encrypted backup to HOS + password manager
- [ ] Write `infra/k8s/base/` + `overlays/{dev,prod}/`
- [ ] Build + push first images via CI
- [ ] `kubectl apply -k overlays/dev/`
- [ ] Validate end-to-end on dev (Stripe test charge → role flips → cancellation flow)
- [ ] `kubectl apply -k overlays/prod/`
- [ ] Update Stripe to live mode keys + production webhook URL
- [ ] Public soft launch

### v1 — ArgoCD + Sentry

- [ ] Install ArgoCD in each cluster
- [ ] Migrate Kustomize apply → Argo Application sync
- [ ] SOPS plugin / sidecar for encrypted secret handling in Argo
- [ ] Sentry integration in API + frontend
- [ ] Capacity-aware UX (warn at 80% of free-tier limits)

### v2 — Observability + scale

- [ ] Grafana Cloud Agent in clusters
- [ ] Dashboards for HTTP latency, error rate, DB connections
- [ ] Prod VPS upgrade to CX32 if RAM headroom drops below 1GB
- [ ] Postgres operator (CloudNativePG) migration when PITR / replicas matter

---

## 10. Open questions / deferred decisions

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
