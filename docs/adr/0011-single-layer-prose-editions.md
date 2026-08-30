# 0011. Single-layer prose editions

**Status**: Accepted
**Date**: 2026-08-30

## Context

Every prose corpus so far has been built in one of two modes. *Source* mode
pairs two curated layers — `md_modernized` as the reading text and
`md_reviewed` as the original orthography — and the reader offers an
"Original orthography" toggle between them. *Translation* mode takes a single
curated layer and locks it 1:1 to a source book by sentence, threading
alignments through `--source-book-slug`.

Both assume the text needs one of those relationships. peirce1
(*Essays in Pragmaticism*, Peirce's journal papers 1867–1908) needs neither. It is
English, so there is nothing to translate from; and its orthography is
already modern, so a reviewed layer would be a near-duplicate of the reading
text, differing in perhaps 50–150 tokens across 400 pages. Manufacturing a
second layer to satisfy the parser would put a toggle in the reader that
changes almost nothing — a reader-facing claim about the text that isn't
true.

The alternative considered was to point both layer directories at the same
files. That parses, but produces a toggle between two identical texts, which
is worse: it asserts a distinction that does not exist.

## Decision

**A third parser mode, `single`: one curated layer, no pairing, no
translation lock.** Selected with `--single` on `md_prose_to_struct`, which
conflicts with `--translation` at the CLI level.

The struct schema needed no change. `original_text` / `original_html` are
already `Option<String>` throughout `text_struct::model`, and `ParsedFile`
already carried `original_blocks: Option<…>` for translation editions — so
"there is no second layer" was expressible before this mode existed.
`struct_to_db` needed no change either: it binds those Options straight into
the INSERT, and NULL originals already worked.

`build_output` treats `single` exactly like `source` — same TOC table for
labels, same sentence splitter selection. The only difference is collection:
`collect_single_files` reads one directory and emits `original_blocks: None`.

**The reader hides the orthography toggle when there is nothing to toggle.**
`BookDetail.has_original_layer` is computed per book (`EXISTS` over
`content_blocks.original_html`), and the menu item is gated on it — mirroring
how the margin-reference section is already gated on `availableSystems`.
Rendering already degraded safely (`showOriginal && s.original_html ? … :
s.html`), so this is about the control, not the text.

## Consequences

- A corpus whose text is already modern no longer has to invent a second
  layer, and the reader stops offering a switch that does nothing.
- Adding such a corpus stays data-only (ADR 0006): a `common::<corpus>`
  module plus a builder arm, with `--single` in its `struct.sh` case arm.
- Existing corpora are untouched. Verified by rebuilding hobbes1, hegel1 and
  hegel2 after the change and confirming byte-identical struct JSON.
- Two-layer corpora remain the norm for texts with genuine orthographic
  distance (Kant, Hegel, Hobbes, Milton, Shakespeare). `single` is for texts
  where the distance is absent, not for skipping curation work.
