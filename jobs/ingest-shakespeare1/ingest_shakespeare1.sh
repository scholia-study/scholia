#!/usr/bin/env bash
#
# Job-image entrypoint for the Shakespeare ingest. Pulls the struct JSON
# from the scholia-assets bucket, then runs shakespeare1_struct_to_db
# once. With no flags the importer reconciles in place when the book
# already exists (preserving sentence UUIDs and the quotations/notes
# anchored to them) and fresh-inserts on first run.
#
# Self-contained on purpose.
set -euo pipefail

cd /app

echo "Pulling shakespeare1 assets from Hetzner Object Storage..."
export RCLONE_CONFIG_SCHOLIA_TYPE=s3
export RCLONE_CONFIG_SCHOLIA_PROVIDER=Other
export RCLONE_CONFIG_SCHOLIA_ENDPOINT=https://fsn1.your-objectstorage.com
export RCLONE_CONFIG_SCHOLIA_REGION=fsn1
export RCLONE_CONFIG_SCHOLIA_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID"
export RCLONE_CONFIG_SCHOLIA_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY"
export RCLONE_CONFIG_SCHOLIA_FORCE_PATH_STYLE=true

# Bucket paths mirror the local assets/ layout. The derived struct is
# gitignored, so it must be uploaded first: regenerate with
# `pnpm dp:shakespeare1:struct`, then `pnpm assets:sync`.
rclone sync scholia:scholia-assets/shakespeare1/derived /app/assets/shakespeare1/derived --fast-list --transfers=8

BIN=/usr/local/bin/shakespeare1_struct_to_db

"$BIN" --input-file assets/shakespeare1/derived/output.json
