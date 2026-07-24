#!/usr/bin/env bash
#
# Restore the Postgres database from a daily backup, driven from your
# shell against whatever cluster your kubectl context points at.
#
# The target env (dev|prod) is read from the cluster's own postgres-backup
# CronJob, so the dump pulled is always the one THIS cluster produced —
# you can't accidentally restore a dev dump onto prod or vice versa.
#
# Usage:
#   scripts/db_restore.sh [--dump <key|latest>] [--region fsn1|hel1|nbg1] [--dry-run]
#     --dump    object under db/daily/<env>/ (default: latest)
#     --region  which mirror to pull from (default: fsn1)
#     --dry-run resolve + print the plan, change nothing
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NS=scholia
DEPLOY=scholia-api
POD=db-restore
POD_MANIFEST="${ROOT}/infra/k8s/jobs/db-restore-pod.yaml"

DUMP=latest
REGION=fsn1
DRY=0
while [ $# -gt 0 ]; do
    case "$1" in
        --dump) DUMP=$2; shift 2 ;;
        --region) REGION=$2; shift 2 ;;
        --dry-run) DRY=1; shift ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
done

case "$REGION" in
    fsn1) BUCKET=scholia-backups ;;
    hel1) BUCKET=scholia-backups-sigma ;;
    nbg1) BUCKET=scholia-backups-tau ;;
    *) echo "bad --region: $REGION (fsn1|hel1|nbg1)" >&2; exit 2 ;;
esac

kctx=$(kubectl config current-context)
server=$(kubectl config view --minify -o jsonpath='{.clusters[0].cluster.server}')

env=$(kubectl -n "$NS" get cronjob postgres-backup \
    -o jsonpath='{.spec.jobTemplate.spec.template.spec.containers[0].env[?(@.name=="SCHOLIA_ENV")].value}' 2>/dev/null || true)
[ -n "$env" ] || {
    echo "could not read SCHOLIA_ENV from the postgres-backup CronJob." >&2
    echo "Is your kubectl context pointed at a Scholia cluster?" >&2
    exit 1
}

echo "→ environment: ${env^^}   (context: ${kctx} @ ${server})"
echo "→ launching restore pod ..."
kubectl -n "$NS" delete pod "$POD" --ignore-not-found --wait=true >/dev/null
kubectl -n "$NS" apply -f "$POD_MANIFEST" >/dev/null
kubectl -n "$NS" wait --for=condition=ready "pod/$POD" --timeout=90s >/dev/null

target_db=$(kubectl -n "$NS" exec "$POD" -- printenv POSTGRES_DB)

# rclone setup reused by the resolve + restore execs.
rclone_env() {
    cat <<RC
export RCLONE_CONFIG_R_TYPE=s3 RCLONE_CONFIG_R_PROVIDER=Other \
  RCLONE_CONFIG_R_ENDPOINT=https://${REGION}.your-objectstorage.com \
  RCLONE_CONFIG_R_REGION=${REGION} \
  RCLONE_CONFIG_R_ACCESS_KEY_ID="\$AWS_ACCESS_KEY_ID" \
  RCLONE_CONFIG_R_SECRET_ACCESS_KEY="\$AWS_SECRET_ACCESS_KEY" \
  RCLONE_CONFIG_R_FORCE_PATH_STYLE=true
RC
}

if [ "$DUMP" = latest ]; then
    key=$(kubectl -n "$NS" exec "$POD" -- bash -c \
        "$(rclone_env); rclone lsf r:${BUCKET}/db/daily/${env}/ | sort | tail -1")
else
    key=$(basename "$DUMP")
fi
[ -n "$key" ] || {
    echo "no dump found under db/daily/${env}/ in ${BUCKET}" >&2
    kubectl -n "$NS" delete pod "$POD" --wait=false >/dev/null
    exit 1
}

replicas=$(kubectl -n "$NS" get deploy "$DEPLOY" -o jsonpath='{.spec.replicas}')

# Turn the dump's UTC timestamp key (…YYYYMMDDTHHMMSSZ.dump.gz) into a
# readable "taken" line with an age, so you can see how fresh it is.
stamp=${key%.dump.gz}
if [[ $stamp =~ ^([0-9]{4})([0-9]{2})([0-9]{2})T([0-9]{2})([0-9]{2})([0-9]{2})Z$ ]]; then
    iso="${BASH_REMATCH[1]}-${BASH_REMATCH[2]}-${BASH_REMATCH[3]}T${BASH_REMATCH[4]}:${BASH_REMATCH[5]}:${BASH_REMATCH[6]}Z"
    taken=$(LC_ALL=C date -u -d "$iso" '+%a %Y-%m-%d %H:%M UTC' 2>/dev/null || echo "$iso")
    secs=$(date -u -d "$iso" +%s 2>/dev/null || echo 0)
    [ "$secs" -gt 0 ] && taken="${taken}  ($(( ($(date -u +%s) - secs) / 3600 ))h ago)"
else
    taken="(unrecognised timestamp: ${stamp})"
fi

cat <<SUMMARY

  +-- DB restore --------------------------------------------
  | environment: ${env^^}   (from postgres-backup CronJob)
  | context:     ${kctx} @ ${server}
  | source:      ${REGION}:${BUCKET}/db/daily/${env}/${key}
  | taken:       ${taken}
  | target DB:   ${target_db}   (LIVE — will be replaced)
  | deployment:  ${DEPLOY}   (${replicas} replica(s); scaled to 0 during restore)
  +----------------------------------------------------------
SUMMARY

if [ "$DRY" = 1 ]; then
    echo "dry-run: nothing changed."
    kubectl -n "$NS" delete pod "$POD" --wait=false >/dev/null
    exit 0
fi

printf 'About to REPLACE the live %s database in the %s environment with this backup.\nContinue? [y/N] ' "$target_db" "${env^^}"
read -r ok </dev/tty
case "$ok" in
    y | Y | yes | YES) ;;
    *) echo "aborted."; kubectl -n "$NS" delete pod "$POD" --wait=false >/dev/null; exit 1 ;;
esac

printf 'Final check — type the database name (%s) to proceed: ' "$target_db"
read -r reply </dev/tty
[ "$reply" = "$target_db" ] || {
    echo "aborted."
    kubectl -n "$NS" delete pod "$POD" --wait=false >/dev/null
    exit 1
}

# API is about to go down. Always bring it back and clean up the pod, even
# on error. The restore runs --single-transaction, so a failure rolls the
# DB back to its prior state — restoring service is safe either way.
cleanup() {
    echo "→ scaling ${DEPLOY} back to ${replicas} ..."
    kubectl -n "$NS" scale deploy "$DEPLOY" --replicas="$replicas" >/dev/null || true
    kubectl -n "$NS" delete pod "$POD" --wait=false >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "→ scaling ${DEPLOY} to 0 ..."
kubectl -n "$NS" scale deploy "$DEPLOY" --replicas=0 >/dev/null
for _ in $(seq 1 60); do
    [ "$(kubectl -n "$NS" get pods -l app.kubernetes.io/name="$DEPLOY" -o name 2>/dev/null | wc -l)" -eq 0 ] && break
    sleep 2
done

echo "→ restoring ${key} into ${target_db} ..."
kubectl -n "$NS" exec -i "$POD" -- bash -s -- "$REGION" "$BUCKET" "$env" "$key" "$target_db" <<'POD'
set -euo pipefail
REGION=$1 BUCKET=$2 ENV=$3 KEY=$4 TARGET_DB=$5
export RCLONE_CONFIG_R_TYPE=s3 RCLONE_CONFIG_R_PROVIDER=Other \
  RCLONE_CONFIG_R_ENDPOINT="https://${REGION}.your-objectstorage.com" \
  RCLONE_CONFIG_R_REGION="$REGION" \
  RCLONE_CONFIG_R_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID" \
  RCLONE_CONFIG_R_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY" \
  RCLONE_CONFIG_R_FORCE_PATH_STYLE=true

rclone copyto "r:${BUCKET}/db/daily/${ENV}/${KEY}" /tmp/restore.dump.gz
gunzip -f /tmp/restore.dump.gz

for _ in $(seq 1 30); do pg_isready -h "$POSTGRES_HOST" -q && break; sleep 2; done

# Drop any lingering sessions on the target DB so --clean isn't blocked.
psql -h "$POSTGRES_HOST" -U "$POSTGRES_USER" -d postgres -v ON_ERROR_STOP=1 -c \
  "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname='${TARGET_DB}' AND pid<>pg_backend_pid();" >/dev/null

pg_restore -h "$POSTGRES_HOST" -U "$POSTGRES_USER" -d "$TARGET_DB" \
  --clean --if-exists --no-owner --single-transaction /tmp/restore.dump

rm -f /tmp/restore.dump
echo "  in-pod restore OK"
POD

echo "→ restore complete."
echo "DONE — ${DEPLOY} will be scaled back to ${replicas} and the pod removed."
