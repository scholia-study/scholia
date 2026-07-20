#!/usr/bin/env bash
#
# Daily Postgres backup. Streams `pg_dump --format=custom | gzip` straight
# to Hetzner Object Storage (scholia-backups/daily/), confirms the object
# landed, then reports over ntfy — success at low priority, failure at
# high so it can't be missed. Runs from the postgres-backup CronJob
# (infra/k8s/base/postgres/backup-cronjob.yaml).
#
# Retention (keep the last 60 daily dumps) is NOT this script's job: it's
# a bucket lifecycle rule applied out of band by scripts/backups_lifecycle.sh.
#
# Env (all from the same Secrets the ingest Jobs use):
#   POSTGRES_HOST/PORT/USER/DB, PGPASSWORD   — the `postgres` Secret
#   AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY — the `assets-bucket` Secret
#   NTFY_URL (optional)                       — the `ntfy` Secret
set -euo pipefail

: "${POSTGRES_HOST:?}"
: "${POSTGRES_USER:?}"
: "${POSTGRES_DB:?}"
: "${PGPASSWORD:?}"

source /app/scripts/lib.sh
scholia_rclone_config

ts="$(date -u +%Y%m%dT%H%M%SZ)"
dump="/tmp/scholia-${ts}.dump.gz"
object="daily/${ts}.dump.gz"
dest="scholia:scholia-backups/${object}"

notify() {
    status=$1 msg=$2
    [ -n "${NTFY_URL:-}" ] || return 0
    if [ "$status" -eq 0 ]; then
        prio=low; tags=floppy_disk
    else
        prio=high; tags=rotating_light
    fi
    curl -fsS -o /dev/null -m 10 \
        -H "Title: db-backup" \
        -H "Priority: ${prio}" \
        -H "Tags: ${tags}" \
        -d "$msg" "$NTFY_URL" || true
}

on_exit() {
    status=$?
    rm -f "$dump"
    if [ "$status" -ne 0 ]; then
        notify "$status" "FAILED at ${ts} (exit ${status}) — no dump uploaded"
    fi
}
trap on_exit EXIT

echo "Dumping ${POSTGRES_DB} on ${POSTGRES_HOST} → ${dest} ..."
pg_dump \
    --format=custom \
    --host="$POSTGRES_HOST" \
    --port="${POSTGRES_PORT:-5432}" \
    --username="$POSTGRES_USER" \
    "$POSTGRES_DB" | gzip -c >"$dump"

bytes=$(stat -c%s "$dump")
echo "Dump is ${bytes} bytes; uploading ..."
rclone copyto "$dump" "$dest"

# Trust nothing: confirm the object is actually listed before calling it a
# backup. Hetzner listings can lag fresh writes, so poll briefly.
found=""
for attempt in 1 2 3 4 5; do
    if rclone lsf "scholia:scholia-backups/daily/" | grep -qxF "${ts}.dump.gz"; then
        found=yes
        break
    fi
    echo "waiting for ${object} to appear in listings (attempt ${attempt}/5)..."
    sleep 5
done
if [ -z "$found" ]; then
    echo "error: ${object} never appeared in listings after upload." >&2
    exit 1
fi

human=$(numfmt --to=iec --suffix=B "$bytes" 2>/dev/null || echo "${bytes}B")
echo "Backup complete: ${object} (${human})"
notify 0 "${object} ok (${human})"
