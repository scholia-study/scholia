# Refactor Plan — `web/`

Deepening opportunities in the web app, ranked by leverage. Vocabulary follows the architecture skill: **module** (interface + implementation), **seam** (where an interface lives), **adapter** (concrete thing satisfying a seam), **shallow** (interface as complex as implementation), **deep** (high leverage at small interface), **locality** (complexity concentrated in one place).

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
