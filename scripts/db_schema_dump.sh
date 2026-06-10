#!/usr/bin/env bash
#
# Dump the live Postgres schema for diagramming tools, two flavours:
#   docs/db/schema.sql            — exact pg_dump DDL (gitignored)
#   docs/db/schema.simplified.sql — clean CREATE TABLEs, FKs inlined
#
set -euo pipefail

DB_URL="${DATABASE_URL:-postgres://prospero:prospero@localhost:5433/prospero}"
PG_CONTAINER="${PG_CONTAINER:-prospero-pg}"
OUT_DIR="docs/db"
SCHEMA_SQL="$OUT_DIR/schema.sql"
SIMPLIFIED="$OUT_DIR/schema.simplified.sql"

mkdir -p "$OUT_DIR"

docker exec -e PGPASSWORD=prospero "$PG_CONTAINER" \
    pg_dump -d prospero -U prospero -s -F p -E UTF-8 > "$SCHEMA_SQL"

psql "$DB_URL" -At <<'SQL' > "$SIMPLIFIED"
WITH t AS (
  SELECT c.oid, c.relname
  FROM pg_class c
  JOIN pg_namespace n ON n.oid = c.relnamespace
  WHERE n.nspname = 'public' AND c.relkind = 'r' AND c.relname <> '_sqlx_migrations'
),
coldefs AS (
  SELECT t.oid,
    string_agg('  ' || a.attname || ' ' || format_type(a.atttypid, a.atttypmod)
      || CASE WHEN a.attnotnull THEN ' NOT NULL' ELSE '' END,
      E',\n' ORDER BY a.attnum) AS cols
  FROM t
  JOIN pg_attribute a ON a.attrelid = t.oid
  WHERE a.attnum > 0 AND NOT a.attisdropped
  GROUP BY t.oid
),
condefs AS (
  SELECT t.oid,
    string_agg('  ' || pg_get_constraintdef(con.oid),
      E',\n' ORDER BY con.contype DESC, con.conname) AS cons
  FROM t
  JOIN pg_constraint con ON con.conrelid = t.oid
  WHERE con.contype IN ('p', 'f')
  GROUP BY t.oid
)
SELECT 'CREATE TABLE ' || t.relname || ' (' || E'\n'
  || coldefs.cols
  || COALESCE(E',\n' || condefs.cons, '')
  || E'\n);' || E'\n'
FROM t
JOIN coldefs ON coldefs.oid = t.oid
LEFT JOIN condefs ON condefs.oid = t.oid
ORDER BY t.relname;
SQL

echo "wrote $SCHEMA_SQL and $SIMPLIFIED"
