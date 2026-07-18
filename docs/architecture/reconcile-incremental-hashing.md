# Spec: Tier-2 incremental reconcile via content hashes

**Status:** proposed (not yet implemented)
**Applies to:** `packages/reconcile`, `packages/struct_to_db`, `packages/bible_to_db`

## Motivation

The reconciling re-import (see `reconcile.rs` in both importers) currently rewrites
**every** sentence on every run: it bumps all rows out of the unique-index space
(the "offset trick") and then reassigns each row individually. That is O(book)
per-row `UPDATE`s even when nothing changed. Two problems:

- **A changed corpus pays full price** — auto-ingest gates cluster Jobs on a
  content hash of the derived structs (`docs/architecture/overview.md`), so
  unchanged content never starts a Job; but any change at all rewrites every
  sentence of every book in the corpus. A one-line MD fix costs O(book), and a
  Job re-created after deletion (selfHeal) re-runs the same full reconcile.
- **The laptop→cluster tunnel is slow** — each per-row `UPDATE` is a separate
  round-trip; a full-book reconcile over the port-forward takes minutes.

Tier-2 adds a **two-level content hash** (book root + per-node) so a run can skip
the content work for unchanged nodes entirely, and makes the positional renumber
**set-based** (one statement) instead of a per-row loop. A `--full` flag bypasses
all of it and forces today's behavior.

Non-goal: a full recursive Merkle tree (tier 3). Two levels capture ~90% of the
benefit at a fraction of the complexity. Non-goal: changing the global
`sentence_number` model.

## What gets hashed — and what must NOT

The **unit** is the per-node subtree:
- **Bible:** the chapter node (`toc_nodes` depth 1). Its subtree = the chapter's
  verses/sentences + verse page-markers + the chapter paragraph block text.
- **Kant:** the section `toc_node` (one `source_ref`). Its subtree = the node's
  blocks (text + sentences), footnotes (+ their sentences), and page-markers.

> **Granularity is node-level, not block-level.** A Bible chapter node has exactly
> one block, so node ≈ block there. A Kant node is a whole *section* with many
> blocks — so editing one paragraph flips that node's hash and the aligner
> re-examines *all* of that node's blocks (in memory). It never affects *other*
> nodes. The cost of that within-node re-examination is in-memory only — see
> "selective writes" below — so it's cheap; the DB only ever sees writes for rows
> that actually changed. (Pruning *within* a node, so siblings aren't even
> re-examined, is the optional third level under "Future extensions".)

A node's hash is computed over **everything reconcile would write for that node,
EXCEPT the recomputed positional/numbering fields**:

- **Include** (content identity): every sentence's `text`/`html`/`original_text`/
  `original_html` and `segment`; page-marker `(system, ref_value, char_offset)`;
  block `text`/`html`/`original_*` and `block_type`; node `label`/`label_html`;
  footnote `number` + its sentences' content. Children are hashed **in order**,
  and the **child count is implied by the sequence** — so a split/merge/insert/
  delete inside the node changes its hash.
- **Exclude** (derived, recomputed each run): global `sentence_number`,
  `paragraph_number`, `figure_number`, within-block `position`, and `natural_key`.

> **Why the exclusion is load-bearing:** `sentence_number` is global per book, so a
> split early in the book shifts the numbers of every later sentence. If numbering
> were in the hash, that one split would invalidate every downstream node's hash
> and destroy all pruning. By hashing only *content + shape*, an upstream split
> leaves downstream node hashes untouched — we skip re-aligning them and fix their
> numbers with a single set-based statement instead (see Algorithm step 6).

The **root hash** = hash of the ordered list of node hashes.

Hash function: `blake3` (one small dep, fast), hex-encoded into a `TEXT` column.
Encode fields with an unambiguous separator (e.g. `0x1f` unit separator) and a
fixed field order so the hash is stable across runs/platforms. (Do **not** use
`std`'s `DefaultHasher` — SipHash isn't guaranteed stable across Rust versions,
and these hashes are persisted.)

## Schema — migration `0005_content_hash.sql`

```sql
ALTER TABLE toc_nodes ADD COLUMN content_hash TEXT;
ALTER TABLE books     ADD COLUMN content_hash TEXT;
```

Both nullable. **`NULL` means "unknown — treat as changed"** so the optimization
is fail-safe: a missing hash never causes a skip. The first reconcile after this
migration sees all-`NULL` and behaves like `--full`, backfilling every hash.
No per-sentence hash column is needed for tier 2 (node hashes are recomputed from
in-memory desired content; only node + root hashes are stored).

## Shared hashing module — `packages/reconcile/src/hash.rs`

Both importers and both code paths (insert + reconcile) must compute identical
hashes, so the hashing lives in the shared crate:

```rust
pub fn sentence_content_hash(text, html, original_text, original_html, segment) -> Hasher input
pub fn node_hash(/* ordered node payload */) -> String   // hex blake3
pub fn root_hash(ordered_node_hashes: &[String]) -> String
```

Define a small `NodeContent`/`SentenceContent` shape the caller fills from its own
model (Kant `Output`, Bible `DesiredChapter`) and feeds to `node_hash`. Keep the
field order + separators in one place here.

## Algorithm — default (incremental) mode

All within the existing single transaction.

1. **Compute desired hashes in memory** from the parsed input: per-node hashes and
   the root hash.
2. **Read the stored root** (`SELECT content_hash FROM books WHERE id = $1`) — one
   row. If it equals the desired root → **nothing changed anywhere; commit nothing
   and return** ("no changes"). One round-trip for a no-op.
3. **Read stored per-node hashes** (`SELECT source_ref, content_hash FROM toc_nodes
   WHERE book_id = $1`) — cheap, no text loaded.
4. **Changed set** = nodes where `desired_hash != stored_hash` (or stored is NULL).
5. **For each changed node:** load its existing sentences and run the per-node
   reconcile logic (aligner → update/split/merge/insert/delete, dependent
   migration, page-marker rebuild, block/node text update), scoped to changed nodes
   instead of all nodes. Track whether any node had a **count delta**
   (insert/delete/split/merge).
   - **Selective writes (required).** Within a changed node, only write the
     sentences whose content actually differs — the aligner already holds the old
     text, so byte-identical rows are skipped. Today's apply rewrites *every*
     sentence in the node it processes; that must change, or the "k content writes"
     figure below is lost (a one-sentence edit in a big Kant section would still
     rewrite the whole section). Unchanged sibling blocks in the same node are
     re-examined in memory but not re-written.
6. **Positional renumber — set-based, only if a count delta occurred.** A structural
   change shifts the global `sentence_number` (and, for changed nodes, within-block
   `position`/`natural_key`, already handled in step 5). Recompute the dense global
   numbering for the whole book in **two set-based statements** (offset then assign),
   e.g.:
   ```sql
   -- 1) move out of the unique-index space (single statement)
   UPDATE sentences SET sentence_number = sentence_number + 1000000
   WHERE book_id = $1 AND sentence_number IS NOT NULL;
   -- 2) reassign by document order (single statement, window function)
   WITH ordered AS (
     SELECT s.id, ROW_NUMBER() OVER (ORDER BY tn.sort_order, cb.position, s.position) AS rn
     FROM sentences s
     JOIN content_blocks cb ON s.block_id = cb.id
     JOIN toc_nodes tn ON cb.node_id = tn.id
     WHERE s.book_id = $1 AND s.sentence_number IS NOT NULL
   )
   UPDATE sentences s SET sentence_number = o.rn FROM ordered o WHERE s.id = o.id;
   ```
   (Footnote sentences use their own sequence; mirror with the footnote ordering.)
   Two round-trips regardless of book size — the chattiness was N *separate*
   per-row queries, not one big statement. If there was **no** count delta, numbers
   are already correct → skip this entirely. The renumber writes only
   `sentence_number` (and within-node `position`) — **never `content_hash`** — so
   the unchanged downstream nodes it sweeps keep their stored hashes and are still
   skipped on the next run. Numbering and hashing stay decoupled in both directions.
7. **Write back hashes:** update `content_hash` for changed nodes and the root on
   `books`. (Unchanged nodes keep their stored hash.)
8. **Commit** (or roll back on `--dry-run`).

## `--full` fallback (required)

`--full` (and the implicit first-run-with-NULL-hashes case) **bypasses all hash
checks**:

- Treat **every** node as changed — load + align + apply all nodes (today's exact
  behavior).
- Always run the positional renumber.
- Recompute and store **all** node hashes + the root.

This is the escape hatch when hashes might be stale or a hashing bug is suspected,
after a hashing-format change, or simply to force a known-good full pass. It is
behaviorally identical to the current reconcile, plus it rewrites the hashes so the
next default run is fast again.

`--dry-run` composes with both modes: do all the work in the transaction, print the
report, then roll back (hashes included).

## Fresh-insert path

The importers' insert path (new book) must also compute and store node + root
hashes as it builds, using the same `reconcile::hash` functions, so the first
*reconcile* after a fresh import is already in the fast state. (If skipped, the
first reconcile just falls into the NULL→full path and backfills — acceptable, but
writing on insert is cleaner.)

## Per-importer notes

- **Bible:** unit = chapter node. `natural_key` is verse-scoped
  (`book:chapter:verse/s{idx}`), so an upstream split in another verse/chapter does
  **not** change a verse's `natural_key` — only the global `sentence_number` shifts,
  which step 6 handles set-based. Within-chapter `position` shifts only inside a
  changed chapter (handled in step 5). Heading sentences (depth-0, no verse marker)
  remain untouched, as today.
- **Kant:** unit = `toc_node`. Node hash must also cover footnote content and the
  block/segment fields. The existing cross-edition parity check
  (`validate_translation_parity`) runs **before** hashing and is unaffected.

## Efficiency (round-trips over the tunnel)

| Scenario | Today | Tier 2 |
|---|---|---|
| No-op re-run | offset + ~N per-row reassigns | **1** (root read) |
| Edit k sentences, no count change | ~N reassigns | node-hash read + k content writes |
| One split | ~N reassigns | changed-node writes + **2** renumber stmts |
| `--full` | N reassigns | same as today + hash write-back |

## Implementation steps

1. `db/migrations/0005_content_hash.sql` (columns above).
2. `packages/reconcile/src/hash.rs` (blake3) + add `blake3` to `reconcile/Cargo.toml`;
   re-export from `lib.rs`.
3. Define the `NodeContent` input shape; have each importer build it from its model.
4. Insert paths (both importers): compute + store node/root hashes.
5. Reconcile (both importers): root short-circuit → per-node diff → scoped apply →
   set-based renumber (only on count delta) → hash write-back. Add `--full` flag
   (and treat NULL hashes as changed). Keep `--dry-run`.
6. Replace the blanket offset-everything in the apply with the set-based renumber.

## Verification

- `--full` produces byte-identical DB state to the current reconcile (diff a dump
  before/after on a fixed dataset).
- No-op default run: zero `UPDATE`s to `sentences` (assert via a row-count/`xmax`
  check or statement log), root unchanged.
- Single text edit: only the edited sentence's row written; neighbors' `updated_at`
  unchanged; quotation intact.
- Single split: first half keeps UUID, second inserted, quotation extended, global
  `sentence_number` dense and correct afterward (compare against a `--full` run).
- Tamper test: manually corrupt a stored `content_hash`, run default → that node is
  treated as changed and corrected; run `--full` → everything re-verified.
- Bible + Kant both: `cargo test`, then e2e on a dev DB.

## Future extensions

- **Third level (block hashing).** Add `content_blocks.content_hash` and descend
  `book → node → block → sentences`, so a one-block edit in a large Kant section
  doesn't even re-examine sibling blocks. Pure in-memory savings (DB writes are
  already minimal via selective writes), and redundant for the Bible (one block per
  chapter). Add only if section-level re-aligns become a measurable cost.
- **Full recursive Merkle (tier 3).** Generalize the above to every level. Same
  caveat — the win is in-memory pruning, since writes are already content-scoped.
- **Non-global numbering.** The remaining book-wide renumber on a split exists only
  because `sentence_number` is global and dense. Per-node numbering would remove it
  entirely, but the reader/quotation display leans on the current scheme — a larger,
  separate change.

## Risks / safeguards

- **Silent skip = data drift.** A hashing bug that wrongly matches would skip a real
  change. Mitigations: `NULL`-is-changed default, the `--full` escape hatch, the
  `--full`-equals-today equivalence test, and hashing *everything reconcile writes*
  (so any writable field change flips the hash).
- **Hash-format changes are breaking** — any change to field order/separators/algo
  invalidates stored hashes. Treat a format change like a data migration: bump and
  run `--full` once to rewrite all hashes.
- **Root cause not addressed:** global dense `sentence_number` still forces a
  book-wide renumber on any split — but now it's 2 statements, not N. Making
  numbering non-global (per-node) would remove even that; out of scope here.
