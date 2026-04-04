---
name: Use db_reset for schema changes
description: Modify 001_schema.sql directly and use db_reset script instead of creating separate migration files
type: feedback
---

Modify the main schema file (db/001_schema.sql) directly instead of creating new migration files. Use `./db_reset.sh` to reset the local dev database.

**Why:** Local dev workflow — no need for incremental migrations, just reset and re-apply.
**How to apply:** When making schema changes, edit db/001_schema.sql directly. Don't create 002_*.sql etc.
