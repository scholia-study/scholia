# Big Books — Bible UX Plan

Tracking the redesign of how compilation-style multi-translation works (the Bible, eventually Q'uran, Mishna, etc.) are presented in the library and read in the reader.

## Locked decisions

### Q1 · Library entry shape — option (c)
Heading **The Bible** with primary book pills `[Genesis] [John] ...` and a small subtle translation chooser alongside. Books are the primary navigation; translation is a preference.

### Q2 · Translation persistence — option (b)
- localStorage on the device, key holds the user's preferred Bible translation.
- WEB is the default.
- Only the **library** chooser writes to localStorage. The reader's translation switcher is ephemeral within that session — comparing translations mid-read does not change your default.

### Q3 · Book pill landing — full-TOC page with fragment shortcut
Clicking a book pill from the library lands on the book's full TOC page at `/books/<active-translation>#<book-slug>` (e.g. `/books/kjv-bible#genesis`). The TOC page lists every Bible-book vertically, each with its chapter pills inline; the URL fragment scrolls to the chosen book on mount. Same pattern Kant's TOC page uses (`/books/<bookSlug>/`), so users follow one mental model.

**Earlier attempt** (option (c) in the original grilling) was to land on a per-Bible-book chapter-picker at `/books/<translation>/<book-node>` and inline chapter pills below the heading. That collapsed two things: it cluttered the reader with navigational chrome next to the verse text, and it forced a two-step click to reach a chapter from the library. Replaced.

### Q4 · TOC redesign — split sidebar vs full-page behavior
**Sidebar TOC** (inside the reader): 2-level pills, always visible. Book pills change which book's chapters are visible **locally** — they do NOT navigate. Only chapter pills navigate. Selected-book state initializes from the active node's containing book and follows the read position when it changes externally.

**Full TOC page** (`/books/<bookSlug>/`): every Bible-book listed vertically with its chapter pills inline. Each book section carries `id={book.slug}` so URL fragments (`#genesis`, `#john`) scroll directly to the section on mount. Click a chapter pill to enter the reader.

This split replaces the earlier "inline chapter pills next to the reader content" prototype, which felt cluttered next to verse text.

### Q5 · Library API shape — option (b)
Add an optional `book_pills?: [{node_slug, label, sort_order}]` field to `LibraryGroup`. Backend computes it for "Bible-shape" groups by enumerating one representative translation's depth=0 toc_nodes (assumes all translations are structurally parallel). Frontend renders pills + a single subtle translation chooser using the existing `LibraryWork.versions` payload (no second translation field).

Importer constraint to add: refuse import if the new translation's depth=0 node slugs disagree with the existing canonical translation. Cheap to enforce, painful to debug otherwise.

### Q6 · Reader-level translation switcher — option (b)
For books where the root source has **no** `translation_of_id` (no source language) but **has** sibling translations, the reader's view-mode menu collapses to a flat translation picker. Side-by-side comparison still available via the existing `companionSlug` mechanism (max 2 per panel; multi-panel works for 3+ comparisons).

### Q7 · Quotation/note semantics across translations — verse-level visual marker
Saved quotations stay locked to their original translation — the row continues to anchor `book_id + sentence_id`, citations honor the saved-from translation, and you cannot "flip" a quotation to point at a different translation's text. This is intentional: KJV and WEB segment verses into different numbers of sentences, so the saved sentence-id has no clean equivalent.

What IS projected across translations is the **visual marker only**: when reading WEB Genesis 5:1, if you previously saved any quotation in KJV Genesis 5:1, the saved-icon shows on the WEB verse. It's a "you've already saved something in this verse" hint — clicking the WEB sentence still saves a brand-new WEB-anchored quotation; the KJV quotation is unaffected.

**Implementation**:
- `list_quotations_for_node` returns own-book quotations PLUS peer-translation quotations whose anchor toc-node shares the source_ref and translation root. Each row carries `anchor_source_ref` + `anchor_verse_start` / `_end` (from `page_markers` joined on the `verse` reference system) for verse-key matching, and `book_slug` + `translation_label` for the badge UI.
- `QuotationContext.isSentenceSaved` keys on `${source_ref}::${verse_ref_value}`; falls back to sentence_number range match for Kant (no verse markers).
- A saved-from translation badge ("KJV"/"WEB" for Bible, "DE"/"EN" for Kant) renders next to the saved icon, in My Quotations rows, and in My Notes rows. Backend computes the label as `publisher` (when short ≤6 chars) else `UPPER(language)`.

### Side-by-side alignment — sentence-link mode + marker mode
`InterleavedNodeRenderer.alignSentences` now tries two strategies in priority:
1. **sentence-link** (Kant) — when companion sentences carry `source_sentence_start_id`, group by the linked primary sentence. Same precision as before.
2. **marker** (Bible) — auto-detected when both sides share at least one reference-system slug (verse for the Bible). Sentences group by their first `ref_value` for that system, then groups pair across primary/companion. KJV Gen 5:1 (2 sentences) renders alongside WEB Gen 5:1 (1 sentence) as one verse pair.
Fallback: degenerate "primary as-is, companion appended" if no alignment hint exists.

User-visible alignment toggle deferred — auto-detection from data is correct in every case we have today.

## Deferred decisions

### Translation-locked quotations — Q7 option (c)
If textual-criticism scholars eventually need quotations that are explicitly tied to one translation's wording ("the KJV's archaic 'thee' here is what I'm noting"), add a per-quotation `translation_locked` flag and a UI toggle. v1 ships pure (b). Revisit when the use case actually shows up; do not pre-build.

### Selection carry across translation switch — Q9 option (c)
The reader's flat picker for Bible-shape books currently lands on the equivalent chapter (resolved by `source_ref`) but drops sentence selection. Pure (c) selection-carry needs:
- Extend `onViewModeChange` to thread an optional target sentence ID through `ReaderLayout`'s navigate (currently only book + node slugs travel).
- Click-time pre-fetch of the target node (or rely on already-loaded companion data when the user is in side-by-side mode) to resolve the equivalent sentence by `(source_ref, position)`.
- Or, alternatively: encode `?sn=<sentence_number>` as a fallback URL param, drop the stale `s` ID, and have the target reader resolve by sentence_number on mount. Cleaner — verse-count parity (enforced at import) guarantees `sentence_number` aligns across translations for Bible-shape works.
v1 ships without it; user re-selects after switching translations.

### Q8 · Pill grouping at scale — option (a), deferred
Flat wrap of all book pills (66 at full ingest) in canonical order (Genesis → Revelation). No OT/NT grouping in v1. Revisit if visual scan becomes a real problem; the API shape stays `book_pills: [...]` (ungrouped) so adding `book_pill_groups` later is a backward-compatible addition.

### Q9 · Scroll/position preservation across translation switch — option (c)
Selection-anchored. If a sentence is selected when the user switches translations, the equivalent sentence (mapped by `source_ref + position`) is selected and scrolled into view in the new translation. If nothing is selected, the user lands at the top of the equivalent chapter. Pure viewport-centered alignment (option b) is deferred — requires intersection-observer tracking we haven't built. (c) is zero new infra and the gesture "click verse, then switch" is already in the muscle memory.

### Cross-translation parity — verse-level, multiple sentences per verse
The Bible has no hosted source language we point at — both KJV and WEB are translations from Hebrew/Greek we don't host. We anchor cross-translation features on the universal versification convention:

- Each verse can hold **one or more grammatical sentences**. The importer segments verses on `[.!?]` followed by whitespace + uppercase letter (or end-of-text); `;` and `:` are treated as internal pauses, not sentence breaks. Tested in `bible_to_db::tests`.
- `toc_nodes.source_ref` (`genesis:1`) is identical across translations.
- Each `sentences` row carries a verse `page_markers` entry with `ref_value = "C:V"`. Verse identity is preserved per-sentence even though one verse maps to N sentences.

`sentence_number` does **not** align across translations any more (different segmentation = different counts). Cross-translation lookups must match on `(source_ref, verse ref_value)`, not on `sentence_number`.

**Importer guard**: when ingesting a new translation, the importer compares per-chapter **verse counts** (distinct `page_markers.ref_value` for the canonical translation's chapter) against the new translation's verse count from the source JSON. Mismatch → refuse import. Catches versification drift before it ships.

## Deferred / not pressing yet

- Library translation chooser visual style (pill toggle / dropdown / underlined links).
- Detection rule wording for "Bible-shape" books (compilation with translations, no `translation_of_id` on root source). Resolved during implementation.
- How quotation citations render when saved-from translation differs from currently-reading translation.
- Side-by-side: behavior when the *primary* translation is switched while a companion is active (does companion follow, stay, or swap?).
- Pure viewport-aware translation switch (Q9 option b) — needs intersection-observer tracking.
- Mobile pill wrap vs horizontal scroll behavior.
- Search behavior across translations.
- Generalization to Q'uran (translations), Mishna (commentaries / multi-tractate), other big books.
