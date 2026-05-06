#!/usr/bin/env bash
# Fetch Genesis (50 chapters) and John (21 chapters) for KJV and WEB
# from bible-api.com into assets/bible/<translation>/<book>/<chapter>.json.
# Both translations are public domain.
set -euo pipefail

cd "$(dirname "$0")/.."

declare -A BOOK_CHAPTERS=(
    [genesis]=50
    [john]=21
)

for translation in kjv web; do
    for book in "${!BOOK_CHAPTERS[@]}"; do
        chapters=${BOOK_CHAPTERS[$book]}
        out_dir="assets/bible/${translation}/${book}"
        mkdir -p "$out_dir"
        for ((c=1; c<=chapters; c++)); do
            out="${out_dir}/${c}.json"
            if [[ -s "$out" ]]; then
                continue
            fi
            url="https://bible-api.com/${book}%20${c}?translation=${translation}"
            echo "fetch ${translation} ${book} ${c}"
            for attempt in 1 2 3 4 5; do
                if curl -sfL --max-time 30 "$url" -o "$out"; then
                    break
                fi
                rm -f "$out"
                if [[ $attempt -eq 5 ]]; then
                    echo "FAILED $url after $attempt attempts"
                    exit 1
                fi
                sleep $((attempt * 2))
            done
            sleep 0.2
        done
    done
done

echo "done"
