# Database migrations

## Purpose

Scholia's schema changes are applied as **append-only** sqlx migrations
in `db/migrations/`. The same migration files are consumed by two
different entry points — `sqlx migrate run` (local dev) and the
`api migrate` subcommand on the production binary (cluster init
container). This doc explains why two entry points exist, what
constraint forces the split, and how to add a migration.

## sqlx compile-time SQL verification

The killer feature of sqlx — and the reason we use it over Diesel or a
hand-rolled SQL builder — is that **every SQL statement in the codebase
is verified at compile time against a real Postgres database**.

When you write

```rust
sqlx::query!(
    "SELECT id, handle FROM users WHERE id = $1",
    user_id,
)
```

the `query!` macro expands at compile time and:

1. Connects to the Postgres at `DATABASE_URL`.
2. Sends the SQL with `PREPARE` so Postgres parses it.
3. Asks Postgres for the inferred parameter types and the returned
   column types.
4. Generates a typed Rust struct matching the result row.

If you reference a column that doesn't exist, or pass the wrong
parameter type, the **build fails** before a single test runs. This
turns the entire schema into a static contract enforced by the
compiler.

The cost: **the schema must exist in the database the compiler points
at, every time you build.**

## The chicken-and-egg in `db_reset.sh`

The reset script needs to throw away the schema and re-apply every
migration from scratch. The naïve flow would be:

```
1. DROP SCHEMA public CASCADE; CREATE SCHEMA public;
2. cargo run -p api -- migrate
```

Step 2 invokes `cargo`, which needs to **compile the api crate** before
it can run anything. Compiling expands every `sqlx::query!` and
`sqlx::query_as!` macro, each of which tries to verify its SQL against
the database that step 1 just emptied. You get hundreds of "relation
'users' does not exist" errors and the build fails.

To use the api binary to migrate, you have to build it. To build it,
you need the schema. To have the schema, you have to migrate. Stuck.

There are two well-known escape hatches and we use **both**, one for
each context:

## Dev path: external sqlx-cli

`sqlx-cli` is a separate binary, installed once via
`cargo install sqlx-cli --no-default-features --features postgres,rustls`.
It has no `sqlx::query!` macros inside it — it just walks a migrations
directory and applies SQL files. Because it's already compiled and has
no compile-time SQL checks, it doesn't care what's in the database
when you invoke it.

The dev reset becomes:

```
1. DROP SCHEMA …                ← schema gone
2. sqlx migrate run              ← external binary, applies files, doesn't compile anything
3. (schema now exists; next `cargo build` of api works as normal)
```

No api crate is rebuilt between steps 1 and 2, so the chicken-and-egg
never appears. This is what `scripts/db_reset.sh` does.

## Prod path: offline metadata + embedded migrations

Production has the same problem in principle (the cluster init
container runs `api migrate` from the same image as the main API), but
we dodge it differently: **we don't compile against the live database
in production.**

The pattern is sqlx's **offline mode**:

1. On a developer's machine, with a populated schema, run `cargo sqlx prepare`.
2. sqlx visits every `query!` site, queries the live DB *once* per
   distinct SQL, and writes the inferred types as JSON into a `.sqlx/`
   folder at the workspace root.
3. Commit `.sqlx/` to git.
4. The Dockerfile builds with `SQLX_OFFLINE=true`. The `query!` macros
   now read the committed JSON instead of connecting to Postgres. The
   build succeeds with **no live DB available**.
5. Migrations are embedded into the binary at compile time via
   `sqlx::migrate!("../../db/migrations")` (see
   `apps/api/src/migrate.rs`), so the SQL files don't even need to be
   on disk in the runtime image.
6. At pod startup, the init container runs `api migrate`. The binary
   is already compiled — no macros are expanded at runtime — so it
   doesn't care what's in the database when it starts. It applies the
   embedded migrations against an empty schema, exits, and the main
   container starts.

The init container and the main container both come from the same
Docker image; `default-run = "api"` in `apps/api/Cargo.toml` plus
argv dispatch in `main.rs` make `api migrate` and `api` (the default
server) two modes of one binary.

The `.sqlx/` offline metadata is committed (the Dockerfile builds with
`SQLX_OFFLINE=true`); regenerate it after any sqlx query change with
`pnpm api:sqlx:prep` against the local DB. Local builds outside Docker
still verify against the live local Postgres — fine, every dev machine
runs one.

## Side-by-side

| | Local dev (`scripts/db_reset.sh`) | Production (init container) |
| --- | --- | --- |
| Entry point | `sqlx migrate run` (external CLI) | `api migrate` (our binary) |
| Reads migrations from | `db/migrations/` on disk | Embedded into binary via `sqlx::migrate!` |
| Compile-time SQL check at run? | None — sqlx-cli is pre-built | None — api binary is pre-built |
| Build-time SQL check uses… | Live dev Postgres | Committed `.sqlx/` JSON (`SQLX_OFFLINE=true`) |
| Live DB needed to *build* api? | Yes | No |

Two paths, same `db/migrations/0000_initial.sql`.

## Adding a migration

```bash
# 1. Create the migration file with a sequential prefix.
sqlx migrate add --sequential <semantic-name>

# Result: db/migrations/<NNNN>_<semantic-name>.sql
# (Numbering starts at 0000 in this repo — see feedback-migration-naming
# in auto-memory. sqlx-cli will pick the next integer.)
```

Edit the new file with `CREATE TABLE` / `ALTER TABLE` / … as needed,
then:

```bash
bash scripts/db_reset.sh   # drop schema + re-apply all migrations from scratch
```

That's the inner loop for dev. The fresh schema includes the new
migration, and the `_sqlx_migrations` ledger records it.

Naming: `NNNN_<semantic-name>.sql`, sequential from `0000`, the name
conveying the change at a glance. Append-only discipline below.

### Append-only rule

Once a migration has been applied anywhere — even just on your laptop
— **treat it as immutable**. sqlx records each migration's SHA-256
checksum in the `_sqlx_migrations` table and refuses to proceed if a
file's hash drifts from what was recorded. The rule that enforces this
in our workflow: **never edit a previously-applied migration**.
Multi-step schema changes (add nullable column → backfill data → set
NOT NULL) become *multiple* migrations, not one edited file.

The first time we deploy to a real cluster, the `0000_initial.sql`
applied there becomes the permanent baseline; every change after that
ships as `0001_…`, `0002_…`, and so on.
