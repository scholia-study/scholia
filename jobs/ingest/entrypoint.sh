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

scholia_rclone_config
if [ -n "${DERIVED_HASH:-}" ]; then
    SRC="scholia:scholia-assets-auto/${CORPUS}/derived@${DERIVED_HASH}"
    if ! rclone lsf "$SRC" --max-depth 1 | grep -q .; then
        echo "error: ${SRC} is missing or empty." >&2
        echo "The artifact likely hit the auto-ingest bucket's 30-day expiry;" >&2
        echo "re-run the Build workflow (workflow_dispatch) to rebuild + re-upload." >&2
        exit 1
    fi
else
    SRC="scholia:scholia-assets/${CORPUS}/derived"
fi

echo "Pulling ${CORPUS} structs from ${SRC}..."
rclone sync "$SRC" "/app/assets/${CORPUS}/derived" --fast-list --transfers=8

STRUCT_TO_DB_BIN=/usr/local/bin/struct_to_db bash /app/scripts/ingest.sh "$CORPUS"
