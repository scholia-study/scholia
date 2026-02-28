psql "postgres://prospero:prospero@localhost:5433/prospero" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;" && \
psql "postgres://prospero:prospero@localhost:5433/prospero" -f db/001_schema.sql