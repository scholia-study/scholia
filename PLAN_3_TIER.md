# Scholia — 3-Tier Plan (nginx-cache + Node SSR + Rust API)

Decision and execution plan for moving Scholia off the static-prerender model
into a runtime-SSR architecture with an HTTP cache in front. Distilled from
the design conversation on 2026-05-13.

---

## Why we're moving

The static-prerender approach (TanStack Start SPA + prerendered HTML) hit a
wall of edge cases that aren't worth chasing further:

- The chapter reader uses Virtuoso (virtualized scroll); making it
  render correctly during prerender required `initialItemCount` hacks
  and produced hydration races.
- Reducing the dehydration cache (target chapter only) created scroll
  leap on TOC navigation — Virtuoso fires `startReached` immediately and
  prepends 20 chapters, shifting scroll position.
- Per-page data is genuinely dynamic (chapter content, alignments,
  breadcrumb). Static prerender fights this; runtime SSR doesn't.
- User-generated content (articles, profiles) needs deploy-on-publish
  CI plumbing under static; with runtime SSR + cache, it's just per-URL
  invalidation.
- Build-time scaling. 6k pages today, easily 50k+ as more texts are
  added. Builds get slower forever.

Sefaria's architecture (Django + Node SSR sidecar + Varnish + Redis pub/sub
+ per-server in-process caches) is the right *pattern*, but ~70% of their
plumbing is multi-server coherence. At our single-node k3s scale we get the
benefits without the coordinator dance.

---

## Target architecture

Three processes inside the k3s cluster:

1. **nginx (with `proxy_cache`)** — public entry; terminates TLS via the
   ingress; caches HTML and anonymous API responses; routes to Node SSR
   for HTML paths and to Rust for `/api/*`.
2. **Node SSR (TanStack Start, server mode)** — receives HTML requests
   from nginx, fetches data from Rust over the in-cluster Service, renders
   React, returns HTML.
3. **Rust Axum API** — unchanged backend. Talks to Postgres. Issues
   PURGE calls to nginx after writes.

### Request paths

- **HTML route** (e.g. `/books/kjv-bible/john-3`):
  browser → nginx (cache hit? serve; miss? forward) → Node SSR → Rust API
  → Postgres → HTML back through nginx (stored on the way out) → browser.
- **Hydrated SPA API call** (e.g. `/api/books/kjv-bible/nodes?at_slug=…`):
  browser → nginx → Rust API. Node is **not** on this path. Anonymous API
  responses are cached at nginx too.
- **Authenticated request** (session cookie present): bypasses cache,
  hits Node or Rust directly.

### Cache key strategy

- Anonymous (no session cookie): cache aggressively. One entry per URL.
- Authenticated (session cookie present): bypass entirely. Easier than
  caching by user-id and effectively never hits.
- Site nav rendered with auth-aware content (avatar, "my notes" link):
  render that part client-only so cached HTML is identity-free.
- Hosted texts (`/books/...`, `/api/books/...`): aggressive TTL, invalidate
  only on re-ingest.
- Article pages: targeted PURGE on publish/edit/delete.
- Article listings (`/articles`, filtered variants): short TTL (~60s)
  rather than enumerating every filter combination to PURGE.
- User profiles: PURGE on profile edit.
- Editor/admin routes: never cached.

### Sizing

At current content (KJV + WEB + ASV + BBE + Darby + Kant A/B):

| URL class | Count |
|---|---|
| Marketing/static | ~15 |
| Book TOCs | ~7 |
| Bible chapters | 5,945 |
| Kant nodes | ~300–600 |
| Articles | 0 today, grows |
| Profiles | grows with users |
| **Approx total HTML URLs** | **~7,000** |

At ~50 KB per cached HTML page → ~350 MB on disk. Add cached API
responses (similar shape, similar size) → ~700 MB. A 5–10 GB PVC is
generous. nginx handles this trivially.

---

## Execution

### Step 1 — Reconfigure TanStack Start as an SSR server

**Goal**: replace the current SPA-with-prerender output with a Node SSR
server that listens on a port and renders on demand. After this step
`pnpm dev` and `pnpm build` produce a runnable Node server, not a
static `dist/client/`.

**Files to change**

- `apps/web/vite.config.ts` — drop `prerender` and `spa` blocks (their
  presence turns TanStack Start into SSG/SPA mode). Default mode is SSR
  runtime.

  ```ts
  tanstackStart({
      // SSR runtime mode: no prerender pass, no SPA shell.
      // The build produces dist/server/ + dist/client/, run via Node.
  }),
  ```

- `apps/web/package.json` — add a `start` script that runs the built
  Node entry. Confirm the exact path TanStack Start emits (likely
  `dist/server/server.mjs` or similar) and wire it up.

  ```jsonc
  "scripts": {
      "start": "node ./.output/server/index.mjs"  // adjust to actual path
  }
  ```

- `apps/web/src/routes/books.$bookSlug.$nodeSlug.tsx` — the loader's
  `prefetchInfiniteQuery` stays useful (populates the cache before render
  so the chapter HTML is filled in server-side). Keep `at_slug` cursor.

- `apps/web/src/modules/reader/components/PanelScrollView.tsx` — remove
  `initialItemCount={nodes.length}` from Virtuoso. With runtime SSR each
  request renders fresh, no hydration mismatch to engineer around.

- `apps/web/src/config.ts` — `getActiveProfile` SSR branch currently
  defaults to `"local"`. In SSR mode, the server runs in a pod with its
  own env; profile should come from `process.env.APP_PROFILE` (or
  similar) on the server side, and from `window.__ENV__` on the client.
  Reconcile these.

**Sanity checks before moving on**

- `pnpm --filter web dev` starts a dev server, route navigation works,
  chapter pages render with real content in the *initial HTML response*
  (not just after hydration).
- `pnpm --filter web build` succeeds and produces a `dist/server/`
  (or `.output/server/`) tree that `node` can run directly.
- `pnpm --filter web start` boots the production server; curl to a
  chapter URL returns HTML with the chapter text inline.
- The bugs that motivated this plan (TOC scroll leap, Kant-chapter
  hydration crash) are gone because the prerender + Virtuoso-SSR
  interaction no longer exists.

**Things to undo from the prerender push**

- `apps/web/vite.config.ts` — the prerender `filter` (excluding `/user`
  and `/admin`) is no longer needed; cache config will handle that.
- `apps/api/src/db/page.rs` — the `at_slug` 1-node restriction was a
  size hack for the dehydration cache. Revisit whether to widen it back
  to a normal window now that prerender isn't writing the data into
  every HTML file. Likely yes — wider window means smoother scroll UX.
- `apps/web/src/modules/reader/components/PanelScrollView.tsx` —
  `initialItemCount` (removed as noted above).

**Things to keep from the prerender push**

- `book_prefixed_label` on `NodeDetail`. Useful in SSR too — server
  computes it, ships it on the node response, client doesn't need TOC.
- `at_slug` query param on `/api/books/:slug/nodes`. Useful as a way
  for the loader to fetch the right window without a sort_order
  pre-lookup.
- TOC-fetch-on-demand inside `ResourcesPanel` (lazy `TocView`). Smaller
  responses on every page even with SSR.
- The `nodePageQuery.ts` shared options helper (loader + component
  share the queryKey).
- Route loader pattern (prefetch chapter content before render).

### Step 2 — Run Node SSR locally end-to-end

After step 1, prove the path works without nginx yet:

- Rust API running on `:4000` (unchanged).
- Node SSR running on `:3000` (built output).
- Browser hits `:3000` → fetches happen via `customFetch` to `:4000`.
- Verify: chapter HTML has full text in the initial response. Verify:
  hydration works (selection, infinite scroll, sentence clicks). Verify:
  login/auth still works (cookies flow correctly).

The `apps/web/src/api/fetcher.ts` `customFetch` runs on both server and
client. On the server side it needs an absolute URL (no `window`-relative
resolution). Likely needs a server-only branch: `API_BASE_URL =
process.env.API_BASE_URL ?? "http://localhost:4000"` so the Node SSR can
talk to Rust over the in-cluster Service name later.

### Step 3 — Add nginx with `proxy_cache`

Standalone first (not yet in k3s):

- nginx config with two `proxy_cache_path` zones (one for HTML, one for
  API), reasonable inactive/max_size.
- Two `location` blocks: `/api/*` → Rust, everything else → Node SSR.
- Cache bypass when `Cookie` header contains the session cookie name.
- PURGE endpoint behind `ngx_cache_purge` module on a separate listen
  port (`:8080`), not exposed publicly.
- `Cache-Control: no-store` on `/config.js` and `/api/auth/*`.

Verify: anonymous chapter request → cache miss the first time, cache
hit the second time (response time drops). Authenticated request →
cache always missed. PURGE removes the cached entry.

### Step 4 — PURGE integration in Rust handlers

Add a small `cache::invalidate(&[paths])` helper that POSTs PURGE to
the internal nginx admin port. Wire it into the handlers that mutate
content the cache stores:

- `articles::create/update/delete` → PURGE `/articles/$slug`,
  `/articles`, `/api/articles/$slug`, `/api/articles`, `/users/$handle`,
  `/api/users/$handle`.
- `users::update_profile` → PURGE `/users/$handle`, `/api/users/$handle`.
- Ingest binaries (`bible_to_db`, `kant1_*`) → end-of-run PURGE for
  every affected slug. Acceptable as a one-time minute of HTTP churn.

Fire-and-forget. PURGE failures get a warning log, not a 500 — stale
content recovers via TTL.

PURGE endpoint URL comes from env: `CACHE_PURGE_URL=http://nginx-cache:8080`.
Empty/missing → invalidation is a no-op (so local dev without nginx
works fine).

### Step 5 — k3s manifests

Three Deployments + Services in one namespace:

- `api` (Rust) — `ClusterIP`, port 4000. Stateless, can scale.
- `web` (Node SSR) — `ClusterIP`, port 3000. Single replica for v1.
- `nginx-cache` — `ClusterIP`, ports 80 (traffic) and 8080 (admin/PURGE).
  Single replica. PVC for cache data (~5 GB, `local-path` storage class
  on k3s).

Ingress (Traefik default) terminates TLS, routes everything to
`nginx-cache:80`. Cert-manager handles certs.

PVC for nginx cache survives pod restarts. PVC for Rust → reuses
existing Postgres connection (no new volume).

`/config.js` envsubst still handled by nginx-cache (same nginx as
today, just with caching added).

NetworkPolicy as defense-in-depth: only `api` pods can reach
`nginx-cache:8080`. Internet only reaches `:80` via the ingress.

### Step 6 — SEO infrastructure (independent of the above)

These are orthogonal to SSR vs SSG; defer until step 5 ships:

- **Sitemap**: Rust handler at `/sitemap.xml` (and shard files
  `/sitemap-bible-1.xml`, etc.) that enumerates all canonical URLs from
  the DB. Cache it. Regenerates implicitly when content changes (via
  same PURGE pipeline).
- **JSON-LD breadcrumbs**: render in the route component or
  `__root.tsx` so it lands in the SSR HTML. Reads from `breadcrumb`
  field on `NodeDetail` (which we can add server-side alongside
  `book_prefixed_label`).
- **OG images**: lazy per-URL generator. Rust endpoint that produces
  a 1200×630 PNG from book title + chapter label. Cache aggressively.
  Punt until we know it matters.

---

## Decisions captured

- **Cache layer**: nginx `proxy_cache`, not Varnish. Reconsider only if
  wildcard BAN-style invalidation becomes painful (unlikely at our update
  cadence).
- **PURGE protection**: separate `:8080` listen port, `ClusterIP`-only.
  No internet exposure. NetworkPolicy as belt-and-braces.
- **Cache persistence**: PersistentVolume, not `emptyDir`. Pod restart
  must not cold-start the cache.
- **Replicas**: single replica for both nginx-cache and Node SSR.
  Multi-replica means each pod has its own cache and PURGE must fan out
  — back to Sefaria's coordinator territory. Not now.
- **API caching**: anonymous API responses cached at nginx (especially
  for hosted texts). Authenticated requests bypass.
- **Listings invalidation**: short TTL (~60s), not per-filter PURGE.
- **Article/profile invalidation**: targeted PURGE, fire-and-forget.

---

## Out of scope (for now)

- Multi-replica nginx (and the pub/sub coordinator it would need).
- Varnish.
- Redis (only relevant if we need session cache, leaderboards, or
  pub/sub — currently sessions live in Postgres via tower-sessions).
- CDN in front of nginx (Cloudflare/Fastly) — defer until traffic
  warrants. The nginx cache is the first edge.
- Sitemap pagination optimization, OG image generation — step 6, after
  the core is up.

---

## What kills this plan

- TanStack Start's SSR runtime turns out to have its own SSR-time gotchas
  that are as bad as the prerender ones. Mitigation: step 2 is a local
  end-to-end check before any nginx or k3s work; if it doesn't render
  correctly we stop and rethink.
- Node sidecar memory leaks under sustained load. Mitigation: standard
  k8s liveness probe + memory limit; pod restart on OOM is normal.
- nginx `ngx_cache_purge` module unavailable in the chosen image.
  Mitigation: file-based deletion as a fallback (Rust knows the cache
  key format).
