# PLAN: Adopting ArgoCD

Status: agreed implementation plan (decisions resolved via design review).
Builds on the v1 step named in `PLAN_DEVOPS.md` (§4.2 Kustomize → ArgoCD,
§4.3 secrets, §4.4 migrations, v1 checklist). This is the concrete how.

Scope of this first cut: **dev cluster only**, structured so prod is a
copy-paste once its cluster exists.

## Bottom line

The repo is already shaped for GitOps: `infra/k8s/` is `base/` +
`overlays/dev` Kustomize with git as the source of truth — exactly what
Argo consumes. This is mostly *additive infra*, not a refactor. The one
real wrinkle is secrets (SOPS); everything else is plumbing.

## Resolved decisions

| # | Branch | Decision |
|---|--------|----------|
| 1 | Scope | Dev only now; structure so prod is copy-paste later |
| 2 | Topology | One self-contained Argo per cluster (in-cluster destination) |
| 3 | Argo owns | api, web, proxy, postgres + secrets. **Not** jobs (manual kubectl). cert-manager stays manual |
| 4 | SOPS decrypt | KSOPS as a CMP sidecar on `argocd-repo-server` |
| 5 | Age key | Manual one-time `kubectl create secret -n argocd` over Tailscale |
| 6 | Argo install | argo-cd Helm chart, version-pinned, `values.yaml` in repo, manual `helm upgrade --install` |
| 7 | Sync policy | Automated: prune + selfHeal |
| 8 | Image flow | CI git write-back; kustomize `images:` block pinned to immutable `main-<sha>` |
| 9 | Argo access | Tailscale port-forward only; no ingress for Argo |
| 10 | Write-back | Bot-bypass on protected `main`; CI pushes the bump commit directly |

## Prerequisites (must land before / alongside implementation)

These are gating — the chosen image-flow and access model assume them.

1. **Make the repo public.** Currently `scholia-study/scholia` is private,
   which blocks free branch protection. Going public is also what lets
   GHCR packages be public (drops imagePullSecret dependence).
2. **Pre-public secret audit (do BEFORE flipping visibility):**
   - Confirm every file in `infra/k8s/overlays/dev/secrets/` is actually
     SOPS-encrypted (ciphertext in `data`/`stringData`, not plaintext).
   - Scan git *history* for any sensitive blob ever committed (a public
     repo exposes all history, not just HEAD). `.env` is gitignored and
     was never tracked — good — but verify nothing else slipped in.
3. **Branch protection on `main`** (a free ruleset on public repos):
   require PR + review for humans; add the **GitHub Actions bot to the
   bypass list** so CI can push the image-tag bump commit directly.
4. **CI permissions:** `build.yml` currently has `contents: read`. The
   write-back step needs `contents: write`.
5. **GHCR package visibility:** once public, set the five
   `scholia-*` packages to public (or confirm the existing pull path).

Until these land, Argo can still be stood up in read-only / manual-sync
mode for drift detection, but the auto-deploy loop (decision 8) depends
on prereqs 1, 3, 4.

## Repo layout (new + changed files)

```
infra/argo/                      # NEW — Argo's own config (NOT synced by Argo)
  values.yaml                    #   Helm values for installing Argo + KSOPS sidecar
  application-dev.yaml           #   the Application CR (registered once at bootstrap)
  README.md                      #   ordered bootstrap runbook

infra/k8s/overlays/dev/
  kustomization.yaml             # CHANGED — add images: block + KSOPS generator
  secret-generator.yaml          # NEW — ksops generator listing encrypted files
  secrets/*.yaml                 # unchanged (stay SOPS-encrypted in git)

.github/workflows/build.yml      # CHANGED — add contents:write + tag-bump write-back step
```

### Why a single `Application` (not app-of-apps)

An Argo `Application` is the CR that says "watch this git path, keep the
cluster matching it." We have exactly one app to deploy, so one
`Application` (pointing at `overlays/dev`) covers api/web/proxy/postgres/
ingress/secrets. App-of-apps (a root Application that creates child
Applications) is for managing many apps/add-ons declaratively — ceremony
with no payoff here. Revisit only if cert-manager/monitoring later get
folded into Argo.

`application-dev.yaml` lives in the repo (version-controlled) but is
registered with Argo by a **one-time** `kubectl apply` during bootstrap.
After that Argo self-manages; re-apply only if the Application spec
itself changes.

### The `Application` CR

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: scholia-dev
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/scholia-study/scholia.git
    targetRevision: main
    path: infra/k8s/overlays/dev
  destination:
    server: https://kubernetes.default.svc
    namespace: scholia
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=false   # base/namespace.yaml already creates it
```

## The two overlay changes

### (a) `images:` block — the CI write-back target

Kustomize overrides image tags without editing base Deployments. The base
hardcodes `:main`; the overlay pins immutable per-build tags:

```yaml
# overlays/dev/kustomization.yaml
images:
  - name: ghcr.io/scholia-study/scholia-api
    newTag: main-PLACEHOLDER    # CI rewrites this line per build
  - name: ghcr.io/scholia-study/scholia-web
    newTag: main-PLACEHOLDER
  - name: ghcr.io/scholia-study/scholia-proxy
    newTag: main-PLACEHOLDER
```

This is the single line(s) CI edits. Argo sees the diff → syncs → pods
roll to the new immutable image. With immutable tags we also **drop
`imagePullPolicy: Always`** from the base Deployments (no longer needed).

### (b) KSOPS generator — Argo-owned secrets

Today `secrets/` is deliberately excluded from the overlay (applied
out-of-band via `sops -d | kubectl apply`). To let Argo own them, point
Kustomize at them through KSOPS:

```yaml
# overlays/dev/kustomization.yaml
generators:
  - secret-generator.yaml
```

```yaml
# overlays/dev/secret-generator.yaml
apiVersion: viaduct.ai/v1
kind: ksops
metadata:
  name: scholia-secrets
  annotations:
    config.kubernetes.io/function: |
      exec: { path: ksops }
files:
  - secrets/postgres.yaml
  - secrets/api.yaml
  - secrets/assets-bucket.yaml
  - secrets/proxy-dev-gate.yaml
```

When Argo's repo-server runs `kustomize build`, the KSOPS plugin decrypts
these inline using the mounted age key and emits real `Secret` resources.
Encrypted files stay in git; plaintext only exists in repo-server memory
at build time.

## Installing Argo with the KSOPS sidecar

argo-cd Helm chart, version-pinned, values in `infra/argo/values.yaml`.
The key customization is a KSOPS init/sidecar on `argocd-repo-server`
plus the age key mounted from a Secret. Shape (final pinning at install):

```yaml
# infra/argo/values.yaml (sketch)
configs:
  cmp:
    create: true
    plugins:
      ksops:
        generate:
          command: ["sh", "-c", "kustomize build --enable-alpha-plugins --enable-exec ."]
repoServer:
  env:
    - name: SOPS_AGE_KEY_FILE
      value: /home/argocd/.config/sops/age/keys.txt
  volumes:
    - name: sops-age
      secret:
        secretName: sops-age           # created manually at bootstrap
  volumeMounts:
    - name: sops-age
      mountPath: /home/argocd/.config/sops/age
  initContainers:
    - name: install-ksops              # downloads ksops + kustomize into repo-server
      ...
server:
  service:
    type: ClusterIP                    # no public ingress; reach via port-forward
```

(Exact KSOPS wiring — official chart's CMP plugin vs the
`viaduct-ai/kustomize-sops` init-container recipe — finalized when the
files are written.)

## Migration ordering

The api Deployment's init container runs `api migrate`, which needs
Postgres reachable.

- **Default (do nothing): init-container crash-retry.** Argo applies all
  resources together; if the migrate init container starts before
  Postgres is ready it exits non-zero and Kubernetes restarts it with
  backoff until Postgres accepts connections, then migrations run and the
  main container proceeds. Self-healing already; only cost is some
  CrashLoopBackOff noise on a cold boot that clears itself.
- **Opt-in hardening: Argo sync-waves.** Annotate postgres
  `argocd.argoproj.io/sync-wave: "0"` and api `"1"`; Argo applies wave 0,
  waits for health, then wave 1. Cleaner cold boot, slightly slower sync.

Plan: start with crash-retry; add sync-waves only if cold-boot noise
proves annoying. One-annotation upgrade if so.

## CI write-back (build.yml)

After the existing build+push matrix, add a step that:
1. Computes the short sha tag (`main-<sha7>`) already produced by
   `docker/metadata-action`.
2. Rewrites the `newTag:` lines in `overlays/dev/kustomization.yaml`
   (e.g. via `kustomize edit set image` or `yq`).
3. Commits + pushes directly to `main` as the Actions bot (on the
   protection bypass list).

Loop-safety is doubly assured: the bump commit touches only `infra/**`,
which is **not** in `build.yml`'s `paths:` filter; and commits made with
the default `GITHUB_TOKEN` don't trigger workflows anyway.

Needs `permissions: contents: write` added to the workflow.

## Argo access

No ingress for Argo (`server.service.type: ClusterIP`). Reach the UI/API
over the tailnet:

```
kubectl port-forward -n argocd svc/argocd-server 8080:443
# then https://localhost:8080
```

Zero public surface beyond Tailscale itself.

## Bootstrap runbook (one-time, per cluster, over Tailscale)

1. `helm upgrade --install argocd argo/argo-cd -n argocd --create-namespace -f infra/argo/values.yaml`
2. Create the age key Secret (KSOPS reads it):
   `kubectl create secret generic sops-age -n argocd --from-file=keys.txt=<age-key-from-1password>`
3. `kubectl apply -f infra/argo/application-dev.yaml`
4. Verify: `kubectl get applications -n argocd` shows `scholia-dev`
   Synced/Healthy; `kubectl get pods -n scholia` matches git.
5. Retire the manual `sops -d | kubectl apply` + `kubectl apply -k`
   steps from the dev bringup docs (Argo owns them now).

## Implementation checklist

- [ ] **Prereqs**: history/secret audit → repo public → branch protection
      + bot bypass → GHCR packages public.
- [ ] `build.yml`: add `contents: write` + tag-bump write-back step.
- [ ] Overlay: add `images:` block (placeholder tags), drop
      `imagePullPolicy: Always` from base Deployments.
- [ ] Overlay: add `secret-generator.yaml` + `generators:` entry.
- [ ] `infra/argo/values.yaml` (chart pin + KSOPS sidecar + age mount +
      ClusterIP server).
- [ ] `infra/argo/application-dev.yaml`.
- [ ] `infra/argo/README.md` (bootstrap runbook above).
- [ ] Bootstrap dev cluster; verify Synced/Healthy.
- [ ] One end-to-end test: push a trivial app change → CI builds → bump
      commit → Argo auto-syncs → pod rolls.

## Deferred to prod / later

- Prod overlay + prod age keypair + `.sops.yaml` prod rule + prod
  `Application` CR (copy of dev with prod path/repo revision).
- Optional app-of-apps if cert-manager / monitoring get folded into Argo.
- Optional Argo Image Updater (we chose CI write-back instead; revisit
  only if CI-side bumping becomes a burden).
- Sync-waves (only if cold-boot crash-retry proves noisy).
