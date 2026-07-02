#!/usr/bin/env bash
#
# Import one corpus's struct JSON(s) into Postgres — THE per-corpus ingest
# manifest. Every struct-importer corpus is one case below; local dev
# (`just db <corpus>`) and the ingest job image both run this script, so a
# corpus can never exist locally but be missing from the deploy path.
#
# Usage: ingest.sh <corpus> [importer flags...]
#   e.g. ingest.sh kant1 --dry-run
#        ingest.sh ibsen1 --database-url postgres://…/scratch
#
# Source-language editions import first: a translation import looks the source
# book up by slug to thread sentence + node alignments.
#
# The job image sets STRUCT_TO_DB_BIN (binary baked in); locally the importer
# is cargo-built. Bible is NOT here — it has its own importer + fetch flow
# (scripts/db_bible.sh).
#
# Run from the repo root (or /app in the job image). Requires structs to be
# built first: `just struct <corpus>` (locally) or the bucket sync (job).
set -euo pipefail

source "$(dirname "$0")/lib.sh"

corpus="${1:?usage: ingest.sh <corpus> [importer flags...] | ingest.sh --list}"
shift

# The corpus roster, one per line — `just db-reload` iterates this.
if [ "$corpus" = "--list" ]; then
    printf '%s\n' "${SCHOLIA_CORPORA[@]}"
    exit 0
fi

if ! scholia_corpus_known "$corpus"; then
    echo "error: unknown corpus '$corpus' (expected: ${SCHOLIA_CORPORA[*]})" >&2
    exit 1
fi

# A `--` separator may arrive literally (pnpm-style forwarding); drop a
# leading one so flags reach the importer.
if [ "${1:-}" = "--" ]; then shift; fi

if [ -n "${STRUCT_TO_DB_BIN:-}" ]; then
    BIN="$STRUCT_TO_DB_BIN"
else
    cargo build -p struct_to_db --release
    BIN=target/release/struct_to_db
fi

case "$corpus" in
    kant1)
        "$BIN" --input-file assets/kant1/derived/output.json "$@"
        "$BIN" --input-file assets/kant1/derived/translation_output.json \
               --source-book-slug kritik-der-reinen-vernunft-b "$@"
        ;;
    kant3)
        "$BIN" --input-file assets/kant3/derived/output.json "$@"
        "$BIN" --input-file assets/kant3/derived/translation_output.json \
               --source-book-slug kritik-der-urteilskraft "$@"
        ;;
    shakespeare1)
        "$BIN" --input-file assets/shakespeare1/derived/output.json "$@"
        ;;
    milton1)
        "$BIN" --input-file assets/milton1/derived/output.json "$@"
        ;;
    ibsen1)
        "$BIN" --input-file assets/ibsen1/derived/output.json "$@"
        "$BIN" --input-file assets/ibsen1/derived/translation_output.json \
               --source-book-slug keiser-og-galileer "$@"
        ;;
    *)
        # Unreachable: membership in SCHOLIA_CORPORA is checked above. Hitting
        # this means the array and the case arms drifted apart.
        echo "error: corpus '$corpus' is in SCHOLIA_CORPORA but has no case arm in ingest.sh" >&2
        exit 1
        ;;
esac
