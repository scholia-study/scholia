#!/usr/bin/env python3
"""Extract structured text from cached Document AI OCR data.

Second stage of the pipeline: reads raw OCR line JSON from the cache
directory (produced by ocr_kant_docai.py) and applies heuristics to produce
structured per-page output (zones, headers, paragraphs, footnotes, etc.).

Unlike the Tesseract script, elements preserve per-line OCR data: each element
has a `lines` array where each entry carries the cleaned text plus any
`line_number` or `b_page_ref` detected on that specific line.
"""

import argparse
import json
import os
import re
import statistics
import sys
from glob import glob

# ---------------------------------------------------------------------------
# Image dimensions (1575 × 2434 px at 300 DPI)
# ---------------------------------------------------------------------------
IMG_W = 1575
IMG_H = 2434

# Spatial zone thresholds (calibrated on actual page images)
HEADER_Y_MAX = 220          # running title + page number
FOOTNOTE_Y_MIN_FRAC = 0.65  # footnotes only in lower 35% of page
FOOTNOTE_GAP_PX = 30        # minimum y-gap to detect footnote boundary (lower than Tesseract:
                             # Doc AI line bounding boxes are taller, so inter-line gaps are smaller)

# Front matter pages (1-based index): skip margin/header/footnote heuristics
FRONT_MATTER_END = 12

# Margin annotation patterns
ROMAN_RE = re.compile(r"^[IVXLCDM]{2,}$")

# Superscript digit mapping (Document AI often returns these for footnote markers)
SUPERSCRIPT_MAP = str.maketrans("⁰¹²³⁴⁵⁶⁷⁸⁹", "0123456789")
SUPERSCRIPT_DIGITS = set("⁰¹²³⁴⁵⁶⁷⁸⁹")

# Regex for footnote marker detection (including superscript digits)
FOOTNOTE_MARKER_RE = re.compile(r"[¹²³⁴⁵⁶⁷⁸⁹⁰\d]+\)|[*†‡][\)\.]")


# ---------------------------------------------------------------------------
# Zone partitioning
# ---------------------------------------------------------------------------

def partition_zones(lines, page_index):
    """Split lines into header, body, footnote zones.

    Returns (header_lines, body_lines, footnote_lines).
    """
    is_front_matter = page_index <= FRONT_MATTER_END

    header = []
    body = []

    for line in lines:
        if line["y"] < HEADER_Y_MAX and not is_front_matter:
            header.append(line)
        else:
            body.append(line)

    # Detect footnote boundary within body lines
    footnote_lines = []
    if not is_front_matter:
        body, footnote_lines = split_footnotes(body)

    return header, body, footnote_lines


def split_footnotes(body_lines):
    """Detect footnote boundary by y-gap + marker pattern in lower portion.

    Finds the earliest qualifying gap (not the largest) whose text below
    contains a footnote marker pattern. This avoids false positives from
    stray elements at the very bottom of the page.
    """
    if not body_lines:
        return body_lines, []

    sorted_by_y = sorted(body_lines, key=lambda l: l["y"])
    page_y_min = FOOTNOTE_Y_MIN_FRAC * IMG_H

    # Collect all qualifying gaps, sorted by y position (earliest first)
    gaps = []
    for i in range(1, len(sorted_by_y)):
        prev_bottom = sorted_by_y[i - 1]["y"] + sorted_by_y[i - 1]["height"]
        curr_top = sorted_by_y[i]["y"]
        gap = curr_top - prev_bottom

        if curr_top > page_y_min and gap >= FOOTNOTE_GAP_PX:
            gaps.append((curr_top, gap))

    # Try each gap from earliest to latest, use the first with footnote markers
    for gap_y, _ in gaps:
        lines_after = [l for l in body_lines if l["y"] >= gap_y]
        if not lines_after:
            continue

        first_lines = sorted(lines_after, key=lambda l: (l["y"], l["x"]))[:5]
        first_text = " ".join(l["text"] for l in first_lines)
        if FOOTNOTE_MARKER_RE.search(first_text):
            main_body = [l for l in body_lines if l["y"] < gap_y]
            return main_body, lines_after

    return body_lines, []


# ---------------------------------------------------------------------------
# Header parsing
# ---------------------------------------------------------------------------

def parse_header(header_lines):
    """Extract page number from header lines."""
    if not header_lines:
        return None

    sorted_by_x = sorted(header_lines, key=lambda l: l["x"])

    for line in reversed(sorted_by_x):
        text = line["text"].strip()
        if re.match(r"^\d+$", text):
            return text

    for line in sorted_by_x:
        text = line["text"].strip()
        if re.match(r"^\d+$", text):
            return text

    for line in sorted_by_x:
        text = line["text"].strip()
        if ROMAN_RE.match(text):
            return text

    return None


# ---------------------------------------------------------------------------
# Per-line annotation (line numbers + B-refs)
# ---------------------------------------------------------------------------

def annotate_lines(body_lines):
    """Detect and strip line numbers and B-edition refs from each body line.

    Each OCR line is annotated with:
      - line_number: int or None (margin line number, multiple of 5, range 5-40)
      - b_page_ref: str or None (B-edition page ref — Roman numeral or Arabic)

    Lines that are purely a standalone line number (e.g. Doc AI returns "15"
    as its own line) are dropped from the output.

    Returns list of annotated line dicts (original fields + line_number, b_page_ref).
    """
    result = []

    for line in body_lines:
        text = line["text"]
        line_number = None
        b_page_ref = None

        # --- Standalone line number (entire line is just a number) ---
        stripped = text.strip()
        if re.match(r"^\d{1,2}$", stripped):
            val = int(stripped)
            if val % 5 == 0 and 5 <= val <= 40:
                # Drop this line — it's a margin annotation, not body text
                continue

        # --- Line number at start of line ---
        m = re.match(r"^(\d{1,2})\s+", text)
        if m:
            val = int(m.group(1))
            if val % 5 == 0 and 5 <= val <= 40:
                line_number = val
                text = text[m.end():]

        # --- B-edition ref at end of line ---
        # Roman numerals
        m = re.search(r"\s+([IVXLCDM]{2,})\s*$", text)
        if m:
            b_page_ref = m.group(1)
            text = text[:m.start()]
        else:
            # Arabic number — at line level, any trailing number separated by
            # whitespace is a margin B-ref (no >= 50 guard needed; body text
            # in this book never ends with a bare number)
            m = re.search(r"\s+(\d{1,4})\s*$", text)
            if m:
                b_page_ref = m.group(1)
                text = text[:m.start()]

        # --- Bullet / artifact at line start (e.g. "• Bedenklichkeit...") ---
        text = re.sub(r"^[•]\s*", "", text)

        result.append({
            **line,
            "text": text,
            "line_number": line_number,
            "b_page_ref": b_page_ref,
        })

    return result


# ---------------------------------------------------------------------------
# Paragraph grouping from annotated lines
# ---------------------------------------------------------------------------

def group_into_paragraphs(annotated_lines):
    """Group annotated body lines into paragraphs using spacing and indentation.

    Returns list of line groups, where each group is a list of annotated line dicts.
    """
    if not annotated_lines:
        return []

    sorted_lines = sorted(annotated_lines, key=lambda l: l["y"])

    # Compute median line spacing
    spacings = []
    for i in range(1, len(sorted_lines)):
        spacing = sorted_lines[i]["y"] - sorted_lines[i - 1]["y"]
        if spacing > 0:
            spacings.append(spacing)

    if not spacings:
        return [sorted_lines]

    median_spacing = statistics.median(spacings)

    # Compute median left-x for indentation detection
    left_xs = [l["x"] for l in sorted_lines]
    median_left_x = statistics.median(left_xs)

    # Group lines into paragraphs
    groups = [[sorted_lines[0]]]

    for i in range(1, len(sorted_lines)):
        prev = sorted_lines[i - 1]
        curr = sorted_lines[i]
        spacing = curr["y"] - prev["y"]

        is_gap = spacing > median_spacing * 1.5
        is_indented = curr["x"] > median_left_x + 30

        if is_gap or is_indented:
            groups.append([curr])
        else:
            groups[-1].append(curr)

    return groups


# ---------------------------------------------------------------------------
# Structure assembly
# ---------------------------------------------------------------------------

def is_heading(text, line_count):
    """Heuristic: short blocks (< 80 chars, <= 2 lines) that start with uppercase."""
    if len(text) >= 80 or line_count > 2:
        return False
    clean = text.lstrip("0123456789. ")
    if not clean or not clean[0].isupper():
        return False
    if text.rstrip() and text.rstrip()[-1] in ",;:":
        return False
    return True


def detect_footnote_markers_in_text(text):
    """Find footnote markers like 1), ¹), *, † in element text."""
    markers = []
    for m in re.finditer(r"([¹²³⁴⁵⁶⁷⁸⁹⁰\d]+)\)", text):
        markers.append(normalize_superscripts(m.group(1)))
    for m in re.finditer(r"([*†‡§])", text):
        markers.append(m.group(1))
    return markers


def build_element(line_group):
    """Build an element dict from a group of annotated lines.

    Each element preserves per-line data and derives aggregate fields from it.
    """
    lines_out = []
    for l in line_group:
        lines_out.append({
            "text": l["text"],
            "line_number": l["line_number"],
            "b_page_ref": l["b_page_ref"],
        })

    text = "\n".join(l["text"] for l in lines_out)
    line_numbers = sorted(set(
        l["line_number"] for l in lines_out if l["line_number"] is not None
    ))
    b_refs = [l["b_page_ref"] for l in lines_out if l["b_page_ref"] is not None]

    elem_type = "heading" if is_heading(text, len(lines_out)) else "paragraph"
    markers = detect_footnote_markers_in_text(text)

    return {
        "type": elem_type,
        "text": text,
        "lines": lines_out,
        "emphasis": [],
        "typeface": "fraktur",
        "line_numbers": line_numbers,
        "footnote_markers": markers,
        "b_page_ref": b_refs[0] if b_refs else None,
    }


# ---------------------------------------------------------------------------
# Footnote parsing
# ---------------------------------------------------------------------------

def normalize_superscripts(text):
    """Replace superscript digits with regular digits."""
    return text.translate(SUPERSCRIPT_MAP)


def parse_footnotes(footnote_lines):
    """Parse footnote lines into list of {marker, text} dicts."""
    if not footnote_lines:
        return []

    sorted_lines = sorted(footnote_lines, key=lambda l: (l["y"], l["x"]))
    full_text = "\n".join(l["text"] for l in sorted_lines)

    # Normalize superscript digits before splitting
    full_text = normalize_superscripts(full_text)

    # Split on footnote markers: digit(s) followed by )
    parts = re.split(r"(?:^|\n)\s*(\d+|\*|†|‡)\)\s*", full_text)

    footnotes = []
    i = 1
    while i < len(parts) - 1:
        marker = parts[i]
        text = parts[i + 1].strip().replace("\n", " ")
        if text:
            footnotes.append({"marker": marker, "text": text})
        i += 2

    return footnotes


# ---------------------------------------------------------------------------
# Front matter detection
# ---------------------------------------------------------------------------

def detect_front_matter_type(page_index, body_lines):
    """Classify front matter pages."""
    if not body_lines:
        return "blank"

    text = " ".join(l["text"] for l in body_lines).lower()

    if page_index <= 2:
        return "title"
    if "inhalt" in text or "inhalts" in text:
        return "toc"

    return "other"


# ---------------------------------------------------------------------------
# Page processing
# ---------------------------------------------------------------------------

def process_page(ocr_lines, page_index):
    """Process cached OCR lines for a single page and return structured JSON dict."""
    # Blank page detection
    meaningful = [l for l in ocr_lines if len(l["text"].strip()) > 1]
    if len(meaningful) < 3:
        return {
            "page_index": page_index,
            "page_number": None,
            "page_type": "blank",
            "elements": [],
            "footnotes": [],
        }

    is_front = page_index <= FRONT_MATTER_END

    # Zone partitioning
    header, body, footnote_lines = partition_zones(ocr_lines, page_index)

    # Header parsing
    page_number = parse_header(header)

    # Front matter type detection
    if is_front:
        page_type = detect_front_matter_type(page_index, body)
    else:
        page_type = "body"

    # Annotate body lines (strip & record line numbers + B-refs per line)
    if not is_front:
        body = annotate_lines(body)
    else:
        # Front matter: no margin annotations, just add null fields
        body = [{**l, "line_number": None, "b_page_ref": None} for l in body]

    # Group annotated lines into paragraphs
    para_groups = group_into_paragraphs(body)

    # Build elements from line groups
    elements = [build_element(group) for group in para_groups]

    # Standalone B-ref cleanup: if an element is nothing but a B-ref,
    # merge it into the previous element
    if not is_front:
        cleaned = []
        for elem in elements:
            text = elem["text"].strip()
            is_b_ref = (
                ROMAN_RE.match(text)
                or (re.match(r"^\d{1,4}$", text) and len(text) < 10)
            )
            if is_b_ref and cleaned:
                cleaned[-1]["b_page_ref"] = text
                continue
            cleaned.append(elem)
        elements = cleaned

    # Parse footnotes
    footnotes = parse_footnotes(footnote_lines)

    return {
        "page_index": page_index,
        "page_number": page_number,
        "page_type": page_type,
        "elements": elements,
        "footnotes": footnotes,
    }


# ---------------------------------------------------------------------------
# CLI and main
# ---------------------------------------------------------------------------

def find_ocr_cache_files(ocr_cache_dir):
    """Find all OCR cache JSON files sorted by filename."""
    return sorted(glob(os.path.join(ocr_cache_dir, "*.json")))


def main():
    parser = argparse.ArgumentParser(
        description="Extract structured text from cached Document AI OCR data"
    )
    parser.add_argument("--ocr-cache-dir", default="assets/kant1_ocr_cache",
                        help="Directory containing cached OCR line JSON (default: assets/kant1_ocr_cache)")
    parser.add_argument("--output-dir", default="assets/kant1_output_docai",
                        help="Directory for per-page JSON output (default: assets/kant1_output_docai)")
    parser.add_argument("--output", default="assets/kant1_kritik_docai.json",
                        help="Final merged JSON output path")
    parser.add_argument("--start", type=int, default=1,
                        help="Start page index, 1-based (default: 1)")
    parser.add_argument("--end", type=int, default=None,
                        help="End page index, 1-based inclusive")
    parser.add_argument("--merge-only", action="store_true",
                        help="Skip extraction, just merge existing page JSONs")
    args = parser.parse_args()

    os.makedirs(args.output_dir, exist_ok=True)

    cache_files = find_ocr_cache_files(args.ocr_cache_dir)
    if not cache_files and not args.merge_only:
        print(f"No OCR cache files found in {args.ocr_cache_dir}/")
        print("Run ocr_kant_docai.py first to populate the cache.")
        return

    if not args.merge_only:
        total = len(cache_files)
        end_idx = args.end if args.end else total

        print(f"Found {total} cached OCR files. Processing pages {args.start}–{end_idx}.")

        for i in range(args.start - 1, min(end_idx, total)):
            page_num = i + 1
            out_path = os.path.join(args.output_dir, f"page_{page_num:04d}.json")

            print(f"  [{page_num}/{end_idx}] {os.path.basename(cache_files[i])}...", end=" ", flush=True)

            try:
                with open(cache_files[i], encoding="utf-8") as f:
                    ocr_lines = json.load(f)

                result = process_page(ocr_lines, page_num)
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
