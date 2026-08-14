# 0010. Margin notes as a first-class sentence-anchored concept

**Status**: Accepted
**Date**: 2026-08-11

## Context

Hobbes's *Leviathan* (1651, EEBO-TCP A43998) carries ~874 authorial margin
notes: short topical side-notes running beside the text ("Appetite.",
"These Rights are indivisible.") plus starred scripture citations. They are
part of the work — Chapter XVIII enumerates the rights of sovereignty *in
the margin* — and every modern critical edition retains them as authorial
text. In the TEI they are `<note place="margin">` elements anchored
mid-paragraph at the exact point where their topic begins: positionally
anchored like footnotes, but with no in-text marker glyph and rendered in
the gutter, not at the foot.

The schema had no margin-note concept. Two shapes were considered: notes
as text columns on a new table (cheap, ~3 files), or notes as full
citizens whose bodies are `sentences` rows (the footnote pattern).

## Decision

1. **Full footnote mirror.** A `margin_notes` table (`book_id`, `number`,
   `anchor_sentence_id`) whose bodies are `sentences` rows via a
   `margin_note_id` FK — third arm in `chk_sentence_parent`, a `'margin'`
   `sentence_kind`, natural keys `{source_ref}/mn{number}/s{pos}`, and a
   third book-global `sentence_number` sequence. This inherits everything
   the sentence ecosystem provides for free: the dual orthography layers,
   FTS indexing, quotation anchoring, canonical passages, and
   reconcile-in-place UUID carrying. `number` is a document-order internal
   identity (margin notes are unnumbered in print); reconcile treats it
   exactly like footnote numbers — strictly-additive, anchor-stable.

2. **Curated-MD token: `` {{$m `content`}} ``** placed inline at the anchor
   position. Content is backtick-delimited (quotes and `}}` need no
   escaping; a literal backtick is unrepresentable and hard-errors), and
   the backticks are *delimiters, not code-span semantics* — `_…_`
   emphasis inside note content renders normally. The parser lifts each
   token out of the raw markdown pre-render (its content is markdown and
   cannot ride through rendering), substitutes an inert numbered sentinel
   (`{{$m N}}`), and attaches the note to the sentence the sentinel lands
   in — anchoring is sentence-level; sub-sentence offsets have no
   rendering or citation value. The general convention is
   `` {{$<letter> `…`}} `` = typed special inline input; `{{$` is matched
   before the `{{ }}`/`{{{ }}}` marker families.

3. **`*` stays reserved for footnotes.** The 1651 print's starred margin
   citations are ordinary margin notes here; their in-text asterisks are
   dropped at conversion (token position carries the anchor).

4. **Reader**: notes render in the existing gutter beside their anchor
   sentence (pseudo-system slug `"notes"` in the margin menu, so toggle +
   side reuse the reference-system plumbing). Narrow panels collapse to a
   gutter button + popover — the trigger lives in the gutter, never in the
   text. When notes share a gutter side with page-reference markers, the
   markers fold into the note block instead of overlapping; books carrying
   notes default page references to the left gutter (the 1651 print's own
   arrangement, notes outer).

5. **Not plumbed for translation editions.** Margin notes on a
   `--source-book-slug` edition are rejected at import (no source-sentence
   map, no parity rule) until a corpus needs them.

## Consequences

- The content hash feeds margin notes only when present, so every
  margin-note-free corpus keeps byte-identical stored hashes — the
  capability landed with zero re-import of existing books.
- Anything referencing `sentences.id` must know about the third parent:
  `reconcile::deps` counts and repoints `margin_notes.anchor_sentence_id`
  alongside footnotes; `canonical.rs` mints kind `'margin'`.
- Margin-note sentences are quotable and searchable by construction; the
  quotation-resolution endpoints address them by (`sentence_number`,
  kind) like footnote sentences when the UI grows that wiring.
- hobbes1 exercises the concept end to end; the capability is
  genre-agnostic and available to any future annotated-prose corpus.
