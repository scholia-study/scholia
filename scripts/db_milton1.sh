#!/usr/bin/env bash
# Import Milton's Paradise Lost into the local Postgres.
#
# Imports the pre-built struct; run `pnpm struct:milton1` first to (re)generate
# it from the curated MD. Does NOT fetch (curated MD is the source of truth) and
# does NOT reset anything.
#
# Pass importer flags through, e.g. `pnpm db:milton1 -- --dry-run`
# (insert + rollback) or `-- --replace` (delete this one book, then re-insert).
#
# Local only
set -euo pipefail

cargo build -p poetry_struct_to_db --release
target/release/poetry_struct_to_db \
    --input-file assets/milton1/derived/output.json "$@"
