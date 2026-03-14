---
name: codegen-and-pnpm
description: Use pnpm (not npx/npm) and run codegen via turbo from project root
type: feedback
---

Use `pnpm` for all JS/TS tooling, not `npx` or `npm`. Run codegen using `pnpm turbo codegen` from the project root, not orval directly from web/.

**Why:** The project uses pnpm as its package manager and Turborepo with a `codegen` task that chains `api:openapi` → `web:codegen`.

**How to apply:** After modifying API models or OpenAPI schema, run `pnpm turbo codegen` from the repo root. Use `pnpm` for all node commands.
