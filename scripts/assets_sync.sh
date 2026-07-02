#!/usr/bin/env bash
#
# Mirror local `assets/` to the `scholia-assets` Hetzner Object Storage
# bucket. Local is canonical; this script makes the bucket match.
# Re-running after edits transfers only the changed files (and deletes
# remote files that no longer exist locally).
#
# Pass-through args go to rclone — useful for `--dry-run`:
#   just assets-sync --dry-run
#
# Requires:
#   - rclone on PATH (`sudo apt install -y rclone`)
#   - AWS_ACCESS_KEY_ID + AWS_SECRET_ACCESS_KEY in env (Hetzner S3
#     credentials, same pair used by Terraform)
#
# Source the env file first:
#   source ~/.config/scholia-infra.env

set -euo pipefail

# Resolve lib.sh before cd-ing: $0 may be relative to the invocation dir.
source "$(dirname "$0")/lib.sh"
cd "$(dirname "$0")/.."

if ! command -v rclone >/dev/null 2>&1; then
    echo "error: rclone not on PATH." >&2
    echo "install with: sudo apt install -y rclone" >&2
    exit 1
fi

scholia_rclone_config

echo "→ Mirroring ./assets/ → scholia:scholia-assets/ ..."
exec rclone sync ./assets/ scholia:scholia-assets/ \
    --progress \
    --fast-list \
    --transfers=8 \
    "$@"
