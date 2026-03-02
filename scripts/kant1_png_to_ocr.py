#!/usr/bin/env python3
"""Send page images to GCP Document AI OCR and save raw responses.

First stage of the pipeline: calls Document AI for each page image and saves
the full raw response (as proto-JSON) so downstream tools can extract whatever
they need (lines, words, symbols, confidence scores, etc.).
"""

import argparse
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
        description="Send Kant KrV page images to GCP Document AI OCR"
    )
    parser.add_argument("--pages-dir", default="assets/kant1_pages",
                        help="Directory containing page images (default: assets/kant1_pages)")
    parser.add_argument("--output-dir", default="assets/kant1_png_to_ocr",
                        help="Directory for raw Document AI responses (default: assets/kant1_png_to_ocr)")
    parser.add_argument("--start", type=int, default=1,
                        help="Start page index, 1-based (default: 1)")
    parser.add_argument("--end", type=int, default=None,
                        help="End page index, 1-based inclusive")
    args = parser.parse_args()

    os.makedirs(args.output_dir, exist_ok=True)

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
        out_path = os.path.join(args.output_dir, f"{basename}.json")

        print(f"  [{page_num}/{end_idx}] {os.path.basename(images[i])}...", end=" ", flush=True)

        try:
            document = call_document_ai(images[i])

            with open(out_path, "w", encoding="utf-8") as f:
                f.write(MessageToJson(documentai.Document.pb(document)))

            print("done")
        except Exception as e:
            print(f"FAILED: {e}", file=sys.stderr)

    saved = len(glob(os.path.join(args.output_dir, "*.json")))
    print(f"{saved} raw OCR files in {args.output_dir}/")


if __name__ == "__main__":
    main()
