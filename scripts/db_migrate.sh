#!/usr/bin/env bash
#
# Apply pending sqlx migrations to the local Postgres WITHOUT wiping data.
#
# Prerequisite: `cargo install sqlx-cli --no-default-features --features postgres,rustls`

set -euo pipefail

DB_URL="${DATABASE_URL:-postgres://prospero:prospero@localhost:5433/prospero}"

if ! command -v sqlx >/dev/null 2>&1; then
    echo "error: sqlx-cli not on PATH." >&2
    echo "install with: cargo install sqlx-cli --no-default-features --features postgres,rustls" >&2
    exit 1
fi

echo "Migration status before:"
DATABASE_URL="$DB_URL" sqlx migrate info --source db/migrations

echo
DATABASE_URL="$DB_URL" sqlx migrate run --source db/migrations
