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

pub static MIGRATOR: Migrator = sqlx::migrate!("../../db/migrations");

pub async fn run(options: PgConnectOptions) -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect_with(options).await?;
    MIGRATOR.run(&pool).await?;
    Ok(())
}
