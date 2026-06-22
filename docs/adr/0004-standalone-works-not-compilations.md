# 0004. Texts are standalone authored works, not Bible-shape compilations

**Status**: Accepted
**Date**: 2026-06-22

## Context

The Bible is modelled as a *compilation*: one `books` row, with depth-0
source-anchored "work" nodes (Genesis, John, …) that render as library **pills**
plus a grand table of contents. That's a genuine fit — 66 interlinked books, and
translations (KJV/WEB/…) that span the whole corpus.

To get that same pill UX, Shakespeare's Sonnets (2026-06-16) and then Milton's
*Paradise Lost* were built the same way: a `shakespeare` / `milton` book, a
depth-0 source-anchored work node ("Sonnets" / "Paradise Lost") backed by a
`chapter` sub-source, with the actual content as children. The author was
attached to the *work* source, leaving the book an author-less "self" group —
which is precisely what triggers the pill rendering.

That coupled a **frontend presentation (pills) to the DB hierarchy**, and the
coupling generated friction as we built on it: per-book reader pagination
couldn't stay clean if a compilation later mixed long + short works; citation
templates had to lean on a synthetic `{parent}` work-node; the importer carried a
`WorkSource`-anchor path; and an author with a single work got a needless level
of nesting.

## Decision

Model each text as a **standalone authored work** — the shape Kant already uses:

- one `books` row whose own `source_type='book'` source carries the author via
  `source_persons`;
- flat top-level `toc_nodes` (no depth-0 work-wrapper, no `chapter` sub-source);
- the library groups it under its author (work cards); the reader shows its
  native TOC.

Applied to Milton (`paradise-lost`, "Paradise Lost") and Shakespeare
(`shakespeares-sonnets`, "Shakespeare's Sonnets"). **The Bible stays a
compilation** — it genuinely is one.

A pill / grouped presentation for a standalone work, if ever wanted, is a
**frontend/library directive** decoupled from the content hierarchy — not a
reason to nest the schema.

## Reasoning

- Presentation shouldn't dictate schema. The pill UI is a library affordance;
  encoding it as "depth-0 source-anchored node" forced unrelated texts into a
  compilation mold.
- The compilation's real value — a grand TOC of an author's many interlinked
  works, plus corpus-spanning translations — is strong for the Bible and weak for
  one epic or one sonnet sequence; "compare text" already covers cross-work
  access.
- Standalone is simpler end to end: per-book pagination is unambiguous; citation
  templates drop to `{self} · {ref}` (the book title *is* the work); the importer
  links the author to the book source and never touches the `WorkSource` path.

## Consequences

- Slugs/titles changed: `shakespeare` → `shakespeares-sonnets`, `milton` →
  `paradise-lost`; the book title is the work, not the author.
- Reading nodes are flat (`depth = 0`); curated MD front-matter `depth` is `0`.
- Citation reads `{self} · {ref}` ("Paradise Lost · Book I · 42"); migration
  `0008` backfills only books that genuinely pre-exist (Bible/Kant), since the
  poetry books are always imported fresh.
- Pills no longer render for these two — they degrade to normal author groups +
  native TOC. The Sonnets revert to a flat 154-item TOC (the pre-compilation
  shape); reinstating a grouped presentation is future frontend work.
- Migrating existing data: the dev cluster's old `shakespeare` compilation book
  is retired with a scoped one-off delete (an ops task, not a migration — a
  migration would be a permanent no-op on every fresh DB); Paradise Lost is a
  fresh import.

## Re-open this ADR if

A genuine multi-work author hub becomes a goal — e.g. Shakespeare's plays as
work→act→scene siblings under one "Works of Shakespeare". Then revisit whether
the compilation structure, or a frontend grouping directive layered over
standalone works, is the better vehicle. ADR 0003 (verse line = sentence row)
is unaffected and still holds.
