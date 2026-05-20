#!/usr/bin/env bash
#
# Job-image entrypoint for the Bible ingest. Pulls the bible asset
# subset from the scholia-assets bucket into /app/assets/bible/, then
# runs the bible_to_db binary across all five translations. KJV first
# (canonical — its verse counts seed the parity guard, and DARBY's
# alignment seeder reads KJV's page_markers); the other four are
# independent and run in parallel.
#
# Self-contained on purpose.
set -euo pipefail

cd /app

echo "Pulling bible assets from Hetzner Object Storage..."
export RCLONE_CONFIG_SCHOLIA_TYPE=s3
export RCLONE_CONFIG_SCHOLIA_PROVIDER=Other
export RCLONE_CONFIG_SCHOLIA_ENDPOINT=https://fsn1.your-objectstorage.com
export RCLONE_CONFIG_SCHOLIA_REGION=fsn1
export RCLONE_CONFIG_SCHOLIA_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID"
export RCLONE_CONFIG_SCHOLIA_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY"
export RCLONE_CONFIG_SCHOLIA_FORCE_PATH_STYLE=true

rclone sync scholia:scholia-assets/bible /app/assets/bible --fast-list --transfers=8

BIN=/usr/local/bin/bible_to_db

# KJV first, sequential.
"$BIN" --translation kjv

# The remaining four are independent.
pids=()
"$BIN" --translation web   & pids+=($!)
"$BIN" --translation asv   & pids+=($!)
"$BIN" --translation bbe   & pids+=($!)
"$BIN" --translation darby & pids+=($!)

# wait-on-each so a failure in any background job propagates as a
# non-zero exit. `wait` without args returns 0 unconditionally, which
# would swallow failures.
status=0
for pid in "${pids[@]}"; do
    if ! wait "$pid"; then
        status=1
    fi
done
exit "$status"
