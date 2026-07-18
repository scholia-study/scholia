# ArgoCD — dev cluster

GitOps for the dev cluster. ArgoCD watches `infra/k8s/overlays/dev` on
`main` and keeps the `scholia` namespace matching it (api, web, proxy,
postgres, ingress, and the SOPS-decrypted Secrets). Image deploys happen
by CI committing immutable `main-<sha>` tag bumps to the overlay; Argo
syncs the diff.

See `../../PLAN_ARGOCD.md` for the full rationale and decisions.

## What Argo manages vs. doesn't

| Managed by Argo | Stays manual |
|-----------------|--------------|
| api, web, proxy, postgres | Bible ingest (`infra/k8s/jobs/ingest-bible.yaml`, kubectl) |
| ingress + redirect middleware | cert-manager + ClusterIssuers (one-time install) |
| SOPS-encrypted Secrets (via KSOPS) | the `sops-age` key Secret (bootstrap step 2 below) |
| auto-ingest Jobs (`overlays/dev/ingest-jobs/`, hash-named, CI-bumped) | manual ingest escape hatch (`infra/k8s/jobs/`, kubectl) |

## Files

- `values.yaml` — Helm values for installing Argo + the KSOPS sidecar on
  the repo-server. Applied manually; not synced by Argo.
- `application-dev.yaml` — the Application CR. Registered once (step 3).
- `secret-generator.yaml` lives in the overlay, not here — it's part of
  what Argo builds.

## Prerequisites

- Repo is public; `main` is branch-protected with the write-back PAT on
  the bypass list (CI commits tag bumps directly — see `build.yml`).
- `cert-manager` + ClusterIssuers already installed on the cluster.
- The dev **age private key** available locally (from 1Password/Bitwarden).
- `helm`, `kubectl`, and a kubeconfig for the dev cluster (fetched over
  the tailnet) on your machine.

## Bootstrap (one-time, over Tailscale)

> **Before you start:** the dev overlay ships `newTag: main-PLACEHOLDER`
> for api/web/proxy. These resolve to real `main-<sha>` tags the first
> time CI's `bump` job runs. Because this change touches
> `.github/workflows/build.yml` — which is in every service's path
> filter — the merge that lands it rebuilds **all** services and bumps
> all three tags at once. Confirm `overlays/dev/kustomization.yaml` no
> longer contains `PLACEHOLDER` on `main` before step 3, or Argo will
> try to pull a tag that was never pushed.

```sh
# 1. Install ArgoCD with the KSOPS-enabled repo-server.
helm repo add argo https://argoproj.github.io/argo-helm
helm repo update
helm upgrade --install argocd argo/argo-cd \
  -n argocd --create-namespace \
  --version 9.5.20 \
  -f infra/argo/values.yaml

# 2. Give KSOPS the age private key (NOT in git, NOT in Terraform state).
#    The key file must be named keys.txt so SOPS_AGE_KEY_FILE resolves.
kubectl create secret generic sops-age \
  -n argocd \
  --from-file=keys.txt=/path/to/dev-age-key.txt

# 3. Register the app. From here Argo self-manages.
kubectl apply -f infra/argo/application-dev.yaml

# 4. Verify.
kubectl get applications -n argocd          # scholia-dev → Synced / Healthy
kubectl get pods -n scholia                 # matches git
```

If you created the `sops-age` Secret after the repo-server pods started
(e.g. order 1 → 3 → 2), restart the repo-server so it picks up the mount:

```sh
kubectl rollout restart deploy/argocd-repo-server -n argocd
```

## Accessing the UI

No ingress — reach it over the tailnet:

```sh
kubectl port-forward -n argocd svc/argocd-server 8080:443
# https://localhost:8080
# initial admin password:
kubectl -n argocd get secret argocd-initial-admin-secret \
  -o jsonpath='{.data.password}' | base64 -d; echo
```

## Day-to-day

- **App code change** → push to `main` → CI builds + pushes images →
  CI commits the `main-<sha>` bump to `overlays/dev/kustomization.yaml`
  → Argo auto-syncs → pods roll. No manual deploy step.
- **Manifest change** (env var, resource limit, etc.) → just merge to
  `main`; Argo syncs it.
- **Secret change** → `sops infra/k8s/overlays/dev/secrets/<file>.yaml`,
  commit the re-encrypted file; Argo decrypts + applies on sync.
- **Curated MD / parser change** → merge to `main` → CI builds structs,
  uploads `derived@<hash>` to `scholia-assets-auto`, and bumps the
  corpus's Job manifest (`overlays/dev/ingest-jobs/`) in the same commit
  as any image tags → Argo creates the new hash-named Job (wave 1, after
  api/migrations) → `struct_to_db` reconciles. A failed reconcile (e.g.
  the aligner's sim < 0.90 bail) shows as a red Job / Degraded app —
  land the edit as two passes, per the reconcile design.
- **Re-run an ingest** → delete the finished Job
  (`kubectl delete job ingest-<corpus>-<hash>`); selfHeal recreates and
  re-runs it. For a `derived@<hash>` that hit the auto-ingest bucket's
  30-day expiry, re-run the Build workflow (`workflow_dispatch`; set
  images=false for a structs-only run) — it rebuilds and re-uploads
  current hashes.
- **Ingest without CI** (haywire escape hatch) →
  `kubectl create -f infra/k8s/jobs/ingest-<corpus>.yaml` still works:
  those manifests have no `DERIVED_HASH`, so the entrypoint pulls the
  manually mirrored `scholia-assets` bucket (`just struct <corpus>` +
  `just assets-sync` first).
- **Manual hotfix via kubectl** → will be reverted by selfHeal. That's
  intended; put the fix in git.
- **Force a sync / inspect drift** → the UI (port-forward) or
  `argocd app sync scholia-dev` / `argocd app diff scholia-dev`.

## Local `kustomize build`

Building the overlay locally needs the KSOPS plugin and the age key:

```sh
export SOPS_AGE_KEY_FILE=/path/to/dev-age-key.txt   # or keys.txt
kustomize build --enable-alpha-plugins --enable-exec \
  infra/k8s/overlays/dev
```

(`ksops` and `kustomize` must be on PATH.)

## Adding prod later

Copy this directory's pattern: a prod age keypair + `.sops.yaml` rule, a
prod overlay, an `application-prod.yaml` (same shape, prod path), and a
second Argo on the prod cluster (one Argo per cluster). Nothing here is
shared cross-cluster.
