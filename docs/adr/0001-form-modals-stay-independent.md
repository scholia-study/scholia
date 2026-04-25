# 0001. Form modals stay independent

**Status**: Accepted
**Date**: 2026-04-25

## Context

The codebase has four form modals — `NoteFormModal`, `PersonFormModal`,
`ResourceFormModal`, `SourceFormModal` — sharing a common shape: Dialog
shell, mutation orchestration (create + optional edit), toast on
success/error, query-cache invalidation, and a Cancel/Submit button row.
Per modal, ~40–80 lines of boilerplate repeats. Across the four,
~200–300 lines of duplication.

A `<FormModalShell>` + `useFormModal()` primitive was considered: each
modal would collapse to a shell call plus its fields, with mutation
glue centralized.

## Decision

Keep the four modals as independent files. Do not extract a shared
form-modal primitive.

## Reasoning

The modals don't share state or coordination — each one is a
self-contained interaction with its own resource. The duplication
is the cost of independence, which is acceptable.

A primitive would buy indirection more than locality: the part worth
centralizing (Dialog shell + mutation + toast + invalidation) is small
enough that copying it is cheaper than the abstraction tax of a shared
module that callers must conform to. Field rendering — which
dominates the file size — isn't duplicated; only the shell is.

## Consequences

- Drift between modals (inconsistent error mapping, inconsistent
  reset-on-cancel behaviour) is accepted; fix per-modal as it surfaces.
- A future architecture review should not re-propose unifying these
  modals on the basis of shared shape alone.
- Re-open this ADR if: a fifth modal forces the same boilerplate
  again, cross-modal behaviour (consistent error UX, consistent
  cancel-during-mutation handling) becomes a real requirement, or
  validation/schema concerns push toward a shared field-config
  primitive.
