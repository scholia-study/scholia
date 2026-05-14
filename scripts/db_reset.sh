#!/usr/bin/env bash
#
# Wipe the local Postgres schema and re-apply all sqlx migrations.
#
# Two-step: (1) drop the `public` schema so the database is empty,
# including the `_sqlx_migrations` ledger that tracks applied migrations.
# (2) run `sqlx migrate run` against `db/migrations/` to re-apply
# everything from scratch.
#
# We use sqlx-cli rather than `cargo run -p api -- migrate` because the
# api crate's sqlx::query! macros verify against a live schema at
# compile time — dropping the schema and then asking cargo to build
# the binary creates a chicken-and-egg. sqlx-cli is self-contained and
# sidesteps that. In production, the cluster init container uses
# `api migrate` (subcommand on the main binary) since the prod build
# uses committed `.sqlx` offline metadata.
#
# Prerequisite: `cargo install sqlx-cli --no-default-features --features postgres,rustls`

set -euo pipefail

DB_URL="${DATABASE_URL:-postgres://prospero:prospero@localhost:5433/prospero}"

if ! command -v sqlx >/dev/null 2>&1; then
    echo "error: sqlx-cli not on PATH." >&2
    echo "install with: cargo install sqlx-cli --no-default-features --features postgres,rustls" >&2
    exit 1
fi

psql "$DB_URL" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"

DATABASE_URL="$DB_URL" sqlx migrate run --source db/migrations
