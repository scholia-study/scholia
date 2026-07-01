#!/usr/bin/env bash
# Import Ibsen's Emperor and Galilean
# 
# Local only
set -euo pipefail

# pnpm forwards the `--` separator literally (e.g. `pnpm db:ibsen1 -- --replace`
# arrives here as `-- --replace`); drop a leading one so flags reach the importer.
if [ "${1:-}" = "--" ]; then shift; fi

cargo build -p struct_to_db --release
BIN=target/release/struct_to_db

"$BIN" --input-file assets/ibsen1/derived/output.json "$@"
"$BIN" --input-file assets/ibsen1/derived/translation_output.json \
       --source-book-slug keiser-og-galileer "$@"
