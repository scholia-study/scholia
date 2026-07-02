#!/usr/bin/env bash
#
# Apply pending sqlx migrations to the local Postgres WITHOUT wiping data.
#
# Prerequisite: `cargo install sqlx-cli --no-default-features --features postgres,rustls`

set -euo pipefail

source "$(dirname "$0")/lib.sh"
DB_URL="${DATABASE_URL:-$SCHOLIA_DB_URL_DEFAULT}"
scholia_require_sqlx

echo "Migration status before:"
DATABASE_URL="$DB_URL" sqlx migrate info --source db/migrations

echo
DATABASE_URL="$DB_URL" sqlx migrate run --source db/migrations
