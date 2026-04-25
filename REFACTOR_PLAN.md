# Refactor Plan — `web/`

Deepening opportunities in the web app, ranked by leverage. Vocabulary follows the architecture skill: **module** (interface + implementation), **seam** (where an interface lives), **adapter** (concrete thing satisfying a seam), **shallow** (interface as complex as implementation), **deep** (high leverage at small interface), **locality** (complexity concentrated in one place).

## 1. Reader as a deep module

**Files**: `components/ReaderLayout.tsx` (631), `TextPanel.tsx` (949), `PanelScrollView.tsx` (706), `BlockRenderer.tsx`, `InterleavedNodeRenderer.tsx`, `ResourcesPanel.tsx` (640), `CommentaryView.tsx`, `SentenceSelectionContext.tsx`, `FootnoteSelectionContext.tsx`, `QuotationContext.tsx`, `routes/books.$bookSlug.$nodeSlug.tsx`.

**Problem**: ~3,000 LoC across 10+ files form one feature with no single interface. Selection state is split across two parallel contexts (Sentence and Footnote) with different shapes. State flows URL → route parses Maps → ReaderLayout → TextPanel local state → SelectionContexts → PanelScrollView (via imperative ref) → BlockRenderer (reads contexts) → user click → callback up to TextPanel → callback up to ReaderLayout → encode to URL → navigate. Each piece is shallow: TextPanel's interface is 12 props + 9 callbacks + 3 contexts — almost as complex as its body. Adding one panel property touches 6 files. Tests can't exercise "click range, observe resources panel" without mocking ~12 hooks.

Deletion test: deleting `CommentaryView` / `SentenceDetail` / `BlockRenderer` individually doesn't concentrate complexity — bodies are mostly rendering, logic moves around. But unifying selection + panel state + URL codec behind one seam does concentrate complexity meaningfully.

**Solution sketch**: form a Reader module that owns URL↔state, selection, pagination, and mutation orchestration. Internal pieces consume one shared state seam instead of context drilling and ref callbacks. The route becomes a thin caller.

**Benefits**: locality — selection, panel state, URL serialization converge. Leverage — new panel property or selection kind is a one-file change. Tests — module exposes a single state surface; "click range in panel 2" is a unit test, not an integration test against the DOM.

## 2. URL state codec for the reader

**Files**: `routes/books.$bookSlug.$nodeSlug.tsx:6-170`, `components/ReaderLayout.tsx:23-125`.

**Problem**: 42 abbreviated query keys (`p2-p4`, `s-s4`, `r-r4`, `og-og4`, `rv-rv4`, `vm-vm4`, `vl-vl4`, `vt-vt4`, `fs-fs4`) split between encode (`buildSearch`) and decode (`parsePanel` + unpacking in `ReaderPage`). TypeScript can't enforce that a panel's nine keys travel together. Adding a 5th panel needs 9 new keys plus parallel edits in both files.

Encode/decode is one mapping expressed as scattered procedural code — a real seam that isn't named. Deletion test: deleting either half leaves the other unusable.

**Solution sketch**: collapse encode/decode into one codec module owning the URL representation. Wire format becomes an implementation detail — could shift to JSON without touching callers.

**Benefits**: locality — one place for the URL scheme. Leverage — round-trip becomes a module invariant. Tests — encode→decode property tests with no rendering.

(Likely subsumed by #1, but a tight standalone refactor on its own.)

## 3. Form modal as a real seam

**Files**: `NoteFormModal.tsx` (186), `PersonFormModal.tsx` (134), `ResourceFormModal.tsx` (529), `SourceFormModal.tsx` (590).

**Problem**: four adapters duplicate the same shape — Dialog shell, field-state binding, mutation orchestration, success toast + cache invalidation, error toast + FetchError mapping, create/edit mode switching. Two adapters would be a hypothetical seam; four is a real one being hand-copied.

Deletion test on any single modal: complexity moves to the caller. Deletion test on the pattern: replacing all four with one parameterized module concentrates mutation orchestration, error mapping, and dialog behaviour in one place — complexity does not reappear N times.

**Solution sketch**: one form-modal module owns the shell, mutation lifecycle, toast + invalidation, error mapping. Each existing modal collapses to an adapter that supplies fields and a mutation. Special cases (Source's nested person creation, Person's auto-derived sortName) become small extensions, not copies.

**Benefits**: locality — one place for "how a form modal behaves." Leverage — fifth form, or behaviour change like cancel-during-mutation, is a one-place edit. Tests — modal tested once against a fake mutation.

## 4. Auth flow module

**Files**: `hooks/useAuth.ts`, `routes/login.tsx` (152), `register.tsx` (150), `reset-password.tsx` (124), `forgot-password.tsx`, `verify-email.tsx`.

**Problem**: each auth route hand-rolls local state, mutation hook, `FetchError.status` → message mapping (`401`/`403`/`409`/`422`), navigate, `queryClient.invalidateQueries`, conditional UI on query params. Status-code mapping is duplicated inconsistently. Backend status-code changes require editing every route.

Five adapters of an unwritten module.

**Solution sketch**: pull state machine, error→message map, post-success navigation, and cache invalidation into one auth-flow module. Routes become thin views that pick a flow and render fields.

**Benefits**: locality — auth error messages and post-success behaviour concentrate. Leverage — adding 2FA or a new provider is a one-module change. Tests — flow logic testable without rendering.

## 5. Routes shrink to wiring once features are modules

**Files**: `routes/user.sources.$id.tsx` (816), `routes/user.articles.$slug.tsx` (508), `user.articles.index.tsx`, `user.quotations.tsx`, `user.notes.tsx`.

**Problem**: routes contain feature implementations, not URL→render wiring. `user.sources.$id.tsx` embeds metadata form, contributors block, person edit modal, references panel — 5 sub-features in one file. Route's interface (URL params) is small but implementation is the whole feature pinned to a URL.

**Solution sketch**: each fat route extracts a feature module (e.g. `SourceDetail`, `UserArticleEditor`). Routes become parameter-extraction shells (~30 lines).

**Benefits**: locality — features named after themselves, not URLs. Leverage — features callable from elsewhere (modal, sidebar). Tests — feature testable without router.

## 6. Confirmation-dialog primitive

**Files**: `hooks/useArchiveArticleDialog.tsx` (79), `usePublishArticleDialog.tsx` (76), `useUnsaveQuotation.tsx` (118).

**Problem**: three hooks repeat the same shape — `target` state, `openFor(item)` setter, dialog rendered when target ≠ null, async confirm running mutation + clearing target, optional pending state. Differences are copy and one conditional warning.

**Solution sketch**: one confirm-dialog hook owns target state, open/close, async confirm with `isPending`, cleanup. Each existing hook becomes a thin call.

**Benefits**: locality, leverage — ESC handling, focus, double-click prevention become one-place changes. Tests — confirm/cancel/error tested once.

## Not flagged

The `api/` layer is Orval-generated pass-through, but `fetcher.ts` centralizes credentials + `FetchError` and the per-resource files are organized boilerplate, not shallow modules masquerading as depth. Leave it.
