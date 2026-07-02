# kant1_elements_to_struct

Pipeline stage that consolidates per-page `kant1_lines_to_elements` output into a
single hierarchical JSON file (`kant1_kritik.json`) matching the `KantBook` structure.

## Where it fits

```
OCR images
  → kant1_ocr_to_lines      (raw OCR → lines with coordinates)
  → kant1_lines_to_elements  (lines → paragraphs, headings, footnotes per page)
  → kant1_elements_to_struct (per-page elements → structured book with TOC)  ← this package
  → struct_to_db / api (JSON → Postgres → REST)
```

## What it does

1. **Reads** all `assets/kant1/raw/lines_to_elements/page_*.json` files.
2. **Infers AA page numbers** for pages that lack an explicit `page_number` header
   using the empirical formula `aa_page = page_index − 9` (derived from the
   consistent 1:1 mapping across all pages where the header was OCR'd).
3. **Stitches hyphenated words** across line and page boundaries
   (`anthropolo-` + `gische` → `anthropologische`).
4. **Assigns elements to TOC nodes** by matching each element's AA page number
   against the authoritative table of contents. The assignment uses a
   last-match lookup: the most recently started section (in document order)
   whose `aa_page ≤` the element's page receives the content.
5. **Splits paragraphs into sentences** using the German-aware splitter from
   `packages/common/src/sentences.rs` (handles abbreviations like d. i., z. B., etc.).
6. **Preserves B-edition page references** (`b_page_ref`) at both the
   content-block and sentence level, anchored by character offset.
7. **Numbers** paragraphs and sentences with globally sequential counters.
8. **Outputs** per-section markdown into `assets/kant1/raw/elements_to_md/`.

## Authoritative TOC

The OCR'd table of contents (scan pages 4–8) is too noisy for reliable
structure recovery. Instead, `src/toc.rs` contains an 88-entry hardcoded TOC
derived from the Akademie-Ausgabe Band III, cross-referenced with scholarly
sources. Each entry carries:

- **label** — section title (German)
- **aa_page** — AA III page where the section starts (primary key for matching)
- **depth** — nesting level (1 = top-level, up to 8 for the Analogies)

The flat list is built into a tree at runtime. Five top-level nodes:

| Node | AA page |
|------|---------|
| Zueignung | 1 |
| Vorrede zur zweiten Auflage | 3 |
| Einleitung | 27 |
| I. Transscendentale Elementarlehre | 49 |
| II. Transscendentale Methodenlehre | 442 |

## Output types

The package defines its own Kant-specific types rather than reusing
`packages/common/src/model.rs`, because the OCR pipeline has no HTML and needs
fields like `aa_page` and `b_page_refs` instead of EPUB-specific ones
(`ncx_id`, `play_order`, `html`).

- `KantBook` — top-level container
- `KantTocNode` — recursive TOC node with `aa_page`, `depth`, `children`, `content`
- `KantContentBlock` — paragraph, heading, or footnote with `b_page_refs` and `sentences`
- `KantSentence` — individual sentence with optional `b_page_ref` anchor

## Usage

```sh
# From the workspace root:
cargo run --package kant1_elements_to_struct

# Custom paths:
cargo run --package kant1_elements_to_struct -- \
  --input-dir assets/kant1/raw/lines_to_elements \
  --output-dir assets/kant1/raw/elements_to_md
```

## Known limitations

- **Shared-page sections**: When two TOC sections start on the same AA page
  (e.g. Einleitung II and III both at AA 28), all content from that page goes
  to the later section. A future refinement could split within-page content by
  matching heading text.
- **Scan coverage**: The current scan data covers ~40 pages (AA 1–31), so only
  the Zueignung, Vorrede, and early Einleitung are populated. The full TOC
  skeleton is present and ready for more pages.
