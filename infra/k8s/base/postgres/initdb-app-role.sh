#!/bin/sh
# Runs ONCE, on a fresh initdb. Inert on an already-initialized data directory.
# Re-provision the DB (fresh PVC) for this to run.
set -e

if [ -z "$SCHOLIA_APP_DB_PASSWORD" ]; then
  echo "initdb-app-role: SCHOLIA_APP_DB_PASSWORD is not set — refusing to create a role with no password" >&2
  exit 1
fi

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" \
     -v pw="$SCHOLIA_APP_DB_PASSWORD" <<'EOSQL'
  CREATE ROLE scholia_app LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION
    PASSWORD :'pw';
  GRANT USAGE ON SCHEMA public TO scholia_app;
  ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO scholia_app;
  ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT USAGE, SELECT ON SEQUENCES TO scholia_app;
  ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT EXECUTE ON FUNCTIONS TO scholia_app;
EOSQL

echo "initdb-app-role: created role scholia_app"
