#!/usr/bin/env bash
# Import KJV first (canonical translation — its verse counts seed the
# parity guard) then WEB. Re-run safe only against a fresh schema:
# pair with db_reset.sh + db_kant1.sh as needed.
set -euo pipefail

cargo run -p bible_to_db -- --translation kjv
cargo run -p bible_to_db -- --translation web
