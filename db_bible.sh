#!/usr/bin/env bash
# Import all five public-domain English Bibles. KJV runs first because
# it's the canonical translation (its verse counts seed the parity
# guard); the rest follow in publication-year order. Re-run safe only
# against a fresh schema: pair with db_reset.sh + db_kant1.sh as needed.
set -euo pipefail

for t in kjv web asv bbe darby; do
    cargo run -p bible_to_db -- --translation "$t"
done
