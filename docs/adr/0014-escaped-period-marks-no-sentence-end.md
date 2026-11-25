# 0014. `\.` in curated markdown marks a period that ends no sentence

**Status**: Accepted
**Date**: 2026-11-25

## Context

Sentence boundaries are found by heuristic: a `.`, `!` or `?`, then
whitespace, then a capital or an opening quote, unless an abbreviation list,
an initial, an enumerator label or a corpus knob (`paren_protected_splits`,
`reference_protected_splits`, …) vetoes the candidate. The heuristics get the
common cases right. The residue is cases no rule can settle without reading
the sentence:

- abbreviations that sometimes end a sentence and sometimes don't
  (`u.s.f. Stoff` vs. `… Götter u.s.f. Zu dem erstern Satze`);
- abbreviation chains in citations (`Hrn. Th. S. 119. ff.`,
  `vergl. Goethe zur Naturw. I. Bd. 1. Hft. S. 28.`);
- enumerated list items that continue one sentence (`I. … II. … III. …`,
  `a. … b. … c. …`).

Widening an abbreviation list moves every corpus that uses it, and a
per-corpus ruling table keyed by offset or context sits apart from the text
and drifts from it when the MD is edited. The curated MD is the reader's
single source of truth, so the ruling belongs in it.

## Decision

**In curated markdown, `\.` is a period that ends no sentence.** The
reviewer writes it at the exact site, in every layer whose split is wrong
(for a two-layer source edition, both layers; a translation only where its
own split is wrong).

The splitters (`common::sentences`, every `*_forced` variant) honour it:

1. Split candidates are found on the text with the escape removed, so every
   guard sees the real period. Hiding it would switch guards off: the
   reference protection keeps `IV. Ep. I. 118.` together only because it sees
   `Ep.` as an abbreviation.
2. Any non-forced boundary whose terminal period, looking back past
   whitespace, a Gedankenstrich and closing quotes/parens, was escaped is
   dropped. Forced (structural) boundaries stand.
3. An escape that suppresses no boundary panics the build with its context
   (`` `\.` after "…" suppresses no sentence split; remove the escape ``).
   A stale escape can't accumulate silently, and an escape on a split caused
   by something other than the period (a quote opening `»Est`) is rejected.

The parser strips the escape from stored block text and HTML and shifts
page-marker offsets accordingly; nothing downstream sees a backslash.

## Consequences

- Text without `\.` splits exactly as before. Adding the convention changed
  no existing corpus output.
- Adding an escape to an already-imported book is a sentence *merge*.
  Reconcile handles it in place: one structural change per block per import,
  the merged sentence keeps the first UUID, dependents of the second are
  repointed. Several merges in one block go in separate waves. Article
  `::quotation` directives resolve by book-global `sentence_number`, which a
  merge shifts for everything after it in the book.
- The escape fixes one site. A defect that repeats across a corpus still
  calls for a rule or knob in the splitter.
