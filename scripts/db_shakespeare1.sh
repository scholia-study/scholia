#!/usr/bin/env bash
# Import Shakespeare's Sonnets into the local Postgres.
#
# Imports the pre-built struct; run `pnpm dp:shakespeare1:struct` first to
# (re)generate it from the curated MD. Does NOT fetch (curated MD is the source
# of truth) and does NOT reset anything.
#
# Pass importer flags through, e.g. `pnpm db:shakespeare1 -- --dry-run`
# (insert + rollback) or `-- --replace` (delete this one book, then re-insert).
#
# Local only
set -euo pipefail

cargo build -p struct_to_db --release
target/release/struct_to_db \
    --input-file assets/shakespeare1/derived/output.json "$@"
