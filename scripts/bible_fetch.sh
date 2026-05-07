#!/usr/bin/env bash
# Fetch all 66 canonical books for KJV and WEB from bible-api.com into
# assets/bible/<translation>/<book>/<chapter>.json. Both translations are
# public domain. Slugs match bible-api.com's normalized book identifiers
# (no spaces/hyphens) so they double as URL fragments and filesystem dirs.
# Existing files are skipped — safe to re-run after a partial fetch.
set -euo pipefail

cd "$(dirname "$0")/.."

# Reading order matters; bash assoc arrays don't preserve insertion order
# so we keep a parallel ordered list. Chapter counts are the canonical
# Protestant counts (KJV/WEB agree).
BOOKS_ORDERED=(
    # Old Testament
    "genesis:50" "exodus:40" "leviticus:27" "numbers:36" "deuteronomy:34"
    "joshua:24" "judges:21" "ruth:4"
    "1samuel:31" "2samuel:24" "1kings:22" "2kings:25"
    "1chronicles:29" "2chronicles:36"
    "ezra:10" "nehemiah:13" "esther:10"
    "job:42" "psalms:150" "proverbs:31" "ecclesiastes:12" "songofsolomon:8"
    "isaiah:66" "jeremiah:52" "lamentations:5" "ezekiel:48" "daniel:12"
    "hosea:14" "joel:3" "amos:9" "obadiah:1" "jonah:4" "micah:7"
    "nahum:3" "habakkuk:3" "zephaniah:3" "haggai:2" "zechariah:14" "malachi:4"
    # New Testament
    "matthew:28" "mark:16" "luke:24" "john:21" "acts:28"
    "romans:16" "1corinthians:16" "2corinthians:13"
    "galatians:6" "ephesians:6" "philippians:4" "colossians:4"
    "1thessalonians:5" "2thessalonians:3" "1timothy:6" "2timothy:4"
    "titus:3" "philemon:1" "hebrews:13" "james:5"
    "1peter:5" "2peter:3" "1john:5" "2john:1" "3john:1" "jude:1"
    "revelation:22"
)

for translation in kjv web asv bbe darby; do
    for entry in "${BOOKS_ORDERED[@]}"; do
        book="${entry%%:*}"
        chapters="${entry##*:}"
        out_dir="assets/bible/${translation}/${book}"
        mkdir -p "$out_dir"
        for ((c=1; c<=chapters; c++)); do
            out="${out_dir}/${c}.json"
            if [[ -s "$out" ]]; then
                continue
            fi
            url="https://bible-api.com/${book}%20${c}?translation=${translation}"
            echo "fetch ${translation} ${book} ${c}"
            success=0
            for attempt in 1 2 3 4 5 6 7 8; do
                if curl -sfL --max-time 30 "$url" -o "$out"; then
                    success=1
                    break
                fi
                rm -f "$out"
                # Exponential-ish backoff: 2, 4, 6, 8, 15, 30, 60, 120s
                case $attempt in
                    1|2|3|4) sleep $((attempt * 2)) ;;
                    5) sleep 15 ;;
                    6) sleep 30 ;;
                    7) sleep 60 ;;
                    8) sleep 120 ;;
                esac
            done
            if [[ $success -eq 0 ]]; then
                # Don't abort the whole run; log to a per-run failure file
                # and keep going. Re-running the script picks up missing
                # chapters via the existing-file guard above.
                echo "FAILED $url" >> /tmp/bible_fetch_failures.log
                fail_count=$((${fail_count:-0} + 1))
                continue
            fi
            sleep 0.3
        done
    done
done

if [[ -n "${fail_count:-}" && "$fail_count" -gt 0 ]]; then
    echo "done — $fail_count chapter(s) failed; re-run to retry. See /tmp/bible_fetch_failures.log"
    exit 2
fi
echo "done"
