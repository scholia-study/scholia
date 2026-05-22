#!/usr/bin/env bash
#
# Job-image entrypoint for the Kant Critique ingest. Pulls the two
# JSON outputs from the scholia-assets bucket, then runs the
# kant1_struct_to_db binary twice: source-language first (German B-
# edition), then translation linked back via --source-book-slug.
#
# Self-contained on purpose.
set -euo pipefail

cd /app

echo "Pulling kant1 assets from Hetzner Object Storage..."
export RCLONE_CONFIG_SCHOLIA_TYPE=s3
export RCLONE_CONFIG_SCHOLIA_PROVIDER=Other
export RCLONE_CONFIG_SCHOLIA_ENDPOINT=https://fsn1.your-objectstorage.com
export RCLONE_CONFIG_SCHOLIA_REGION=fsn1
export RCLONE_CONFIG_SCHOLIA_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID"
export RCLONE_CONFIG_SCHOLIA_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY"
export RCLONE_CONFIG_SCHOLIA_FORCE_PATH_STYLE=true

# Bucket paths mirror the local assets/ layout. To re-upload after the
# kant1 raw/curated/derived reorg:
#   rclone moveto scholia:scholia-assets/kant1_md_to_struct scholia:scholia-assets/kant1/derived/md_to_struct
#   rclone moveto scholia:scholia-assets/kant1_md_translation_to_struct scholia:scholia-assets/kant1/derived/md_translation_to_struct
rclone sync scholia:scholia-assets/kant1/derived/md_to_struct /app/assets/kant1/derived/md_to_struct --fast-list --transfers=8
rclone sync scholia:scholia-assets/kant1/derived/md_translation_to_struct /app/assets/kant1/derived/md_translation_to_struct --fast-list --transfers=8

BIN=/usr/local/bin/kant1_struct_to_db

"$BIN" --input-file assets/kant1/derived/md_to_struct/output.json
"$BIN" --input-file assets/kant1/derived/md_translation_to_struct/output.json \
       --source-book-slug kritik-der-reinen-vernunft-b
