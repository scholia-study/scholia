#!/usr/bin/env bash
#
# Job-image entrypoint for every struct-importer corpus. `CORPUS` (env)
# selects which one: kant1 | kant3 | shakespeare1 | milton1 | ibsen1.
# Pulls the corpus's derived struct JSON(s) from the scholia-assets bucket,
# then runs the shared per-corpus manifest (scripts/ingest.sh) with the
# baked-in struct_to_db binary. With no flags the importer reconciles in
# place when the book already exists (preserving sentence UUIDs and the
# quotations/notes anchored to them) and fresh-inserts on first run.
#
# Bucket paths mirror the local assets/ layout. The derived structs are
# gitignored, so they must be uploaded first: regenerate with
# `just struct <corpus>`, then `just assets-sync`.
set -euo pipefail

: "${CORPUS:?Set CORPUS to one of: kant1 | kant3 | shakespeare1 | milton1 | ibsen1}"

cd /app
source /app/scripts/lib.sh

echo "Pulling ${CORPUS} assets from Hetzner Object Storage..."
scholia_rclone_config
rclone sync "scholia:scholia-assets/${CORPUS}/derived" "/app/assets/${CORPUS}/derived" --fast-list --transfers=8

STRUCT_TO_DB_BIN=/usr/local/bin/struct_to_db bash /app/scripts/ingest.sh "$CORPUS"
