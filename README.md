# Prospero

Structured extraction of Hegel's *Wissenschaft der Logik* (Science of Logic) from EPUB to JSON.

The pipeline reads `assets/wdl.epub` and produces `assets/wdl.json` -- a single JSON file containing the complete text with hierarchical table-of-contents structure, book metadata, global paragraph numbering, and sentence-level segmentation.

## Running

```
cargo run -p hegel-wdl
```

Reads `assets/wdl.epub`, writes `assets/wdl.json`.

## JSON output structure

### Top level

```json
{
  "title": "Wissenschaft der Logik",
  "author": "Georg Wilhelm Friedrich Hegel",
  "language": "de",
  "publisher": "Zeno.org",
  "date": "2015-06-29",
  "nodes": [ ... ]
}
```

| Field       | Type     | Description                                     |
|-------------|----------|-------------------------------------------------|
| `title`     | string   | Book title from the NCX navigation file         |
| `author`    | string   | Author with `role="aut"` from the OPF metadata  |
| `language`  | string   | ISO 639-1 language code                         |
| `publisher` | string   | Publisher name from OPF metadata                |
| `date`      | string   | Publication date (ISO 8601)                     |
| `nodes`     | array    | Top-level table-of-contents nodes (see below)   |

### Nodes (`TocNode`)

The `nodes` array is a recursive tree that mirrors the EPUB's table of contents. Each node can contain its own content and child nodes. Nesting goes up to 8 levels deep, reflecting the detailed structure of Hegel's text (Parts > Books > Sections > Chapters > Sub-sections > Remarks > etc.).

```json
{
  "ncx_id": "np-42",
  "play_order": 42,
  "label": "C. Werden",
  "depth": 6,
  "children": [ ... ],
  "content": [ ... ]
}
```

| Field        | Type     | Description                                                        |
|--------------|----------|--------------------------------------------------------------------|
| `ncx_id`     | string   | Unique navigation ID from the EPUB NCX (e.g. `"np-42"`)           |
| `play_order` | integer  | Reading order index from the NCX (1-based, sequential)             |
| `label`      | string   | Human-readable section title (e.g. `"Vorrede zur ersten Ausgabe"`) |
| `depth`      | integer  | Nesting depth in the TOC tree (1 = top level, up to 8)             |
| `children`   | array    | Child `TocNode` objects (empty `[]` for leaf nodes)                |
| `content`    | array    | Ordered content blocks belonging to this section (see below)       |

Every node has content -- there are 229 nodes total, all with at least one content block.

### Content blocks (`ContentBlock`)

The `content` array within each node is a flat, ordered sequence of blocks representing the section's text. Each block has a `type` that determines which fields are present.

There are four block types:

#### `"paragraph"` -- body text (1,913 total)

The primary content type. Each paragraph carries a global number and is split into sentences.

```json
{
  "position": 22,
  "type": "paragraph",
  "paragraph_number": 42,
  "text": "Um dies in die Vorstellung wenigstens aufzunehmen, ...",
  "html": "Um dies in die Vorstellung wenigstens aufzunehmen, ...",
  "page_ref": "44",
  "sentences": [
    {
      "position": 0,
      "sentence_number": 199,
      "text": "Um dies in die Vorstellung wenigstens aufzunehmen, ...",
      "html": "Um dies in die Vorstellung wenigstens aufzunehmen, ..."
    },
    {
      "position": 1,
      "sentence_number": 200,
      "text": "Solche Handgreiflichkeit wird zum Beispiel ...",
      "html": "Solche Handgreiflichkeit wird zum Beispiel ..."
    }
  ]
}
```

#### `"heading"` -- section titles (324 total)

Chapter and section headings extracted from `<h1>` through `<h5>` elements.

```json
{
  "position": 0,
  "type": "heading",
  "text": "Georg Wilhelm Friedrich Hegel",
  "html": "Georg Wilhelm Friedrich Hegel"
}
```

#### `"footnote"` -- authorial footnotes (23 total)

Footnotes from the source text, appearing at the end of certain chapters.

```json
{
  "position": 12,
  "type": "footnote",
  "text": "1 Phänomenologie des Geistes, Vorrede zur ersten Ausgabe. ...",
  "html": "1 Phänomenologie des Geistes, Vorrede zur ersten Ausgabe. ..."
}
```

#### `"separator"` -- visual breaks (280 total)

Empty-line separators between text segments. Both `text` and `html` are empty strings.

```json
{
  "position": 11,
  "type": "separator",
  "text": "",
  "html": ""
}
```

#### Field reference

| Field              | Type           | Present on              | Description                                                       |
|--------------------|----------------|-------------------------|-------------------------------------------------------------------|
| `position`         | integer        | all types               | 0-based index within the parent node's `content` array            |
| `type`             | string         | all types               | One of `"paragraph"`, `"heading"`, `"footnote"`, `"separator"`    |
| `paragraph_number` | integer        | `paragraph` only        | Global 1-based paragraph count across the entire book (1..1,913)  |
| `text`             | string         | all types               | Plain text content (empty for separators)                         |
| `html`             | string         | all types               | HTML content preserving inline formatting (empty for separators)  |
| `page_ref`         | string or null | `paragraph`, `footnote` | Page number from the source edition, if present (e.g. `"44"`)    |
| `sentences`        | array          | `paragraph` only        | Sentence-level breakdown (see below)                              |

Fields that are absent from the JSON (rather than null) when not applicable: `paragraph_number`, `page_ref`, `sentences`. This keeps heading, footnote, and separator blocks compact.

### `text` vs `html`

Every paragraph and heading has both a `text` and an `html` field:

- **`text`** is the plain-text content with all markup stripped. Use this for search, NLP, and analysis.
- **`html`** preserves inline formatting tags from the source: `<i>` (italic), `<b>` (bold), `<sup>` (superscript), `<sub>` (subscript). All other tags and page-reference links are stripped.

Example where they differ:

```
text: "Dieses Reich ist die Wahrheit, wie sie ohne Hülle an und für sich selbst ist."
html: "<i>Dieses Reich ist die Wahrheit, wie sie ohne Hülle an und für sich selbst ist</i>."
```

### Sentences

Each paragraph is split into individual sentences. Sentence boundaries are detected by punctuation (`.` `?` `!`) followed by whitespace and an uppercase letter, with exceptions for German abbreviations (`d. i.`, `z. B.`, `u. dgl.`, etc.) and single-letter initials (`G. W. F.`).

```json
{
  "position": 0,
  "sentence_number": 470,
  "text": "Das reine Sein und das reine Nichts ist also dasselbe.",
  "html": "Das reine Sein und das reine Nichts ist also dasselbe."
}
```

| Field             | Type    | Description                                                      |
|-------------------|---------|------------------------------------------------------------------|
| `position`        | integer | 0-based index within the parent paragraph's `sentences` array    |
| `sentence_number` | integer | Global 1-based sentence count across the entire book (1..7,950)  |
| `text`            | string  | Plain text of the sentence                                       |
| `html`            | string  | HTML of the sentence with inline formatting preserved             |

When a formatting tag (e.g. `<i>`) spans a sentence boundary in the source paragraph, the HTML is **rebalanced** so that each sentence's HTML is independently well-formed. Tags that are open at the end of a sentence are closed, and reopened at the start of the next:

```
Paragraph html: "<i>Erster Satz. Zweiter Satz.</i>"
Sentence 1 html: "<i>Erster Satz.</i>"
Sentence 2 html: "<i>Zweiter Satz.</i>"
```

### Numbering

Two global counters run across the entire book in depth-first reading order:

- **`paragraph_number`** (1..1,913) -- counts only `"paragraph"` blocks, skipping headings, footnotes, and separators.
- **`sentence_number`** (1..7,950) -- counts every sentence across all paragraphs.

Both are contiguous with no gaps, so they can serve as stable identifiers for citation and cross-referencing.

### Statistics

| Metric          | Count |
|-----------------|-------|
| TOC nodes       | 229   |
| Paragraphs      | 1,913 |
| Sentences        | 7,950 |
| Headings         | 324   |
| Separators       | 280   |
| Footnotes        | 23    |
| Total blocks     | 2,540 |
| Max TOC depth    | 8     |

## Project structure

```
prospero/
  assets/
    wdl.epub              # Source EPUB
    wdl.json              # Generated output
  lib/common/src/
    model.rs              # Data structures (Book, TocNode, ContentBlock, Sentence)
    epub_reader.rs        # ZIP-based EPUB file reader
    ncx.rs                # NCX table-of-contents parser
    opf.rs                # OPF metadata parser
    content.rs            # XHTML content extraction and paragraph/sentence numbering
    sentences.rs          # Sentence splitter with HTML tag rebalancing
  packages/hegel-wdl/src/
    main.rs               # CLI entry point
```
