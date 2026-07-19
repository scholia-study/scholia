#!/usr/bin/env bash
#
# Job-image entrypoint for every struct-importer corpus. `CORPUS` (env)
# selects which one: kant1 | kant3 | shakespeare1 | milton1 | ibsen1.
# Pulls the corpus's derived struct JSON(s) from Hetzner Object Storage,
# then runs the shared per-corpus manifest (scripts/ingest.sh) with the
# baked-in struct_to_db binary. With no flags the importer reconciles in
# place when the book already exists (preserving sentence UUIDs and the
# quotations/notes anchored to them) and fresh-inserts on first run.
#
# Two pull modes:
#   DERIVED_HASH set   — auto-ingest (Argo-managed Job): pull the immutable
#                        CI-built artifact scholia-assets-auto/<corpus>/
#                        derived@<hash> (hash = scripts/derived_hash.sh).
#   DERIVED_HASH unset — manual flow (kubectl create -f infra/k8s/jobs/…):
#                        pull scholia-assets/<corpus>/derived, the manually
#                        mirrored bucket (`just struct <corpus>` +
#                        `just assets-sync`).
set -euo pipefail

: "${CORPUS:?Set CORPUS to a struct-importer corpus (see SCHOLIA_CORPORA in scripts/lib.sh)}"

cd /app
source /app/scripts/lib.sh

REPORT="$(mktemp)"
exec > >(tee "$REPORT") 2>&1

report_and_notify() {
    status=$?
    sleep 1 # let tee drain
    ts="$(date -u +%Y%m%dT%H%M%SZ)"
    outcome=$([ "$status" -eq 0 ] && echo ok || echo failed)
    rclone copyto "$REPORT" \
        "scholia:scholia-assets-auto/${CORPUS}/reports/${ts}-${DERIVED_HASH:-manual}-${outcome}.log" \
        2>/dev/null || true
    if [ "$status" -eq 0 ] && [ -n "${NTFY_URL:-}" ]; then
        summary="$(grep -A1 '=== Reconcile ===' "$REPORT" |
            grep -v '=== \|^--' | sed 's/^ *//' | paste -sd '; ' - || true)"
        curl -fsS -o /dev/null -m 10 \
            -H "Title: ingest ${CORPUS}" \
            -H "Priority: low" \
            -H "Tags: package" \
            -d "${CORPUS}@${DERIVED_HASH:-manual}: ${summary:-done}" \
            "$NTFY_URL" || true
    fi
}
trap report_and_notify EXIT

scholia_rclone_config
if [ -n "${DERIVED_HASH:-}" ]; then
    SRC="scholia:scholia-assets-auto/${CORPUS}/derived@${DERIVED_HASH}"
    # CI ends every upload with a `.complete` marker. Freshly created
    # prefixes can lag in Hetzner's S3 listings (seen in the wild: a Job
    # listed an empty prefix minutes after a verified upload), so poll
    # rather than trusting the first answer.
    found=""
    for attempt in 1 2 3 4 5 6; do
        if rclone lsf "$SRC" --max-depth 1 | grep -qxF ".complete"; then
            found=yes
            break
        fi
        echo "waiting for ${SRC} to appear in listings (attempt ${attempt}/6)..."
        sleep 10
    done
    if [ -z "$found" ]; then
        echo "error: ${SRC} has no .complete marker after 6 checks." >&2
        echo "Either the artifact hit the auto-ingest bucket's 30-day expiry, or its" >&2
        echo "upload never finished. Re-run the Build workflow (workflow_dispatch)" >&2
        echo "to rebuild + re-upload; it re-copies and re-marks incomplete prefixes." >&2
        exit 1
    fi
else
    SRC="scholia:scholia-assets/${CORPUS}/derived"
fi

echo "Pulling ${CORPUS} structs from ${SRC}..."
rclone sync "$SRC" "/app/assets/${CORPUS}/derived" --fast-list --transfers=8

STRUCT_TO_DB_BIN=/usr/local/bin/struct_to_db bash /app/scripts/ingest.sh "$CORPUS"
