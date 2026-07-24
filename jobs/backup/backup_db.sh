#!/usr/bin/env bash
#
# Daily Postgres backup, mirrored across three Hetzner Object Storage
# regions for redundancy. Takes one `pg_dump --format=custom | gzip` and
# fans the same dump out to all three buckets, confirming each landed.
# Silent on success; alerts over ntfy only when a dump fails.
#
# Object key is `db/daily/<env>/<ts>.dump.gz`: db (backup type) → daily
# (cadence) → env (SCHOLIA_ENV). Dev and prod live in separate namespaces
# so they never collide, and one fixed lifecycle rule on `db/daily/`
# covers every environment no matter how many we add.
#
# Failure policy: the night is a success as long as at least one region
# received the dump (you can always restore). A partial failure still
# succeeds but fires a high-priority alert naming the down region(s); a
# total failure exits non-zero. Retention (keep the last 60 daily dumps)
# is a per-env lifecycle rule applied out of band by
# scripts/backups_lifecycle.sh.
#
# Env (Secrets as the ingest Jobs use; SCHOLIA_ENV from the CronJob):
#   SCHOLIA_ENV                               — dev | prod (base defaults
#                                               dev; the prod overlay patches
#                                               it, like APP_PROFILE)
#   POSTGRES_HOST/PORT/USER/DB, PGPASSWORD    — the `postgres` Secret
#   AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY — the `assets-bucket` Secret
#                                               (project-scoped: one keypair
#                                               works across all regions)
#   NTFY_URL (optional)                       — the `ntfy` Secret
set -euo pipefail

: "${SCHOLIA_ENV:?}"
: "${POSTGRES_HOST:?}"
: "${POSTGRES_USER:?}"
: "${POSTGRES_DB:?}"
: "${PGPASSWORD:?}"
: "${AWS_ACCESS_KEY_ID:?}"
: "${AWS_SECRET_ACCESS_KEY:?}"

# region  bucket — primary first; the dump is mirrored to every entry.
TARGETS=(
    "fsn1 scholia-backups"        # Falkenstein
    "hel1 scholia-backups-sigma"  # Helsinki
    "nbg1 scholia-backups-tau"    # Nuremberg
)

# An rclone remote per region: same credentials, region-specific endpoint.
# Env-var config keys are uppercase; the remote is referenced lowercase.
setup_remote() {
    local region=$1 up
    up=$(echo "$region" | tr 'a-z' 'A-Z')
    export "RCLONE_CONFIG_${up}_TYPE=s3"
    export "RCLONE_CONFIG_${up}_PROVIDER=Other"
    export "RCLONE_CONFIG_${up}_ENDPOINT=https://${region}.your-objectstorage.com"
    export "RCLONE_CONFIG_${up}_REGION=${region}"
    export "RCLONE_CONFIG_${up}_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID}"
    export "RCLONE_CONFIG_${up}_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY}"
    export "RCLONE_CONFIG_${up}_FORCE_PATH_STYLE=true"
}

ts="$(date -u +%Y%m%dT%H%M%SZ)"
dump="/tmp/scholia-${ts}.dump.gz"
object="db/daily/${SCHOLIA_ENV}/${ts}.dump.gz"

alert() {
    local msg=$1
    [ -n "${NTFY_URL:-}" ] || return 0
    curl -fsS -o /dev/null -m 10 \
        -H "Title: db-backup" \
        -H "Priority: high" \
        -H "Tags: rotating_light" \
        -d "$msg" "$NTFY_URL" || true
}

phase=setup
on_exit() {
    local status=$?
    rm -f "$dump"
    # Normal outcomes notify themselves and set phase=done; this only
    # catches an unexpected death mid-run.
    if [ "$status" -ne 0 ] && [ "$phase" != done ]; then
        alert "FAILED during ${phase} at ${ts} (exit ${status})"
    fi
}
trap on_exit EXIT

# kube-router can take a second or two to program a freshly-started pod's
# IP into Postgres's allow-ipset. This Job hits the DB as its very first
# action, so without waiting it races the enforcer and gets a spurious
# "connection refused". Block until Postgres actually accepts connections.
# (The ingest Jobs never hit this because they pull from S3 first.)
echo "Waiting for ${POSTGRES_HOST}:${POSTGRES_PORT:-5432} to accept connections..."
for attempt in $(seq 1 30); do
    pg_isready -h "$POSTGRES_HOST" -p "${POSTGRES_PORT:-5432}" -q && break
    if [ "$attempt" -eq 30 ]; then
        echo "error: ${POSTGRES_HOST}:${POSTGRES_PORT:-5432} unreachable after 30 tries" >&2
        exit 1
    fi
    sleep 2
done

phase=dump
echo "Dumping ${POSTGRES_DB} on ${POSTGRES_HOST} → ${object} ..."
pg_dump \
    --format=custom \
    --host="$POSTGRES_HOST" \
    --port="${POSTGRES_PORT:-5432}" \
    --username="$POSTGRES_USER" \
    "$POSTGRES_DB" | gzip -c >"$dump"

bytes=$(stat -c%s "$dump")
human=$(numfmt --to=iec --suffix=B "$bytes" 2>/dev/null || echo "${bytes}B")
echo "Dump is ${human}; mirroring to ${#TARGETS[@]} regions ..."

phase=upload
succeeded=()
failed=()
for entry in "${TARGETS[@]}"; do
    read -r region bucket <<<"$entry"
    setup_remote "$region"
    # copy + confirm the object is actually listed before trusting it.
    if rclone copyto "$dump" "${region}:${bucket}/${object}" 2>&1 &&
        rclone lsf "${region}:${bucket}/db/daily/${SCHOLIA_ENV}/" | grep -qxF "${ts}.dump.gz"; then
        echo "  ${region}:${bucket} ok"
        succeeded+=("$region")
    else
        echo "  ${region}:${bucket} FAILED" >&2
        failed+=("$region")
    fi
done

phase=done
if [ ${#succeeded[@]} -eq 0 ]; then
    echo "Backup FAILED: no region accepted ${object}" >&2
    alert "${object} FAILED to ALL regions (${TARGETS[*]%% *})"
    exit 1
fi
if [ ${#failed[@]} -gt 0 ]; then
    echo "Backup degraded: ok=[${succeeded[*]}] failed=[${failed[*]}]"
    alert "${object} degraded (${human}) — ok:[${succeeded[*]}] FAILED:[${failed[*]}]"
else
    echo "Backup complete to all regions: ${object} (${human})"
fi
