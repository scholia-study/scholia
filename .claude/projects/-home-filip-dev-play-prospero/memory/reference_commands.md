---
name: Project commands reference
description: Key pnpm/turbo commands for the Prospero project - codegen, openapi, db_reset
type: reference
---

- `./db_reset.sh` — drop and recreate the database from db/001_schema.sql
- `pnpm turbo openapi` — regenerate openapi.json from Rust utoipa annotations
- `pnpm codegen` (in web/) — run Orval to generate React Query hooks from openapi.json
- `pnpm ready` — full reset: db:reset + extract + db:import
