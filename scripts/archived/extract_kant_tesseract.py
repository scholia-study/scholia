#!/usr/bin/env python3
"""Extract structured text from Kant's Kritik der reinen Vernunft using Tesseract OCR.

Zero external dependencies — uses only Python stdlib + tesseract CLI.
Runs entirely offline. Processes all 561 pages in minutes.

Tradeoff vs Claude Vision: emphasis always [], typeface always "fraktur",
~90-95% character accuracy. Structure (line numbers, page numbers, footnotes,
B-edition refs, paragraph segmentation) preserved via spatial heuristics.
"""

import argparse
import csv
import json
import os
import re
import subprocess
import sys
from glob import glob
from io import StringIO

# ---------------------------------------------------------------------------
# Image dimensions (1575 × 2434 px at 300 DPI)
# ---------------------------------------------------------------------------
IMG_W = 1575
IMG_H = 2434

# Spatial zone thresholds (calibrated on actual page images)
HEADER_Y_MAX = 220          # running title + page number
LEFT_MARGIN_X_MAX = 115     # line numbers / B-refs on recto pages (left edge check)
RIGHT_MARGIN_X_MIN = 1440   # line numbers / B-refs on verso pages
FOOTNOTE_Y_MIN_FRAC = 0.65  # footnotes only in lower 35% of page
FOOTNOTE_GAP_PX = 45        # minimum y-gap to detect footnote boundary

# Front matter pages (1-based index): skip margin/header/footnote heuristics
FRONT_MATTER_END = 12

# ---------------------------------------------------------------------------
# Word dataclass (plain dict for zero-dep)
# ---------------------------------------------------------------------------

def parse_tsv(tsv_text):
    """Parse Tesseract TSV output into list of word dicts."""
    words = []
    reader = csv.DictReader(StringIO(tsv_text), delimiter="\t")
    for row in reader:
        text = row.get("text", "").strip()
        if not text:
            continue
        try:
            words.append({
                "level": int(row["level"]),
                "block_num": int(row["block_num"]),
                "par_num": int(row["par_num"]),
                "line_num": int(row["line_num"]),
                "word_num": int(row["word_num"]),
                "left": int(row["left"]),
                "top": int(row["top"]),
                "width": int(row["width"]),
                "height": int(row["height"]),
                "conf": float(row["conf"]),
                "text": text,
            })
        except (ValueError, KeyError):
            continue
    return words


def run_tesseract(image_path, lang="frk", psm=3):
    """Run tesseract on an image and return TSV output string."""
    cmd = [
        "tesseract", image_path, "stdout",
        "--psm", str(psm),
        "-l", lang,
        "tsv",
    ]
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    if result.returncode != 0:
        raise RuntimeError(f"Tesseract failed on {image_path}: {result.stderr}")
    return result.stdout


# ---------------------------------------------------------------------------
# Zone partitioning
# ---------------------------------------------------------------------------

def is_margin_annotation(w):
    """Check if a word is a margin annotation (line number or B-edition ref) by position + pattern."""
    x = w["left"]
    in_left = x < LEFT_MARGIN_X_MAX
    in_right = x > RIGHT_MARGIN_X_MIN
    if not (in_left or in_right):
        return False
    kind, _ = classify_margin_item(w["text"])
    return kind is not None


def partition_zones(words, page_index):
    """Split words into header, margin_items, body, footnote zones."""
    header = []
    margin_items = []
    body = []

    is_front_matter = page_index <= FRONT_MATTER_END

    for w in words:
        y = w["top"]

        # Header zone
        if y < HEADER_Y_MAX and not is_front_matter:
            header.append(w)
            continue

        # Margin annotations: identified by position + content pattern
        if not is_front_matter and y >= HEADER_Y_MAX and is_margin_annotation(w):
            margin_items.append(w)
            continue

        body.append(w)

    # Detect footnote boundary within body words
    footnote_words = []
    if not is_front_matter:
        body, footnote_words = split_footnotes(body)

    return header, margin_items, body, footnote_words


def split_footnotes(body_words):
    """Detect footnote boundary by y-gap + marker pattern in lower portion."""
    if not body_words:
        return body_words, []

    # Sort body words by top position
    sorted_by_y = sorted(body_words, key=lambda w: w["top"])

    page_y_min = FOOTNOTE_Y_MIN_FRAC * IMG_H

    # Find the largest y-gap in the lower portion
    best_gap_y = None
    best_gap_size = 0

    for i in range(1, len(sorted_by_y)):
        prev_bottom = sorted_by_y[i - 1]["top"] + sorted_by_y[i - 1]["height"]
        curr_top = sorted_by_y[i]["top"]
        gap = curr_top - prev_bottom

        if curr_top > page_y_min and gap >= FOOTNOTE_GAP_PX:
            if gap > best_gap_size:
                best_gap_size = gap
                best_gap_y = curr_top

    if best_gap_y is None:
        return body_words, []

    # Check that text after the gap contains a footnote marker pattern
    words_after = [w for w in body_words if w["top"] >= best_gap_y]
    if not words_after:
        return body_words, []

    # Look for marker pattern in first 20 words after gap (sorted by position)
    first_words = sorted(words_after, key=lambda w: (w["top"], w["left"]))[:20]
    first_words_text = " ".join(w["text"] for w in first_words)
    if not re.search(r"\d+\)|[*†‡][\)\.]", first_words_text):
        return body_words, []

    main_body = [w for w in body_words if w["top"] < best_gap_y]
    return main_body, words_after


# ---------------------------------------------------------------------------
# Margin annotation detection
# ---------------------------------------------------------------------------

ROMAN_RE = re.compile(r"^[IVXLCDM]{2,}$")


def classify_margin_item(text):
    """Classify a margin annotation as line_number, b_page_ref, or None."""
    cleaned = text.strip().rstrip(".")

    # Line numbers: 1-2 digits, multiples of 5, value 5-40
    if re.match(r"^\d{1,2}$", cleaned):
        val = int(cleaned)
        if val % 5 == 0 and 5 <= val <= 40:
            return "line_number", val

    # B-edition page refs: Roman numerals
    if ROMAN_RE.match(cleaned):
        return "b_page_ref", cleaned

    # B-edition page refs: arabic numbers >= 50
    if re.match(r"^\d{2,4}$", cleaned):
        val = int(cleaned)
        if val >= 50:
            return "b_page_ref", str(val)

    return None, None


def extract_margin_annotations(margin_items):
    """Extract line numbers and B-edition refs from margin words.

    Returns:
        line_numbers: list of (y_center, value) tuples
        b_page_refs: list of (y_center, ref_string) tuples
    """
    line_numbers = []
    b_page_refs = []

    for w in margin_items:
        kind, val = classify_margin_item(w["text"])
        y_center = w["top"] + w["height"] // 2
        if kind == "line_number":
            line_numbers.append((y_center, val))
        elif kind == "b_page_ref":
            b_page_refs.append((y_center, val))

    line_numbers.sort(key=lambda t: t[0])
    b_page_refs.sort(key=lambda t: t[0])
    return line_numbers, b_page_refs


# ---------------------------------------------------------------------------
# Post-processing
# ---------------------------------------------------------------------------

def postprocess_fraktur_ocr(text):
    """Fix systematic Tesseract Fraktur misrecognitions."""
    # <h → ch  (e.g. dur<h → durch, auc<h → auch)
    # Must come before the bare < rule to avoid double-replace.
    text = text.replace("<h", "ch")

    # Remaining < → ch  (e.g. dur< → durch, ni<t → nicht)
    text = text.replace("<", "ch")

    # > → ck  (e.g. zurü> → zurück, Ste>en → Stecken)
    text = text.replace(">", "ck")

    # 8 between letters → s  (e.g. bi8her → bisher)
    # Guard: only when flanked by word chars on both sides.
    text = re.sub(r"(?<=\w)8(?=\w)", "s", text)

    return text


# ---------------------------------------------------------------------------
# Structure assembly
# ---------------------------------------------------------------------------

def group_into_paragraphs(body_words):
    """Group body words by (block_num, par_num), reconstruct text with line breaks."""
    if not body_words:
        return []

    paragraphs = {}
    for w in body_words:
        key = (w["block_num"], w["par_num"])
        if key not in paragraphs:
            paragraphs[key] = []
        paragraphs[key].append(w)

    result = []
    for key in sorted(paragraphs.keys()):
        par_words = paragraphs[key]
        result.append(build_paragraph(par_words))
    return result


def build_paragraph(par_words):
    """Build a paragraph dict from a list of words in the same (block, par)."""
    # Group by line_num
    lines = {}
    for w in par_words:
        ln = w["line_num"]
        if ln not in lines:
            lines[ln] = []
        lines[ln].append(w)

    # Sort lines and words within lines
    text_lines = []
    y_min = float("inf")
    y_max = 0
    for ln in sorted(lines.keys()):
        line_words = sorted(lines[ln], key=lambda w: w["left"])
        text_lines.append(" ".join(w["text"] for w in line_words))
        for w in line_words:
            y_min = min(y_min, w["top"])
            y_max = max(y_max, w["top"] + w["height"])

    text = "\n".join(text_lines)
    return {
        "text": text,
        "y_min": y_min,
        "y_max": y_max,
        "line_count": len(text_lines),
        "char_count": len(text),
    }


def is_heading(par_info):
    """Heuristic: short blocks (< 80 chars, ≤ 2 lines) that start with uppercase."""
    if par_info["char_count"] >= 80 or par_info["line_count"] > 2:
        return False
    # Must start with uppercase letter (headings don't start mid-sentence)
    text = par_info["text"].lstrip("0123456789. ")
    if not text or not text[0].isupper():
        return False
    # Reject if it looks like a sentence ending (continuation fragment)
    stripped = par_info["text"].rstrip()
    if stripped and stripped[-1] in ",;:":
        return False
    return True


def extract_inline_b_ref(text):
    """Extract B-edition ref that Tesseract merged at end of a text line.

    Returns (cleaned_text, ref) or (text, None).
    """
    lines = text.split("\n")
    for i, line in enumerate(lines):
        # Check for Roman numeral at end of line (possibly after whitespace)
        m = re.search(r"\s+([IVXLCDM]{2,})\s*$", line)
        if m:
            ref = m.group(1)
            lines[i] = line[:m.start()]
            return "\n".join(lines), ref
        # Check for arabic number >= 50 at end of line
        m = re.search(r"\s+(\d{2,4})\s*$", line)
        if m:
            val = int(m.group(1))
            if val >= 50:
                lines[i] = line[:m.start()]
                return "\n".join(lines), str(val)
    return text, None


def assign_annotations(par_info, line_numbers, b_page_refs):
    """Assign line numbers and B-edition refs by y-range overlap."""
    y_min = par_info["y_min"]
    y_max = par_info["y_max"]

    # Expand range slightly for overlap tolerance
    margin = 30
    matched_lines = [val for y, val in line_numbers if y_min - margin <= y <= y_max + margin]
    matched_refs = [ref for y, ref in b_page_refs if y_min - margin <= y <= y_max + margin]

    return sorted(set(matched_lines)), matched_refs[0] if matched_refs else None


def detect_footnote_markers_in_text(text):
    """Find footnote markers like 1), 2), *, † in element text."""
    markers = []
    for m in re.finditer(r"(\d+)\)", text):
        markers.append(m.group(1))
    for m in re.finditer(r"([*†‡§])", text):
        markers.append(m.group(1))
    return markers


# ---------------------------------------------------------------------------
# Footnote parsing
# ---------------------------------------------------------------------------

def parse_footnotes(footnote_words):
    """Parse footnote words into list of {marker, text} dicts."""
    if not footnote_words:
        return []

    # Group words into lines by y-proximity, then sort within each line by x
    # Use median y of group (not running average) to avoid drift
    sorted_words = sorted(footnote_words, key=lambda w: w["top"])

    line_groups = []  # list of lists of words
    current_group = []

    for w in sorted_words:
        if current_group:
            # Compare against median y of current group
            ys = [cw["top"] for cw in current_group]
            median_y = sorted(ys)[len(ys) // 2]
            if abs(w["top"] - median_y) > 25:
                line_groups.append(current_group)
                current_group = []
        current_group.append(w)
    if current_group:
        line_groups.append(current_group)

    # Build text lines: sort each group by x-position (left to right)
    lines = []
    for group in line_groups:
        group.sort(key=lambda w: w["left"])
        lines.append(" ".join(w["text"] for w in group))

    full_text = "\n".join(lines)

    # Split on footnote markers: digit(s) followed by )
    # Also handle *) and similar
    parts = re.split(r"(?:^|\n)\s*(\d+|\*|†|‡)\)\s*", full_text)

    footnotes = []
    # parts[0] is text before first marker (usually empty or noise)
    i = 1
    while i < len(parts) - 1:
        marker = parts[i]
        text = parts[i + 1].strip().replace("\n", " ")
        if text:
            footnotes.append({"marker": marker, "text": text})
        i += 2

    return footnotes


# ---------------------------------------------------------------------------
# Header parsing
# ---------------------------------------------------------------------------

def parse_header(header_words):
    """Extract page number from header words."""
    if not header_words:
        return None

    # Page number is usually the rightmost or leftmost numeric item in header
    sorted_by_x = sorted(header_words, key=lambda w: w["left"])

    # Check rightmost word first (common for recto pages)
    for w in reversed(sorted_by_x):
        if re.match(r"^\d+$", w["text"].strip()):
            return w["text"].strip()

    # Check leftmost word (common for verso pages)
    for w in sorted_by_x:
        if re.match(r"^\d+$", w["text"].strip()):
            return w["text"].strip()

    # Check for Roman numerals
    for w in sorted_by_x:
        if ROMAN_RE.match(w["text"].strip()):
            return w["text"].strip()

    return None


# ---------------------------------------------------------------------------
# Page processing
# ---------------------------------------------------------------------------

def process_page(image_path, page_index, lang="frk", psm=3):
    """Process a single page image and return structured JSON dict."""
    tsv_text = run_tesseract(image_path, lang=lang, psm=psm)
    words = parse_tsv(tsv_text)

    # Blank page detection
    meaningful_words = [w for w in words if len(w["text"]) > 1 or w["text"].isalpha()]
    if len(meaningful_words) < 3:
        return {
            "page_index": page_index,
            "page_number": None,
            "page_type": "blank",
            "elements": [],
            "footnotes": [],
        }

    is_front = page_index <= FRONT_MATTER_END
    header, margin_items, body, footnote_words = partition_zones(words, page_index)

    # Page number from header
    page_number = parse_header(header)

    # For front matter, detect page type
    if is_front:
        page_type = detect_front_matter_type(page_index, body)
    else:
        page_type = "body"

    # Margin annotations
    line_numbers, b_page_refs = extract_margin_annotations(margin_items)

    # Build paragraphs from body text
    par_infos = group_into_paragraphs(body)

    # Build elements
    elements = []
    for par in par_infos:
        matched_lines, matched_ref = assign_annotations(par, line_numbers, b_page_refs)

        text = par["text"]

        # Try to extract B-edition ref merged inline by Tesseract (body pages only)
        if matched_ref is None and not is_front:
            text, inline_ref = extract_inline_b_ref(text)
            if inline_ref:
                matched_ref = inline_ref

        markers = detect_footnote_markers_in_text(text)
        elem_type = "heading" if is_heading(par) else "paragraph"

        elements.append({
            "type": elem_type,
            "text": text,
            "emphasis": [],
            "typeface": "fraktur",
            "line_numbers": matched_lines,
            "footnote_markers": markers,
            "b_page_ref": matched_ref,
        })

    # Post-process: standalone B-ref elements (body pages only)
    if not is_front:
        cleaned_elements = []
        for elem in elements:
            text = elem["text"].strip()
            is_b_ref = ROMAN_RE.match(text) or (re.match(r"^\d{2,4}$", text) and int(text) >= 50)
            if is_b_ref and len(text) < 10:
                if cleaned_elements:
                    cleaned_elements[-1]["b_page_ref"] = text
                continue
            cleaned_elements.append(elem)
        elements = cleaned_elements

    # Parse footnotes
    footnotes = parse_footnotes(footnote_words)

    # Fix systematic Fraktur OCR errors
    for elem in elements:
        elem["text"] = postprocess_fraktur_ocr(elem["text"])
    for fn in footnotes:
        fn["text"] = postprocess_fraktur_ocr(fn["text"])

    return {
        "page_index": page_index,
        "page_number": page_number,
        "page_type": page_type,
        "elements": elements,
        "footnotes": footnotes,
    }


def detect_front_matter_type(page_index, body_words):
    """Classify front matter pages."""
    if not body_words:
        return "blank"

    text = " ".join(w["text"] for w in body_words).lower()

    if page_index <= 2:
        return "title"
    if "inhalt" in text or "inhalts" in text:
        return "toc"

    return "other"


# ---------------------------------------------------------------------------
# CLI and main
# ---------------------------------------------------------------------------

def find_page_images(pages_dir):
    """Find all page images sorted by filename."""
    patterns = ["*.png", "*.jpg", "*.jpeg"]
    files = []
    for p in patterns:
        files.extend(glob(os.path.join(pages_dir, p)))
    return sorted(files)


def main():
    parser = argparse.ArgumentParser(
        description="Extract structured text from Kant KrV pages using Tesseract OCR"
    )
    parser.add_argument("--pages-dir", default="assets/kant1_pages",
                        help="Directory containing page images (default: assets/kant1_pages)")
    parser.add_argument("--output-dir", default="assets/kant1_output_tesseract",
                        help="Directory for per-page JSON output (default: assets/kant1_output_tesseract)")
    parser.add_argument("--output", default="assets/kant1_kritik_tesseract.json",
                        help="Final merged JSON output path")
    parser.add_argument("--start", type=int, default=1,
                        help="Start page index, 1-based (default: 1)")
    parser.add_argument("--end", type=int, default=None,
                        help="End page index, 1-based inclusive")
    parser.add_argument("--merge-only", action="store_true",
                        help="Skip extraction, just merge existing page JSONs")
    parser.add_argument("--lang", default="Fraktur",
                        help="Tesseract language model (default: Fraktur)")
    parser.add_argument("--psm", type=int, default=3,
                        help="Tesseract page segmentation mode (default: 3 = auto)")
    args = parser.parse_args()

    os.makedirs(args.output_dir, exist_ok=True)

    images = find_page_images(args.pages_dir)
    if not images and not args.merge_only:
        print(f"No page images found in {args.pages_dir}/")
        print("Run: pdftoppm -png -r 300 assets/kant_kritik_2ed_1911.pdf assets/kant1_pages/page")
        return

    if not args.merge_only:
        total = len(images)
        end_idx = args.end if args.end else total

        print(f"Found {total} page images. Processing pages {args.start}\u2013{end_idx}.")

        for i in range(args.start - 1, min(end_idx, total)):
            page_num = i + 1
            out_path = os.path.join(args.output_dir, f"page_{page_num:04d}.json")

            if os.path.exists(out_path):
                print(f"  [{page_num}/{end_idx}] Skipping (exists)")
                continue

            print(f"  [{page_num}/{end_idx}] {os.path.basename(images[i])}...", end=" ", flush=True)

            try:
                result = process_page(images[i], page_num, lang=args.lang, psm=args.psm)
                with open(out_path, "w", encoding="utf-8") as f:
                    json.dump(result, f, ensure_ascii=False, indent=2)
                n_elem = len(result["elements"])
                n_fn = len(result["footnotes"])
                print(f"done ({result['page_type']}, {n_elem} elements, {n_fn} footnotes)")
            except Exception as e:
                print(f"FAILED: {e}", file=sys.stderr)

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
