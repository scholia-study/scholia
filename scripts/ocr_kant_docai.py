#!/usr/bin/env python3
"""Send page images to GCP Document AI OCR and cache results.

This is the first stage of the pipeline: it calls Document AI for each page
image and saves two things per page:

  1. The full raw Document AI response (as proto-JSON) — preserved so we can
     re-extract with different heuristics or access word/symbol-level data,
     confidence scores, detected languages, etc.

  2. Extracted line-level OCR data (text + bounding boxes) — the lightweight
     format that extract_kant_docai.py reads.
"""

import argparse
import json
import os
import sys
from glob import glob

from google.api_core.client_options import ClientOptions
from google.cloud import documentai_v1 as documentai
from google.protobuf.json_format import MessageToJson

# ---------------------------------------------------------------------------
# GCP Document AI config
# ---------------------------------------------------------------------------
PROJECT_ID = "cerebro-401111"
OCR_LOCATION = "europe-west2"
OCR_PROCESSOR_ID = "c5387b98e91f93de"

# ---------------------------------------------------------------------------
# Image dimensions (1575 × 2434 px at 300 DPI)
# ---------------------------------------------------------------------------
IMG_W = 1575
IMG_H = 2434


# ---------------------------------------------------------------------------
# Document AI OCR
# ---------------------------------------------------------------------------

def call_document_ai(image_path):
    """Send image to Document AI OCR and return the Document proto."""
    opts = ClientOptions(api_endpoint=f"{OCR_LOCATION}-documentai.googleapis.com")
    client = documentai.DocumentProcessorServiceClient(client_options=opts)
    name = client.processor_path(PROJECT_ID, OCR_LOCATION, OCR_PROCESSOR_ID)

    with open(image_path, "rb") as f:
        image_content = f.read()

    mime = "image/png" if image_path.endswith(".png") else "image/jpeg"
    raw_document = documentai.RawDocument(content=image_content, mime_type=mime)

    process_options = documentai.ProcessOptions(
        ocr_config=documentai.OcrConfig(
            hints=documentai.OcrConfig.Hints(language_hints=["de"]),
        )
    )
    request = documentai.ProcessRequest(
        name=name,
        raw_document=raw_document,
        process_options=process_options,
    )

    result = client.process_document(request=request)
    return result.document


def extract_lines_from_document(document):
    """Extract lines with text and bounding boxes from Document AI response.

    Returns list of dicts: {text, x, y, width, height}
    """
    if not document.pages:
        return []

    page = document.pages[0]
    lines = []

    for line in page.lines:
        # Extract text via text_anchor
        segments = line.layout.text_anchor.text_segments
        text = "".join(
            document.text[int(s.start_index):int(s.end_index)]
            for s in segments
        ).rstrip("\n")

        if not text.strip():
            continue

        # Extract bounding box from vertices
        verts = line.layout.bounding_poly.normalized_vertices
        if not verts:
            continue

        # Convert normalized coords to pixel coords
        xs = [v.x * IMG_W for v in verts]
        ys = [v.y * IMG_H for v in verts]

        x_min = min(xs)
        y_min = min(ys)
        x_max = max(xs)
        y_max = max(ys)

        lines.append({
            "text": text,
            "x": x_min,
            "y": y_min,
            "width": x_max - x_min,
            "height": y_max - y_min,
        })

    return lines


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
        description="Send Kant KrV page images to GCP Document AI OCR and cache results"
    )
    parser.add_argument("--pages-dir", default="assets/kant1_pages",
                        help="Directory containing page images (default: assets/kant1_pages)")
    parser.add_argument("--raw-dir", default="assets/kant1_ocr_raw",
                        help="Directory for full Document AI responses (default: assets/kant1_ocr_raw)")
    parser.add_argument("--ocr-cache-dir", default="assets/kant1_ocr_cache",
                        help="Directory for extracted line-level OCR cache (default: assets/kant1_ocr_cache)")
    parser.add_argument("--start", type=int, default=1,
                        help="Start page index, 1-based (default: 1)")
    parser.add_argument("--end", type=int, default=None,
                        help="End page index, 1-based inclusive")
    parser.add_argument("--force", action="store_true",
                        help="Re-OCR even if cached result exists")
    args = parser.parse_args()

    os.makedirs(args.raw_dir, exist_ok=True)
    os.makedirs(args.ocr_cache_dir, exist_ok=True)

    images = find_page_images(args.pages_dir)
    if not images:
        print(f"No page images found in {args.pages_dir}/")
        print("Run: pdftoppm -png -r 300 assets/kant_kritik_2ed_1911.pdf assets/kant1_pages/page")
        return

    total = len(images)
    end_idx = args.end if args.end else total

    print(f"Found {total} page images. OCR-ing pages {args.start}–{end_idx}.")

    for i in range(args.start - 1, min(end_idx, total)):
        page_num = i + 1
        basename = os.path.splitext(os.path.basename(images[i]))[0]
        cache_path = os.path.join(args.ocr_cache_dir, f"{basename}.json")

        if os.path.exists(cache_path) and not args.force:
            print(f"  [{page_num}/{end_idx}] {os.path.basename(images[i])} — cached, skipping")
            continue

        print(f"  [{page_num}/{end_idx}] {os.path.basename(images[i])}...", end=" ", flush=True)

        try:
            document = call_document_ai(images[i])

            # Save full raw Document AI response
            raw_path = os.path.join(args.raw_dir, f"{basename}.json")
            with open(raw_path, "w", encoding="utf-8") as f:
                f.write(MessageToJson(documentai.Document.pb(document)))

            # Save extracted lines
            lines = extract_lines_from_document(document)
            with open(cache_path, "w", encoding="utf-8") as f:
                json.dump(lines, f, ensure_ascii=False, indent=2)

            print(f"done ({len(lines)} lines)")
        except Exception as e:
            print(f"FAILED: {e}", file=sys.stderr)

    cached = len(glob(os.path.join(args.ocr_cache_dir, "*.json")))
    print(f"OCR cache now has {cached} files in {args.ocr_cache_dir}/")


if __name__ == "__main__":
    main()
