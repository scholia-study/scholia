# `reconcile`

The book-agnostic engine for **reconciling re-imports**: updating an
already-imported book *in place* from a freshly parsed struct, instead of
deleting and re-inserting it. An in-place update preserves the UUIDs of
unchanged sentences — and therefore the quotations, notes, resources, and
cross-references anchored to them — across edits to the source text.

Write the hard parts (alignment, content hashing, dependent migration, the full
update/insert/delete/renumber dance) **once**, here, and let every importer
reuse them. Adding the Nth book should never mean re-implementing reconcile.

## Two tiers of reuse

**Tier 1 — primitives** (`align`, `deps`, `hash`). Small, composable, no opinion
about document shape. Use these directly if your book's structure doesn't fit
the standard node→block→sentence tree.

- `align`: `plan_block(label, old, new) -> BlockPlan` — text-aligns one "unit"'s
  before/after sentence lists into update / split / merge / insert / delete, or
  errors on ambiguity. Knows nothing of paragraphs/verses/footnotes; feed it one
  unit at a time.
- `deps`: `migrate_dependents`, `extend_anchors_to`, `sentence_has_dependents` —
  keep user/editor data attached to the right sentence when one is merged away or
  split. Data-integrity-critical, so it lives in exactly one place.
- `hash`: `NodeContent`/`BlockContent`/`SentenceContent`/`MarkerContent`/
  `FootnoteContent` + `node_hash` / `root_hash` — the per-node + root content
  hashing that drives the incremental reconcile.

**Tier 2 — the full orchestrator** (`orchestrate`). One generic
`reconcile_book(...)` that reads from the owned `ReconcileInput` **IR** and does
the entire reconcile. This is what a standard prose/poetry book wants.

| Importer | Uses |
|---|---|
| `struct_to_db` (all corpora) | Tier 2 (`reconcile_book`) |
| `shakespeare1_struct_to_db` | Tier 2 (`reconcile_book`) |
| `bible_to_db` | Tier 1 primitives, under its own verse-shaped orchestration (does **not** use `reconcile_book`) |

The rest of this document is about Tier 2.

## The IR (`ReconcileInput`)

IR = **Intermediate Representation**, the compiler term for a neutral data
structure that sits between many front-ends (each book's concrete struct model)
and one back-end (`reconcile_book`). Each book's model is a *different Rust type*,
so a single generic function can't take them; instead every book translates its
model *into* this one canonical shape and the engine only ever sees the IR.

```
ReconcileInput { nodes: Vec<NodeInput> }
  NodeInput     { source_ref, parent_source_ref, slug, path, sort_order, depth,
                  label, label_html, anchor: NodeAnchor, blocks: Vec<BlockInput> }
    BlockInput    { position, block_type, paragraph_number, figure_number,
                    text, html, original_text, original_html, sentences: Vec<SentenceInput> }
      SentenceInput { position, sentence_number, segment, indent,
                      text, html, original_text, original_html,
                      markers: Vec<MarkerInput>, footnotes: Vec<FootnoteInput> }
```

Two properties make it work:

- **Owned, not borrowed.** The IR holds `String`/`Vec`, not references into the
  model, so it outlives the model borrow and survives across the many `await` DB
  calls inside the orchestrator without lifetime entanglement.
- **A union of every book's needs.** Each mapper fills the fields its book has and
  defaults the rest — and that defaulting *is* how book-specific behaviour is
  expressed, as data rather than as branches in the engine:
  - `indent: None` (kant1) vs `Some(..)` (shakespeare) → NULL column vs written.
  - `footnotes: vec![]` (shakespeare) vs populated (kant1) → footnote loops no-op
    when empty.
  - `anchor: NodeAnchor` → how an *added* node's source link is created (below).

  So `reconcile_book` has one code path; "kant1 has footnotes, shakespeare has
  work-sources" is encoded in what the IR carries, never in `if book == …`.

### `NodeAnchor`

The one genuinely polymorphic field — how an **added** node's source link is set
(existing nodes are never touched):

- `None` — both `source_id` and `source_node_id` stay NULL.
- `SourceNode(uuid)` — translation node: point `toc_nodes.source_node_id` at the
  source book's matching node (kant1 translations).
- `WorkSource { title, publication_year, parent_source_id, author_person_id,
  created_by }` — Bible-shape sub-work: the engine creates a `source_type='chapter'`
  source under the book's compilation source, links the author, and points
  `toc_nodes.source_id` at it (shakespeare). The ids are passed *in* so the mapper
  stays pure (no DB query).

## Two mappings per book — and the hash-parity invariant

Each importer writes a small glue module with **two** model→X mappings:

1. **`to_input(model, …) -> ReconcileInput`** — the full, writeable IR above.
2. **`compute_hashes(model) -> (Vec<(source_ref, hash)>, root)`** — built from a
   *content-only* `NodeContent` (deliberately **omits** positions,
   `sentence_number`, `paragraph_number`, `natural_key`, and `indent`, so a pure
   renumber can't invalidate a node's hash).

> ⚠️ **Hash parity is sacred.** The importer computes hashes itself and passes
> them *into* `reconcile_book` (which never recomputes them). Both the fresh-insert
> path and the reconcile path must hash *identical* content, and the hashing must
> stay byte-stable over time — otherwise every stored hash goes stale and every
> node looks "changed". Never alter `hash.rs` or a book's `node_content` mapping
> casually; if you must, run every affected book with `--full-rewrite` once to
> rewrite stored hashes. (Corollary: because `indent` is not in the hash, an
> indent-only edit isn't detected — it needs `--full-rewrite`.)

## What `reconcile_book` guarantees

```rust
pub async fn reconcile_book(
    tx: &mut Transaction<'_, Postgres>,
    book_id: Uuid,
    input: &ReconcileInput,
    desired_node_hashes: &[(String, String)],   // (source_ref, hash), document order
    desired_root: &str,
    system_ids: &HashMap<String, Uuid>,         // reference-system slug -> id
    is_translation: bool,
    source_sentence_map: &HashMap<(String, i16, i16), Uuid>,   // empty unless translation
    source_fn_sentence_map: &HashMap<(i32, i16), Uuid>,        // empty unless translation
    force: bool,
    full_rewrite: bool,
) -> Result<ReconcileReport, Box<dyn std::error::Error>>
```

- **Incremental.** If the root hash matches, it returns `unchanged` after one
  query. Otherwise only nodes whose hash differs are loaded and applied.
- **Identity-preserving.** Unchanged sentences keep their UUIDs; a same-count
  block rewrites only the rows whose content actually differs. Splits/merges
  migrate dependents onto the surviving sentence; deletes of a sentence that still
  has user data abort unless `--force`.
- **Strictly-additive growth is allowed** (new TOC nodes, appended blocks, new
  footnotes) and reconciled alongside existing rows.
- **Ambiguity aborts** (with a message pointing at `--replace` / reset): removed
  or reordered nodes, shifted block positions, mid-book paragraph/figure/footnote
  renumbering, or two structural edits in one block.
- `full_rewrite` bypasses the hash checks and rewrites everything.
- It only does DB work inside the passed `tx`; **commit/rollback (`--dry-run`)
  and any cache purge are the importer's job**, after `reconcile_book` returns.

## Adding a new book

For a standard node→block→sentence book, the per-book cost is small and
mechanical — the engine is reused wholesale.

1. **Model + producer** (`<book>_md_to_struct`) — the only genuinely
   text-specific part: parse the source into a struct (`Output` with
   `toc_nodes → content_blocks → sentences`).
2. **A glue module** (`<book>_struct_to_db/src/reconcile_input.rs`), ~150 lines:
   - `node_content` / `block_content` / `sentence_content` → build the crate's
     `NodeContent` for hashing (content fields only).
   - `compute_hashes(&output)` → `(node_hashes, root)` via `node_hash`/`root_hash`.
   - `to_input(&output, …)` → the `ReconcileInput`, choosing the right
     `NodeAnchor` for your source shape and defaulting fields you don't have
     (`indent: None`, `footnotes: vec![]`, etc.).
3. **Importer wiring** (`<book>_struct_to_db/src/import.rs`): on a book that
   already exists and isn't `--replace`d, reconcile by default:

   ```rust
   let (node_hashes, root) = reconcile::/* your */ compute_hashes(&output);
   let input = to_input(&output, /* anchor inputs */);
   let report = reconcile::reconcile_book(
       &mut tx, book_id, &input, &node_hashes, &root, &system_ids,
       /* is_translation */ false, &empty_map, &empty_map, force, full_rewrite,
   ).await?;
   if dry_run { tx.rollback().await? } else { tx.commit().await? }
   report.print(dry_run);
   ```

   Keep `--replace` (cascading delete + fresh insert) as the destructive escape
   hatch, and expose `--dry-run` / `--force` / `--full-rewrite`.

That's it — no new alignment, hashing, split/merge, dependent-migration, or
renumber logic. Those are written and tested once here, and every book inherits
fixes and new capabilities for free.

### Scaling discipline

- **New need → IR field/variant + one orchestrator branch, never a per-book
  fork.** When a future book wants something new (drama speakers, multi-edition
  apparatus, …), add it to the IR (like `NodeAnchor`/`footnotes`/`indent`) and
  grow `reconcile_book` by one well-tested branch — don't reimplement reconcile
  in the importer.
- **The mapper boilerplate may repeat, and that's fine.** `block_input` /
  `sentence_input` are near-identical 1:1 field copies across books; that's the
  right place for duplication (the thin adapter where real model differences
  live). Resist abstracting it (a derive macro, a shared "standard prose model")
  until several books have *literally identical* models and the pattern forces it.

## Verifying a reconcile change

Never test against the real/dev database — use a disposable Postgres (e.g. a
throwaway `docker run … postgres`), apply `db/migrations/*.sql`, and point the
importer at it with `--database-url`. For each affected book:

1. Fresh import succeeds.
2. **Re-run with no change → must report `no changes (root hash matched)`.** This
   is the proof hash parity held (the most important check after touching hashing
   or a mapper).
3. A one-character edit reconciles with the expected `sentences updated` count,
   `global renumber: skipped`, and the edited sentence's **UUID unchanged**.

Plus `cargo test -p reconcile` (alignment + classifier + hash unit tests) and
`cargo build` / `cargo clippy` on the crate and its importers.

## See also

- `docs/architecture/reconcile-incremental-hashing.md` — the hashing/short-circuit
  design in depth.
