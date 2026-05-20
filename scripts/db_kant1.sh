#!/usr/bin/env bash
# Import the Kritik der reinen Vernunft B-edition (German) and its
# English translation. Source language goes first because the
# translation import looks the source book up by slug to thread
# sentence + node alignments.
#
# Local only
set -euo pipefail

cargo build -p kant1_struct_to_db --release
BIN=target/release/kant1_struct_to_db

"$BIN" --input-file assets/kant1_md_to_struct/output.json
"$BIN" --input-file assets/kant1_md_translation_to_struct/output.json \
       --source-book-slug kritik-der-reinen-vernunft-b
