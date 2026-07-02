#!/usr/bin/env bash
#
# Job-image entrypoint for the Bible ingest. Pulls the bible asset
# subset from the scholia-assets bucket into /app/assets/bible/, then
# runs the shared import logic (scripts/bible_import.sh: KJV-first
# canonical, four parallel) with the baked-in bible_to_db binary.
set -euo pipefail

cd /app

echo "Pulling bible assets from Hetzner Object Storage..."
source /app/scripts/lib.sh
scholia_rclone_config

rclone sync scholia:scholia-assets/bible /app/assets/bible --fast-list --transfers=8

BIBLE_TO_DB_BIN=/usr/local/bin/bible_to_db exec bash /app/scripts/bible_import.sh
