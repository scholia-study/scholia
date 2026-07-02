#!/usr/bin/env bash

# Guarded by an explicit confirmation prompt — accidentally typing
# this instead of the local `db:reset` would otherwise be silent
# data loss on the dev cluster. Skip the prompt with `--yes` for
# automation:
#   scripts/db_dev_reset.sh --yes
#
# Requires, in another terminal:
#   just dev-forward
# This shell:
#   source ~/.config/scholia-infra.env

set -euo pipefail

cd "$(dirname "$0")/.."

if [[ "${1:-}" != "--yes" ]]; then
    echo "This will DROP the public schema on the dev cluster's Postgres"
    echo "(everything ingested into dev.scholia.study) and re-apply"
    echo "every migration in db/migrations/ from scratch."
    echo
    read -r -p "Type 'yes' to continue: " confirm
    if [[ "$confirm" != "yes" ]]; then
        echo "aborted." >&2
        exit 1
    fi
fi

exec bash scripts/db_dev_run.sh bash scripts/db_reset.sh
