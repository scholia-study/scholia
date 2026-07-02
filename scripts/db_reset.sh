#!/usr/bin/env bash
#
# Wipe the local Postgres schema and re-apply all sqlx migrations.
#
# ! DO NOT RUN UNLESS PERMISSION IS GIVEN EXPLICITLY
#
# Prerequisite: `cargo install sqlx-cli --no-default-features --features postgres,rustls`

set -euo pipefail

source "$(dirname "$0")/lib.sh"
DB_URL="${DATABASE_URL:-$SCHOLIA_DB_URL_DEFAULT}"
scholia_require_sqlx

psql "$DB_URL" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"

DATABASE_URL="$DB_URL" sqlx migrate run --source db/migrations
