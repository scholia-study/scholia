// sqlx migration runner.
//
// The migrations live in db/migrations/ at the repo root. They're embedded
// into the binary at compile time by sqlx::migrate!, so the runtime image
// doesn't need the SQL files on disk — the init container in cluster just
// runs `api migrate`.
//
// Append-only invariant: never edit a migration that has already been
// applied to any database. sqlx tracks checksums in _sqlx_migrations and
// refuses to proceed if a file's hash drifts from what was recorded.

use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgConnectOptions;
use tower_sessions_sqlx_store::PostgresStore;

pub static MIGRATOR: Migrator = sqlx::migrate!("../../db/migrations");

pub async fn run(options: PgConnectOptions) -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect_with(options).await?;
    MIGRATOR.run(&pool).await?;

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;

    // Grant the restricted runtime role DML on the session table — but only
    // when that role exists.
    sqlx::query(
        r#"DO $$ BEGIN
             IF EXISTS (SELECT FROM pg_roles WHERE rolname = 'scholia_app') THEN
               GRANT USAGE ON SCHEMA tower_sessions TO scholia_app;
               GRANT SELECT, INSERT, UPDATE, DELETE ON tower_sessions.session TO scholia_app;
             END IF;
           END $$;"#,
    )
    .execute(&pool)
    .await?;

    Ok(())
}
