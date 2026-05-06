# Scholia — Shape 3: Per-toc-node bibliographic identity

Implementation plan for the "compilation-friendly" data shape, decided
2026-05-05. Lets one hosted text (e.g. King James Bible) carry granular
per-book attribution (Genesis → Moses, Romans → Paul, etc.) without
forking the hosting model.

This plan is sequenced so the schema lands first while we're still
pre-migrations-bootstrap (every later step is application code that can
land in normal PR cadence).

---

## 0. Pre-flight check

Are there **other schema changes** queued that should land in the same
pre-bootstrap window? (User-table fields, billing additions, anything
brewing.) Once §1 lands and 0.3 ships, every schema change becomes a
new migration file with append-only discipline. **Surface them now.**

---

## 1. Schema changes (final pre-bootstrap edit)

All in `db/001_schema.sql`. Scope is intentionally minimal.

### 1.1 Allow `book` source to have a parent

```sql
-- before
CONSTRAINT chk_no_parent CHECK (
    source_type NOT IN ('book', 'web') OR parent_source_id IS NULL
),

-- after
CONSTRAINT chk_no_parent CHECK (
    source_type != 'web' OR parent_source_id IS NULL
),
```

`chk_chapter_has_parent` stays as-is. `web` still cannot have a parent.
Books become parent-optional, enabling Bible (parent) → Genesis (child).

### 1.2 Add nullable `source_id` to `toc_nodes`

```sql
-- in CREATE TABLE toc_nodes (...), after source_node_id:
source_id       UUID REFERENCES sources(id) ON DELETE SET NULL,
```

NULL = "this node is internal structure of an ancestor that did make
a bibliographic assertion." Non-null = "this node *is* the toc anchor
for a discrete bibliographic work."

Plus a partial index for reverse lookups:

```sql
CREATE INDEX idx_nodes_source ON toc_nodes (source_id)
    WHERE source_id IS NOT NULL;
```

### 1.3 Denormalize effective source on quotations

Quotations are listed across many surfaces (user's quotations panel,
article-quotation pickers, etc.). Resolving the effective source per
row at read time means N ancestor walks per page. Denormalize:

```sql
-- on quotations only:
source_id  UUID NOT NULL REFERENCES sources(id),
```

NOT NULL because every quotation anchors to a real toc node, which
always resolves to *some* source (worst case the book's root source).
Filled at create time by the citation resolver (§2).

**Not** added to:

- `article_quotations` — these are snapshots from user-generated
  articles, not from hosted books. No `anchor_node_id` / `book_id`;
  they already snapshot `author_display_name` directly. Orthogonal
  to Shape 3.
- `resources` — displayed in a side panel scoped to the current toc
  node; the parent UI already holds the resolved source.

### 1.4 Run `db_reset.sh`

We're early development, no production data. Reset cleanly to confirm
the new schema applies.

---

## 2. Citation resolver (Rust helper)

New module `packages/api/src/db/citations.rs`. One function:

```rust
/// Returns the effective bibliographic source for an anchor.
/// Walks ancestors of `anchor_node_id` looking for a non-null
/// `toc_nodes.source_id`; falls back to `books.source_id`.
pub async fn resolve_effective_source(
    pool: &PgPool,
    book_id: Uuid,
    anchor_node_id: Uuid,
) -> Result<Uuid, sqlx::Error> { ... }
```

Implementation uses LTREE ancestor lookup (one query):

```sql
WITH target AS (
    SELECT path FROM toc_nodes WHERE id = $2 AND book_id = $1
)
SELECT anc.source_id
FROM toc_nodes anc, target
WHERE anc.book_id = $1
  AND anc.path @> target.path
  AND anc.source_id IS NOT NULL
ORDER BY anc.depth DESC
LIMIT 1;
```

If null → fetch `books.source_id` and return that.

Used by §3 at write time. Optionally exposed as a SQL function later
for convenience; for now a Rust helper is fine.

---

## 3. Wire denormalization at write time

One caller in `packages/api/src/db/quotations.rs::create_quotation`:
after computing `anchor_node_id`, call `resolve_effective_source` and
include `source_id` in the INSERT.

`article_quotations` are unrelated (see §1.3).

### 3.1 No backfill needed

We reset the DB after §1.4. The fresh DB has no quotations. Kant
reimport seeds nothing into `quotations` (only user-generated). So
no migration / backfill step.

---

## 4. Citation rendering swap

Mechanical. Anywhere we currently render a quotation's source by going
`books.source_id → sources` table, switch to using
`quotations.source_id → sources` (already denormalized in §3).

### 4.1 Files

- `packages/api/src/db/quotations.rs` — `list_quotations`,
  `list_all_quotations`, `quotation_with_context` shapes.
- Response models: `models/quotation.rs`.

`article_quotations` left alone (orthogonal — see §1.3).

### 4.2 Add parent-compilation hint

Each citation response gains an optional `parent_compilation_title`
(populated when the cited source has a non-null `parent_source_id`).
Frontend renders *"Genesis 1:1, in: King James Bible."*

### 4.3 Codegen

Regenerate `openapi.json` + frontend client.

---

## 5. Library refactor

The substantive piece. `packages/api/src/db/library.rs`.

### 5.1 Work-discovery union

Today: every `books.source_id`. New: union of `books.source_id` and
`toc_nodes.source_id` (where non-null), de-duped.

For child sources, attach the parent compilation title via
`JOIN sources parent ON parent.id = s.parent_source_id`.

### 5.2 Group shape — author OR self-named

No "Anonymous" bucket. Every work belongs to a group; the group's
primary identity is either an author (when the source has one) or
the source itself (when it doesn't).

```
group_label =
    primary_author.display_name        if the source has an author
    coalesce(title_display, title)     otherwise
```

Three structural variants render under one shape:

- **Authored multi-work** (Kant): header is the author; works list is
  the author's books.
- **Authorless compilation** (Bible): header is the work's display
  name; works list is the *child* sources (Genesis, Exodus, …).
- **Authorless singleton** (Gilgamesh, Beowulf): header is the work's
  display name; works list is empty — the frontend renders the
  header alone as a leaf entry.

`title_display` already exists in `sources` for the display-name slot.
No new schema column.

### 5.3 Library response shape

Each group carries `primary_kind` so the frontend can route correctly:

```jsonc
{
    "primary_label": "Kant" | "The Bible" | "Gilgamesh",
    "primary_kind": "author" | "self",
    "primary_slug": "/authors/kant" | "/books/king-james-bible",
    "works": [{ "title": "...", "slug": "...", ... }, ...]
}
```

`primary_kind: "self"` + non-empty `works` ⇒ compilation; empty
`works` ⇒ singleton.

### 5.4 Co-existence

Each top-level work (Kant's Critique, the Bible itself) appears once.
Child sources (Genesis, Romans) appear once each under their compilation
header. No double-counting; child sources don't *also* appear under a
"Moses" or "Paul" group unless we explicitly model traditional-author
attribution as `source_persons` rows on the children — which we may or
may not (see Open question below).

### 5.5 Frontend

Render header as link:
- `primary_kind === "author"` → existing author page route.
- `primary_kind === "self"` → reader at `primary_slug`.

Render `works` as children under the header. If `works` is empty,
render the header alone (singleton case).

### 5.6 Open question (Bible-specific, deferred)

If a Bible child source *also* has author attribution (Romans → Paul),
should the work appear under the Bible header *and* under the Paul
header? Two reasonable answers:

- **Compilation-primary** (default): the work appears only under the
  Bible. Author attribution is shown in the work's own metadata, but
  doesn't drive a second library entry.
- **Dual-listed**: the work appears under both. Cleaner discoverability,
  but breaks the "each work appears once" invariant.

Default to compilation-primary. Revisit when actually importing the
Bible.

---

## 6. Editor + frontend touch-ups

### 6.1 Source-create form

Allow `parent_source_id` selection when `source_type === "book"`,
matching the current chapter UX. Validation update on the API side
already covered by the relaxed CHECK constraint.

### 6.2 Toc-node source assignment

Admin-only affordance: on a toc node, assign a `source_id`. For Kant
this is never used; for a future Bible import it'd be set
programmatically by the import script. UI deferred until we actually
import a compilation; the column is queryable directly via DB tools
in the meantime.

---

## 7. Audit & smoke test

After §1–§4 land:

- [ ] `db_reset.sh` cleanly re-applies the new schema
- [ ] Kant import runs end-to-end with no changes (`source_id` stays
      NULL on all toc nodes)
- [ ] Read a Kant page, create a quotation, confirm the citation still
      shows "Critique of Pure Reason — Kant"
- [ ] List quotations, confirm rendering unchanged
- [ ] `cargo check -p api` clean, `pnpm tsc --noEmit` clean

After §5:

- [ ] Library page renders Kant under Kant (no regression)
- [ ] Manually insert a fake compilation (Bible parent + 1–2 child
      sources + a toc node with `source_id`) via SQL; confirm library
      shows it correctly. Tear down after.

---

## 8. Sequencing & estimate

| Step | Scope | Estimate |
|---|---|---|
| §1 | Schema edit + db reset | 30 min |
| §2 | Citation resolver | 1–2 h |
| §3 | Denormalization wiring | 1–2 h |
| §4 | Citation rendering swap + codegen | 2–3 h |
| §5 | Library refactor | ~1 day |
| §6 | Editor / frontend touch-ups | 1–2 h |
| §7 | Audit & smoke test | 30 min |

Steps §1–§4 are one PR-sized unit and unblock the schema portion.
§5 is a separate unit. §6 is a follow-on. The Bible isn't reachable
in the library until §5 lands, but it can be read end-to-end after §4.

---

## 9. Out of scope

Explicitly deferred:

- **`source_id` on `resources`** — borrows resolution from the toc
  panel context. Add later if a real need surfaces.
- **`source_id` on cross_references / footnotes** — internal
  navigation, no source-attribution rendering today.
- **Multi-translation Bible (NRSV alongside KJV)** — handled by the
  existing `translation_of_id` / per-translation child sources; no
  Shape-3 change required. Plan when the second translation actually
  lands.
- **Bible import script itself** — design once we're ready to import.
- **Stale-on-toc-source-reassignment backfill** — extremely rare event
  (admin re-attributes a Bible book); add an admin SQL recipe in the
  ops doc when first needed, no automation.
