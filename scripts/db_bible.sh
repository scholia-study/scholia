#!/usr/bin/env bash
# Import all five public-domain English Bibles from a local laptop
# checkout. KJV runs first because it's the canonical translation (its
# verse counts seed the parity guard) and DARBY's alignment seeder
# reads KJV's page_markers; the other four are independent and import
# in parallel.
#
# Local only
set -euo pipefail

cargo build -p bible_to_db --release

BIN=target/release/bible_to_db

# KJV first, sequential. It writes the canonical "The Bible" source
# row and the depth=0 toc_nodes the parity guard and DARBY alignment
# seeder both depend on.
"$BIN" --translation kjv

# The remaining four are independent: each writes its own book +
# sources + content tree. The only shared row is the canonical
# "The Bible" source which KJV created; the others SELECT it.
pids=()
"$BIN" --translation web   & pids+=($!)
"$BIN" --translation asv   & pids+=($!)
"$BIN" --translation bbe   & pids+=($!)
"$BIN" --translation darby & pids+=($!)

# wait-on-each so a failure in any background job propagates as a
# non-zero exit from the script. `wait` without args returns 0
# unconditionally, which would swallow failures.
status=0
for pid in "${pids[@]}"; do
    if ! wait "$pid"; then
        status=1
    fi
done
exit "$status"
