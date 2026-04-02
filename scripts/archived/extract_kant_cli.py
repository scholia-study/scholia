#!/usr/bin/env python3
"""Extract structured text from Kant's Kritik der reinen Vernunft using Claude Code CLI.

Uses `claude -p` (non-interactive mode) so it runs on your Max plan — no API key needed.
"""

import argparse
import json
import os
import subprocess
import sys
import time
from glob import glob

EXTRACTION_PROMPT = """\
You are extracting text from a scanned page of Kant's *Kritik der reinen Vernunft* (2nd edition, 1787) in the 1911 Akademie-Ausgabe. The text is in Fraktur with some Antiqua passages (Latin/technical terms). Transcribe accurately.

Return a JSON object (no markdown fences, just raw JSON) with this structure:

{
  "page_index": <1-based index of the page in the PDF>,
  "page_number": "<printed page number as it appears, e.g. 'V', '27', or null if none>",
  "page_type": "<one of: title, toc, preface, body, blank, other>",
  "elements": [
    {
      "type": "<heading | paragraph | toc_entry | signature | dedication | epigraph | other>",
      "text": "<transcribed text, preserving line breaks within the element as newlines>",
      "emphasis": [{"start": <char offset>, "end": <char offset>, "kind": "<sperrdruck | bold | antiqua>"}],
      "typeface": "<fraktur | antiqua | mixed>",
      "line_numbers": [<Akademie line numbers visible in the margin that span this element, as integers>],
      "footnote_markers": ["<marker characters like 1, 2, *, † etc.>"],
      "b_page_ref": "<B-edition page number if a page marker appears at or within this element, else null>"
    }
  ],
  "footnotes": [
    {"marker": "<marker>", "text": "<footnote text>"}
  ]
}

Rules:
- Transcribe Fraktur faithfully. Use modern Unicode (ä, ö, ü, ß). Keep original spelling (e.g. "Theil" not "Teil").
- For Sperrdruck (letterspaced text used for emphasis), transcribe the text normally (without extra spaces) and record it in the "emphasis" array with character offsets into the "text" field.
- For text set in Antiqua (roman) typeface amidst Fraktur, note it as emphasis kind "antiqua" or set the element typeface to "antiqua"/"mixed".
- "line_numbers": include all Akademie margin line numbers that fall within this element's span. These are the small numbers printed in the left or right margin (typically multiples of 5).
- "b_page_ref": the B-edition page number marker. These appear as bold or large numbers in the margin or inline. Record per-element where the marker appears.
- "footnote_markers": list any superscript markers (numbers, *, †) that appear in this element's text.
- "footnotes": transcribe all footnotes at the bottom of the page, with their markers.
- For table-of-contents pages, use type "toc_entry" for each line/entry.
- Preserve document order in the elements array.
- If the page is blank or has only a page number, return page_type "blank" with empty elements.
- Do NOT wrap the JSON in markdown code fences. Return ONLY the JSON object."""


def extract_page(image_path: str, page_index: int) -> dict:
    """Call claude CLI to extract structured text from a page image."""
    abs_path = os.path.abspath(image_path)
    prompt = (
        f"Read the image file at {abs_path} using the Read tool. "
        f"This is page index {page_index} of Kant's Kritik der reinen Vernunft (1911 Akademie-Ausgabe).\n\n"
        f"{EXTRACTION_PROMPT}"
    )

    env = {k: v for k, v in os.environ.items() if k != "CLAUDECODE"}
    result = subprocess.run(
        [
            "claude", "-p", prompt,
            "--output-format", "json",
            "--allowed-tools", "Read",
            "--no-session-persistence",
            "--model", "sonnet",
        ],
        capture_output=True,
        text=True,
        timeout=180,
        env=env,
    )

    if result.returncode != 0:
        raise RuntimeError(f"claude CLI failed (exit {result.returncode}): {result.stderr[:500]}")

    # Parse the outer JSON envelope from claude --output-format json
    outer = json.loads(result.stdout)
    response_text = outer.get("result", "")

    # Strip markdown fences if present
    text = response_text.strip()
    if text.startswith("```"):
        first_newline = text.index("\n")
        text = text[first_newline + 1:]
        if text.rstrip().endswith("```"):
            text = text.rstrip()[:-3]
        text = text.strip()

    return json.loads(text)


def find_page_images(pages_dir: str) -> list[str]:
    patterns = ["*.png", "*.jpg", "*.jpeg"]
    files = []
    for p in patterns:
        files.extend(glob(os.path.join(pages_dir, p)))
    return sorted(files)


def main():
    parser = argparse.ArgumentParser(description="Extract text from Kant PDF pages using Claude Code CLI")
    parser.add_argument("--pages-dir", default="assets/kant1_pages", help="Directory containing page images")
    parser.add_argument("--output-dir", default="assets/kant1_output", help="Directory for per-page JSON output")
    parser.add_argument("--output", default="assets/kant1_kritik.json", help="Final merged JSON output path")
    parser.add_argument("--start", type=int, default=1, help="Start page index (1-based)")
    parser.add_argument("--end", type=int, default=None, help="End page index (1-based, inclusive)")
    parser.add_argument("--merge-only", action="store_true", help="Only merge existing page JSONs, skip extraction")
    parser.add_argument("--concurrency", type=int, default=1, help="Number of parallel extractions (be careful with rate limits)")
    args = parser.parse_args()

    os.makedirs(args.output_dir, exist_ok=True)

    images = find_page_images(args.pages_dir)
    if not images and not args.merge_only:
        print(f"No page images found in {args.pages_dir}/")
        print("Run: pdftoppm -png -r 300 assets/kant_kritik_2ed_1911.pdf assets/kant1_pages/page")
        sys.exit(1)

    if not args.merge_only:
        total = len(images)
        start_idx = args.start - 1
        end_idx = args.end if args.end else total

        print(f"Found {total} page images. Processing pages {args.start}–{end_idx}.")

        for i in range(start_idx, min(end_idx, total)):
            page_num = i + 1
            out_path = os.path.join(args.output_dir, f"page_{page_num:04d}.json")

            if os.path.exists(out_path):
                print(f"  [{page_num}/{end_idx}] Skipping (already exists)")
                continue

            print(f"  [{page_num}/{end_idx}] Extracting {os.path.basename(images[i])}...", end=" ", flush=True)

            retries = 0
            result = None
            while retries <= 3:
                try:
                    result = extract_page(images[i], page_num)
                    break
                except subprocess.TimeoutExpired:
                    retries += 1
                    print(f"timeout, retry {retries}/3...", end=" ", flush=True)
                except (RuntimeError, json.JSONDecodeError) as e:
                    retries += 1
                    print(f"error ({e}), retry {retries}/3...", end=" ", flush=True)
                    time.sleep(2)

            if result is None:
                print("FAILED")
                continue

            with open(out_path, "w", encoding="utf-8") as f:
                json.dump(result, f, ensure_ascii=False, indent=2)
            print("done")

    # Merge step
    page_files = sorted(glob(os.path.join(args.output_dir, "page_*.json")))
    if not page_files:
        print("No page JSON files to merge.")
        return

    pages = []
    for pf in page_files:
        with open(pf, encoding="utf-8") as f:
            pages.append(json.load(f))

    with open(args.output, "w", encoding="utf-8") as f:
        json.dump({"pages": pages}, f, ensure_ascii=False, indent=2)

    print(f"Merged {len(pages)} pages into {args.output}")


if __name__ == "__main__":
    main()
