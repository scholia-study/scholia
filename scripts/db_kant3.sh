#!/usr/bin/env bash
# Import Kant's Kritik der Urteilskraft (German, Akademie-Ausgabe Band V) and
# its English translation. The German source language goes first because the
# translation import looks the source book up by slug to thread sentence + node
# alignments.
#
# Imports the pre-built structs; run `pnpm struct:kant3` first to (re)generate
# them from the curated MD. Pass importer flags through, e.g.
# `pnpm db:kant3 -- --database-url postgres://…/scratch` to target a throwaway DB.
#
# Local only
set -euo pipefail

cargo build -p kant3_struct_to_db --release
BIN=target/release/kant3_struct_to_db

"$BIN" --input-file assets/kant3/derived/md_to_struct/output.json "$@"
"$BIN" --input-file assets/kant3/derived/md_translation_to_struct/output.json \
       --source-book-slug kritik-der-urteilskraft "$@"
