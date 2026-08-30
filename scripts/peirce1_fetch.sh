#!/usr/bin/env bash
#
# Fetch peirce1's Wikisource proofread transcriptions into assets/peirce1/raw/
# (gitignored). One file per scanned page, so the converter can read each
# page's running head for its printed page number.
#
# The page list comes from the converter itself (`--fetch-plan`), so the source
# table lives in exactly one place: packages/peirce1_wikisource_to_md/papers.rs.
#
#   bash scripts/peirce1_fetch.sh            # all papers
#   bash scripts/peirce1_fetch.sh 110        # one paper, by position
#
# Then: cargo run -p peirce1_wikisource_to_md
set -euo pipefail

UA="scholia-ingest/0.1 (+https://scholia.study)"
API="https://en.wikisource.org/w/index.php"

plan_args=(--fetch-plan)
if [ $# -gt 0 ]; then plan_args+=(--position "$1"); fi

fetched=0
skipped=0
while IFS=$'\t' read -r index page dest; do
    if [ -s "$dest" ]; then
        skipped=$((skipped + 1))
        continue
    fi
    mkdir -p "$(dirname "$dest")"
    title="Page:${index// /_}/${page}"
    curl -sfL --max-time 30 --retry 3 --retry-delay 2 -A "$UA" \
        --get "$API" --data-urlencode "title=$title" --data-urlencode "action=raw" \
        -o "$dest"
    fetched=$((fetched + 1))
    # Courtesy rate limit — this is someone else's volunteer-run server.
    sleep 0.4
done < <(cargo run -q -p peirce1_wikisource_to_md -- "${plan_args[@]}")

echo "fetched $fetched page(s), $skipped already present" >&2
