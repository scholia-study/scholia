#!/usr/bin/env bash
#
# Regenerate one corpus's derived struct JSON(s) from its curated markdown —
# the parser-side sibling of ingest.sh. Which genre parser a corpus uses (and
# whether it has a translation edition) is per-corpus knowledge, so it lives
# here as one case arm each.
#
# Usage: struct.sh <corpus> | struct.sh --list
set -euo pipefail

source "$(dirname "$0")/lib.sh"

corpus="${1:?usage: struct.sh <corpus> | struct.sh --list}"

if [ "$corpus" = "--list" ]; then
    printf '%s\n' "${SCHOLIA_CORPORA[@]}"
    exit 0
fi

if ! scholia_corpus_known "$corpus"; then
    echo "error: unknown corpus '$corpus' (expected: ${SCHOLIA_CORPORA[*]})" >&2
    exit 1
fi

case "$corpus" in
    kant1 | kant3)
        cargo run -p md_prose_to_struct -- --corpus "$corpus"
        cargo run -p md_prose_to_struct -- --corpus "$corpus" --translation
        ;;
    shakespeare1 | milton1)
        cargo run -p md_poetry_to_struct -- --corpus "$corpus"
        ;;
    ibsen1)
        cargo run -p md_drama_to_struct -- --corpus "$corpus"
        cargo run -p md_drama_to_struct -- --corpus "$corpus" --translation
        ;;
    *)
        # Unreachable: membership in SCHOLIA_CORPORA is checked above. Hitting
        # this means the array and the case arms drifted apart.
        echo "error: corpus '$corpus' is in SCHOLIA_CORPORA but has no case arm in struct.sh" >&2
        exit 1
        ;;
esac
