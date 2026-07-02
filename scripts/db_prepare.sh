#!/usr/bin/env bash
#
# Regenerate sqlx offline metadata into .sqlx/ at the repo root.
#
# Why: production builds (Dockerfile, GitHub Actions) compile the api
# crate with SQLX_OFFLINE=true so they don't need a live database. The
# sqlx::query! macros validate against the committed .sqlx/query-*.json
# files instead of connecting. This script regenerates those files
# against your local DB. Commit the resulting .sqlx/ diff alongside any
# Rust change that adds, removes, or alters a sqlx query.
#
# Prerequisites:
#   - Local DB up with the latest schema applied (run pnpm db:migrate first).
#   - sqlx-cli installed:
#       cargo install sqlx-cli --no-default-features --features postgres,rustls

set -euo pipefail

source "$(dirname "$0")/lib.sh"
DB_URL="${DATABASE_URL:-$SCHOLIA_DB_URL_DEFAULT}"
scholia_require_sqlx

DATABASE_URL="$DB_URL" cargo sqlx prepare --workspace
