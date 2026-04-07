# Article Builder — Implementation Summary

## Backend (Rust/Axum)

- **Schema**: `article_status` enum, `topics`, `articles`, `article_topics` tables with indexes and 10 seed topics
- **Models**: All request/response types for articles, topics, batch sentences, limits
- **Permissions**: `ArticlesCreate`, `ArticlesPublish` added to all roles
- **DB layer**: Full CRUD, slug generation with collision handling, `pulldown-cmark` markdown-to-HTML rendering with quotation directive-to-placeholder conversion, tier-based limits (10/3 free), topic management (max 5)
- **Handlers**: 11 endpoints covering CRUD, publish/unpublish/archive, public listing, topics, batch sentence fetching
- **Routes**: All registered in `main.rs` and OpenAPI `lib.rs`

## Frontend (React/TypeScript)

- **API client**: Auto-generated `articles.ts`, `topics.ts`, `sentences.ts` hooks via Orval
- **Navigation**: "Articles" in UserSubnav, "My Articles" in Navbar menu
- **`/user/articles`**: Management page with status tabs, limits display, create dialog, publish/unpublish/archive actions
- **`/user/articles/$slug`**: Full editor page with:
  - Editable title, description, topic selector (Autocomplete, max 5)
  - **Milkdown v7 editor** with commonmark + history + listener plugins
  - **Custom quotation directive node** (`:quotation{...}`) rendered as atom blocks via ProseMirror NodeView using the shared `QuotationCard` component
  - **Quotation picker modal** with book filtering, quotation selection, display mode + layout options
  - "Insert Quotation" toolbar button
  - Auto-save with 1.5s debounce, save status indicator
  - Publish/unpublish controls
- **`/articles`**: Public listing with topic chip filtering and pagination
- **`/articles/$slug`**: Published article view with `html-react-parser` hydrating quotation embed divs into `QuotationCard` components
- **`QuotationCard`**: Shared component supporting source/translation/both modes, stacked/side-by-side layouts, with link back to source text

## New files created

- `packages/api/src/models/article.rs`
- `packages/api/src/db/articles.rs`
- `packages/api/src/handlers/articles.rs`
- `web/src/components/QuotationCard.tsx`
- `web/src/components/editor/MilkdownEditor.tsx`
- `web/src/components/editor/quotation-plugin.ts`
- `web/src/components/editor/QuotationNodeView.tsx`
- `web/src/components/editor/QuotationPickerModal.tsx`
- `web/src/routes/user.articles.index.tsx`
- `web/src/routes/user.articles.$slug.tsx`
- `web/src/routes/articles.index.tsx`
- `web/src/routes/articles.$slug.tsx`

## Existing files modified

- `db/001_schema.sql` — appended articles/topics schema
- `packages/api/Cargo.toml` — added pulldown-cmark, slug, regex
- `packages/api/src/auth/permissions.rs` — added ArticlesCreate, ArticlesPublish
- `packages/api/src/models/mod.rs` — registered article module
- `packages/api/src/db/mod.rs` — registered articles module
- `packages/api/src/handlers/mod.rs` — registered articles module
- `packages/api/src/main.rs` — registered all article routes
- `packages/api/src/lib.rs` — registered OpenAPI paths and schemas
- `web/package.json` — added Milkdown, prosemirror-adapter, remark-directive
- `web/src/components/UserSubnav.tsx` — added Articles nav link
- `web/src/components/Navbar.tsx` — added My Articles menu item
