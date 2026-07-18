#!/usr/bin/env bash
#
# Print the content hash of one corpus's derived struct JSON(s) — the
# auto-ingest "deploy tag". THE hash definition: CI's structs job keys
# bucket uploads (scholia-assets-auto/<corpus>/derived@<hash>) and ingest
# Job names (ingest-<corpus>-<hash>) with it, so it must be computed the
# same way everywhere. Covers file bytes AND repo-relative paths (a rename
# is a content change); first 12 hex chars of a sha256-of-sha256sums.
#
# Run from the repo root, after `just struct <corpus>`.
#
# Usage: derived_hash.sh <corpus>
set -euo pipefail

source "$(dirname "$0")/lib.sh"

corpus="${1:?usage: derived_hash.sh <corpus>}"

if ! scholia_corpus_known "$corpus"; then
    echo "error: unknown corpus '$corpus' (expected: ${SCHOLIA_CORPORA[*]})" >&2
    exit 1
fi

dir="assets/$corpus/derived"
files=$(find "$dir" -name '*.json' | sort)
if [ -z "$files" ]; then
    echo "error: no derived JSON under $dir — run 'just struct $corpus' first" >&2
    exit 1
fi

echo "$files" | xargs sha256sum | sha256sum | cut -c1-12
