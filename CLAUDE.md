# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Scholia — an interactive reader for primary-source texts (Bible translations, Kant's first Critique, etc.). Rust axum API + React frontend backed by Postgres, plus a family of CLI binaries that ingest source texts into the database.

## Commands

All JS package operations use **pnpm**, never npm (see auto-memory).

- `pnpm dev` — `turbo run dev` starts both API (`cargo run -p api`, port 4000) and web (`vite dev`, port 3000).
- `pnpm validate` — runs `turbo lint ct test check:modules` across the workspace. This is what pre-commit runs.
- `pnpm sanity` (`pnpm s`) — biome format + validate.
- `pnpm codegen` — regenerates the API client. Runs `api#openapi` (Rust binary dumps `openapi.json` at repo root) → orval generates `apps/web/src/api/**` → biome formats. **Run this after any Rust handler/model change that affects the OpenAPI surface.**
- `pnpm db:reset` — runs `scripts/db_reset.sh`, which drops + recreates the `public` schema then applies every migration in `db/migrations/` via `sqlx migrate run`. Always use this script for schema resets, never raw inline psql DROP/CREATE. Requires `cargo install sqlx-cli --no-default-features --features postgres,rustls`.
- `pnpm stripe:listen` — forwards Stripe webhooks to `localhost:4000/api/webhooks/stripe` for local billing dev.

Per-package:
- `pnpm --filter web ct` — typecheck only (`tsc --noEmit`).
- `pnpm --filter web test` — vitest. `pnpm --filter web test -- <pattern>` to filter.
- `pnpm --filter web check:modules` — runs `scripts/check-module-imports.mjs` (see Module Encapsulation below).
- `cargo test -p api` — Rust API tests. `cargo test -p api <name>` for a single test.
- `cargo run -p api --bin openapi` — regenerate `openapi.json` directly without orval.

Asset → DB ingestion (run in order against a fresh schema):
- `pnpm db:reset` — wipe DB and re-apply migrations.
- `scripts/db_bible.sh` — imports KJV, WEB, ASV, BBE, DARBY (KJV must run first; it seeds canonical verse counts).
- `scripts/db_kant1.sh` — imports Kant's Kritik (German + English from `assets/`).
- Or run the post-OCR/markdown pipeline binaries individually — see `README.md` and `dp:kant1` script in `package.json`.

Pre-commit (lefthook): formats staged JS/TS via biome, runs `cargo fmt`, then runs `pnpm validate`. Do not bypass with `--no-verify`.

## Architecture

**Workspace**: pnpm workspace (`apps/web`, `apps/api`) + Cargo workspace (`apps/api`, `packages/*`). Turbo orchestrates the JS side; Cargo handles Rust crate dependencies.

### Backend — `apps/api`

Axum 0.8 + `utoipa-axum` (OpenAPI-as-code). `src/lib.rs` declares the `ApiDoc` with every path/schema; `src/main.rs` composes routers and wires middleware. Five router groups merged into one app:

- **auth_router** — `/api/auth/*`, rate-limited via `tower_governor` (10 req / 60s / IP).
- **user_router** — session-authenticated user actions (quotations, notes, articles, billing checkout/portal).
- **public_router** — books, library, TOC, nodes, page, public articles, public user profiles.
- **editor_router** — resource/source/person CRUD; auth enforced inside handlers (`Permission::*`).
- **admin_router** — feedback queue and article editorial labels.
- **webhook_router** — Stripe webhook lives outside the main app so it bypasses session + CORS layers; only `Stripe-Signature` authenticates it.

Sessions are `tower-sessions` with Postgres backing. `AppState { pool, config, stripe }` is shared via Axum state.

Module layout under `src/`:
- `handlers/` — HTTP handlers, one file per resource. Annotated with `#[utoipa::path(...)]`.
- `db/` — sqlx queries, one file per resource. **DB-layer payload param names: `entry` for creates, `patch` for updates; no destructure block at the top** (see auto-memory).
- `models/` — request/response DTOs (utoipa `ToSchema`).
- `auth/`, `email.rs`, `config.rs`, `state.rs`, `error.rs`, `validation.rs` — cross-cutting.

**sqlx gotcha**: LEFT-JOINed NOT NULL columns need `"col?"` aliasing or `query_as!` will panic at runtime (see auto-memory).

### Frontend — `web`

TanStack Start (SSR/SPA hybrid) + TanStack Router (file-based, codegen → `routeTree.gen.ts`) + TanStack Query. MUI v7 + Tailwind v4 + Emotion. Orval generates `src/api/**` from `openapi.json` with react-query hooks; the `get_node_page` operation uses `useSuspenseInfiniteQuery` with `after` as cursor. All API calls flow through `src/api/fetcher.ts` (`customFetch`) which includes credentials and throws `FetchError` on non-2xx.

Route file conventions in `src/routes/`:
- `_auth.*` — wrapped by an auth-required layout.
- `_auth._admin.*` — auth + admin-permission required.
- `.by-id.$id.tsx` — id → handle/slug redirect resolvers.

**Module encapsulation (`pnpm check:modules`)**: code outside `apps/web/src/modules/<x>/` may only import from `apps/web/src/modules/<x>` (the barrel index), never from internal files of another module. Same-module imports may use relative paths. Enforced by `apps/web/scripts/check-module-imports.mjs` and pre-commit.

Path alias `#/*` → `apps/web/src/*`. The active runtime config is selected per-deployment via `window.__ENV__.APP_PROFILE` (`local` | `local-proxy` | `dev` | `prod`); the Node SSR reads `APP_PROFILE` from its container env at render time and inlines `<script>window.__ENV__ = { APP_PROFILE: "…" }</script>` in `<head>` (see `apps/web/src/routes/__root.tsx`). One image, env-selected profile. **Never put secrets in `apps/web/src/config.ts`.**

### Database — `db/migrations/`

PostgreSQL 18+ with `ltree`. Append-only sqlx migrations named
`NNNN_<semantic-name>.sql` (sequential, starting at `0000`). Embedded
into the api binary via `sqlx::migrate!`; the cluster init container
runs `api migrate`. Dev resets via `pnpm db:reset` (uses `sqlx-cli`).
~35 tables grouped into: users + auth (users, sessions, oauth, password reset, email verification, released_handles), bibliography (persons, sources, books, source_persons, resources), text content (toc_nodes, content_blocks, sentences, footnotes, page_markers, facsimile_pages, reference_systems, cross_references, **cross_translation_alignments** — see `docs/architecture/cross-translation-alignment.md`), user content (quotations, articles, article_quotations, quotation_notes, tags, topics, editorial_labels, feedback), and billing (subscriptions, stripe_processed_events).

### Rust packages (ingest CLIs)

- `packages/common` — shared parsers (epub, ncx, opf, kant1, sentences, content).
- `packages/bible_to_db` — `--translation kjv|web|asv|bbe|darby`.
- `packages/kant1_*` — multi-stage pipeline: OCR → lines → elements → MD → modernized/translated MD → struct → DB. See `README.md` and `assets/kant1/`, which splits into three tiers: `raw/` (pre-curation pipeline outputs — gitignored), `curated/` (human-reviewed MD — tracked), `derived/` (struct JSONs auto-generated from curated MD — gitignored).

### Docs

- `docs/adr/` — architectural decisions (e.g., 0001 says: don't extract a shared form-modal primitive — keep the four modals independent).
- `docs/architecture/cross-translation-alignment.md` — how quotation projection follows content, not verse numbers, across translations. Read this before touching `apps/api/src/db/quotations.rs`.
- `docs/architecture/database-migrations.md` — sqlx migration setup, why dev uses sqlx-cli while prod uses `api migrate`, and the chicken-and-egg that forces the split. Read this before touching `db/migrations/`, `apps/api/src/migrate.rs`, or `scripts/db_reset.sh`.

## Formatting

Biome 2.4.5, 4-space indent, 80-col width, double quotes, trailing commas, semicolons. `cargo fmt` for Rust. Both run on staged files at commit time.
