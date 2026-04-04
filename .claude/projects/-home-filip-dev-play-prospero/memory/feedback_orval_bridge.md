---
name: Always use Orval for API bridge
description: Always use OpenAPI spec and Orval to bridge Rust backend with React frontend - never hand-write API clients
type: feedback
---

Always use OpenAPI (via utoipa) and Orval to bridge the Rust backend with the React frontend. Never hand-write API client code.

**Why:** The project has an established codegen pipeline: Rust utoipa annotations -> openapi.json -> Orval -> React Query hooks. Hand-written API clients bypass this and create inconsistency.
**How to apply:** When adding new backend endpoints, annotate with utoipa, run `pnpm turbo openapi` to regenerate spec, then `pnpm codegen` (in web/) to generate hooks. Use the generated hooks in frontend code.
