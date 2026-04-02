Done. Here's a summary of what was implemented:

New file: scripts/extract_kant_docai.py — Document AI OCR extraction script that mirrors the Tesseract script's CLI, output schema, and directory conventions.

Key implementation details:

- Sends PNGs to the europe-west2 OCR processor with de language hint
- Works at the line level (blocks/paragraphs are too coarse)
- Reuses the Tesseract spatial thresholds for zone partitioning (header, body, footnotes)
- Strips line numbers and B-refs from body line text via regex
- Groups lines into paragraphs by spacing (1.5× median gap) and indentation
- Handles Document AI's superscript Unicode digits (¹²³⁴⁵) in footnote markers
- Lowered FOOTNOTE_GAP_PX to 30 (Doc AI line boxes are taller than Tesseract word boxes)
- Uses earliest-qualifying-gap strategy for footnote detection (avoids false positives from stray bottom-of-page elements)
- Handles standalone line numbers that Document AI returns as separate lines
- No Fraktur post-processing — Document AI handles ch/ck/s correctly

Other changes:

- .gitignore — added assets/kant_output_docai/
- Deleted scripts/test_document_ai.py (folded into the real script)

Test results verified:

- Page 20: durch, nicht, Wisſenſchaft, nachzuſpüren all correct; line numbers [5,10,15,20,25,30,35] and B-refs XV/XVI extracted; page number "11"
- Pages 1-10: front matter type detection works (title, blank, toc)
- Page 40: 5 footnotes correctly parsed
- Page 100: B-refs "103"/"104" (Arabic ≥50) detected
- --merge-only works correctly

✻ Sautéed for 7m 55s
