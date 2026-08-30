# 0013. Per-essay bibliographic sources on essay collections

**Status**: Accepted
**Date**: 2026-08-30

## Context

peirce1 (*Essays in Pragmaticism*) is our own edition of sixteen papers
that were each published independently, in five different periodicals,
across four decades. A quotation from "The Fixation of Belief" was
bibliographed as the volume — "Peirce, Charles Sanders. (1867) 2026.
*Essays in Pragmaticism*. Scholia Sodalitas." — which hides the paper
actually being cited and its real imprint (*Popular Science Monthly* 12,
1877).

The mechanism for sub-work identity already existed: the Bible anchors a
'chapter'-type source on each Bible-book's `toc_node.source_id`, and
`quotations.source_id` is denormalized from exactly that ancestor walk
(`corpus::core::resolve_effective_source`). What was missing was (a) a
way for a struct-pipeline corpus to declare per-node sources with a
periodical imprint, and (b) the article bibliography consuming them.

## Decision

**`text_struct::NodeSource` carries the imprint** — `journal_name` and
`volume` alongside `title` and `publication_year` (serde-skipped when
absent, so every other corpus's struct JSON is byte-identical). The
parser fills it from a per-corpus hook (`Corpus::node_source`); peirce1's
reads `common::peirce1::meta::PAPER_IMPRINTS`, a 16-row table parallel
to the TOC.

**The importer materializes each as a protected 'chapter' source** —
parented to the book's bibliographic source, author-linked, upserted on
the `(title, source_type, publication_year)` unique so a re-run reclaims
the existing row instead of colliding. Because node sources are *not*
part of the reconcile content hash, a sync pass in the reconcile branch
upserts and links them on every import, before the root-hash
short-circuit — this is how books imported before this change (and
imprint corrections later) reach existing deployments without
`--replace`.

**The article bibliography resolves quoted passages to the sub-work** —
from each `::quotation{book= node=}` directive, the deepest toc-node
ancestor whose chapter source **carries a `journal_name`** wins; anything
else falls back to the book's source as before. The imprint gate is the
scope guard: per-Bible-book chapter sources have no imprint, so Bible
quotations keep citing the whole translation — this change may not
silently rewrite existing bibliographies.

**Rendering** uses Chicago's journal-article form when `journal_name` is
set: `Peirce, Charles Sanders. 1877. "The Fixation of Belief." <em>Popular
Science Monthly</em> 12.` — quoted title, italicized journal, volume, no
publisher/place/edition apparatus. All other sources keep the book form.

## Consequences

- Declaring per-essay sources is data-only per corpus: an imprint table
  plus a `node_source` hook arm.
- `--replace` on a book with chapter sub-works used to die on
  `chk_chapter_has_parent` (the parent delete fired the FK's SET NULL);
  the replace path now deletes chapter children before their parent.
- A chapter title or year change mints a new source row (they are the
  identity key); the node is re-pointed, the old row lingers until
  cleaned up manually. Imprint fields (`journal_name`, `volume`,
  re-parenting) follow the curated data on every import.
- Verified on scratch DBs: fresh 16-paper import, no-op re-import,
  `--replace` + re-import, mid-book insertion (ADR 0012 path) with zero
  sentence-UUID churn, legacy-book backfill on a root-hash no-op run,
  and a hobbes1 control import — all clean; struct JSON byte-identical
  for the nine other corpora.
