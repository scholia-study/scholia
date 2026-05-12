# 0002. Feature-coherent files may be long

**Status**: Accepted
**Date**: 2026-05-12

## Context

Architecture reviews on this codebase repeatedly surface large files as
candidates for splitting — `db/articles.rs` (1594 lines), `db/quotations.rs`
(1009), `handlers/auth.rs` (876). In each case the proposed split would
move related-but-distinguishable concerns into separate files: e.g.
extracting the ~550-line citation/markdown rendering pipeline out of
`db/articles.rs`, or splitting `handlers/auth.rs` into separate files for
session lifecycle, credential recovery, and profile management.

The reviewer's instinct is that smaller files are easier to navigate.
The maintainer's instinct is that scrolling through one coherent file is
easier than jumping across three.

## Decision

Long files are acceptable when their contents cohere around a single
feature. Do not split a module on size alone. Split only when there is
a **real seam** — independent reuse, a distinct test surface, or a
separate lifecycle that the current single-file layout obscures.

## Reasoning

Splitting an internally-coherent feature into multiple files trades
**locality of the feature** for **file-level locality of sub-concerns**.
The cost is real: every reader who wants to understand "how does this
feature work" now navigates N files instead of scrolling one.

The citation renderer inside `db/articles.rs` was the worked example.
Its functions (`render_article_markdown`, `fetch_citation_data`,
`format_inline_citation`, `format_bibliography_entry`, …) form a
coherent rendering pipeline, but they are only ever invoked from three
article-fetch functions in the same file. There is no independent
reuse, no separate test surface, and no distinct lifecycle — splitting
would have produced two files that always change together, with the
reader paying a navigation tax for nothing.

Size is a *symptom* of feature complexity, not a *cause* of friction.

## When splitting *is* warranted

- **Independent reuse**: a second feature starts calling the same
  internal code — the seam is now real, not hypothetical.
- **Distinct test surface**: the subset has invariants that want
  isolated tests the larger feature can't provide.
- **Separate lifecycle**: the subset is updated, versioned, or
  reasoned about on a different cadence than its host.
- **Cross-cutting concern**: the code is being copied between
  features (e.g. note/tag machinery duplicated across
  `db/quotations.rs` and `db/article_quotations.rs`) — the duplicate
  itself is the seam.

If none of these apply, leave the file alone.

## Consequences

- Architecture reviews must not list "this file is N lines" as a
  finding. The deletion test and seam criteria above are the bar.
- Files of 500–2000 lines are normal for non-trivial features and
  should not be flagged absent one of the criteria above.
- This ADR does not forbid all splitting — it forbids size-driven
  splitting. Real seams remain fair game.
- Re-open this ADR if: a file grows past the point where editor
  search and AI navigation degrade noticeably; or a pattern emerges
  where N near-duplicate large files would clearly benefit from a
  shared core (the cross-cutting case above).
