#!/usr/bin/env bash
#
# Mirror local `assets/` to the `scholia-assets` Hetzner Object Storage
# bucket. Local is canonical; this script makes the bucket match.
# Re-running after edits transfers only the changed files (and deletes
# remote files that no longer exist locally).
#
# Pass-through args go to rclone — useful for `--dry-run`:
#   pnpm assets:sync --dry-run
#
# Requires:
#   - rclone on PATH (`sudo apt install -y rclone`)
#   - AWS_ACCESS_KEY_ID + AWS_SECRET_ACCESS_KEY in env (Hetzner S3
#     credentials, same pair used by Terraform)
#
# Source the env file first:
#   source ~/.config/scholia-infra.env

set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v rclone >/dev/null 2>&1; then
    echo "error: rclone not on PATH." >&2
    echo "install with: sudo apt install -y rclone" >&2
    exit 1
fi

: "${AWS_ACCESS_KEY_ID:?Set AWS_ACCESS_KEY_ID (source ~/.config/scholia-infra.env)}"
: "${AWS_SECRET_ACCESS_KEY:?Set AWS_SECRET_ACCESS_KEY (source ~/.config/scholia-infra.env)}"

# Configure the `scholia` rclone remote inline via env vars — no
# rclone.conf file involved. Lets the script run on any machine that
# has rclone + the creds, no first-time `rclone config` ritual.
export RCLONE_CONFIG_SCHOLIA_TYPE=s3
export RCLONE_CONFIG_SCHOLIA_PROVIDER=Other
export RCLONE_CONFIG_SCHOLIA_ENDPOINT=https://fsn1.your-objectstorage.com
export RCLONE_CONFIG_SCHOLIA_REGION=fsn1
export RCLONE_CONFIG_SCHOLIA_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID"
export RCLONE_CONFIG_SCHOLIA_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY"
export RCLONE_CONFIG_SCHOLIA_FORCE_PATH_STYLE=true

echo "→ Mirroring ./assets/ → scholia:scholia-assets/ ..."
exec rclone sync ./assets/ scholia:scholia-assets/ \
    --progress \
    --fast-list \
    --transfers=8 \
    "$@"
