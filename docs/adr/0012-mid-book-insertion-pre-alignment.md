# 0012. Mid-book insertion via opt-in numbering pre-alignment

**Status**: Accepted
**Date**: 2026-08-30

## Context

Reconcile-in-place is strictly additive: growth may only append. That invariant
comes from the book-global display sequences — `paragraph_number`,
`figure_number`, and footnote/margin-note numbers — which an insertion shifts
for everything after it. The pre-flight reads any such shift as structural
drift and aborts to `just db-reload`, which re-mints sentence UUIDs and breaks
anchored quotations.

peirce1 (*Essays in Pragmaticism*) is a growing selection: its position numbers are
spaced a thousand apart precisely so papers can be inserted in date order.
Spacing solves the *identity* half — `source_ref` and body-sentence natural
keys (`{source_ref}/b{pos}/s{pos}`) never involve the global counters, so
insertion is safe in principle. The counters were the remaining blocker.

## Decision

**An opt-in pre-alignment pass (`reconcile::insertion`), gated by
`struct_to_db --allow-insertion`, set per corpus in its `scripts/ingest.sh`
arm** (peirce1 only). The default stays strict.

The flag is a safety interlock, not bureaucracy: for a fixed corpus a mid-book
structural addition is *always* an accident (a stray curated file, TOC drift),
and the strict abort is the corruption detector. Whether a book may grow
mid-volume is per-corpus editorial knowledge, and `ingest.sh` is where that
knowledge already lives (`--source-book-slug` precedent).

The pass runs inside the reconcile transaction, after the root-hash
short-circuit, and only when an added node actually sorts ahead of an existing
one (pure appends take the strict path untouched). It rewrites the stored
numbering of existing rows to the desired values, so the unmodified
strictly-additive pipeline then sees a plain append:

1. **Blocks** keep identity by (node `source_ref`, position); their
   paragraph/figure numbers are set to the desired values — offset the whole
   sequence first, then assign, because both columns carry partial unique
   indexes.
2. **Footnotes/margin notes** keep identity by document order: existing and
   desired notes in non-added nodes are paired 1:1 in order, anchors must agree
   pairwise (a mismatch aborts — that is drift, not insertion), numbers are
   rewritten via the pairing, and the note sentences' natural keys (which embed
   the number, `fn{N}`/`mn{N}`) are rewritten with them.
3. **Anchor-sentence HTML** — which bakes in footnote reference numbers — is
   not touched by the pass: it is hashed content, so the shifted nodes flow
   through the normal edit path, which updates the HTML while carrying the
   sentence UUIDs.

`sentence_number` needs nothing new: the existing set-based global renumber
already reassigns it whenever counts change.

## Deliberate limits

- **Translation editions are refused** (`--allow-insertion` +
  `--source-book-slug` errors): they are sentence-locked 1:1 to a source.
  Insert into the source first; translation support is a separate change.
- **One run may not combine a mid-book insertion with notes added to existing
  nodes** — the document-order pairing would be ambiguous. The pass aborts
  with instructions to land them as two runs.
- Renumbering display sequences on insertion is *correct*, not a side effect:
  paragraph ¶N and footnote numbers are document-order presentation, and a
  reader-visible shift is inherent to inserting a chapter. Citations anchor to
  original-printing page markers, which never move.

## Verification (scratch DB, 2026-08-30)

Imported peirce1 minus its 4th paper (15 papers), then inserted it: without
the flag the import aborts exactly as before; with it, all 4,164 pre-existing
body-sentence UUIDs and natural keys survive, paragraph and footnote sequences
end dense and document-ordered, no stale natural keys or anchor HTML remain, a
re-import short-circuits on the root hash, and the resulting rows are
block-for-block identical to a fresh 16-paper import.
