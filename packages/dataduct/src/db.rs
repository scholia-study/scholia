use sqlx::postgres::PgConnectOptions;

/// Build sqlx connect options. CLI `--database-url` wins; otherwise
/// prefer discrete `POSTGRES_*` env vars (k8s Secret pattern — avoids
/// the URL-special-char trap when `$(VAR)` substitution is literal);
/// fall back to `DATABASE_URL` for laptop `.env`.
pub fn pg_connect_options(
    cli_url: Option<String>,
) -> Result<PgConnectOptions, Box<dyn std::error::Error>> {
    if let Some(url) = cli_url {
        return Ok(url.parse()?);
    }
    if let Ok(user) = std::env::var("POSTGRES_USER") {
        let password = std::env::var("POSTGRES_PASSWORD")
            .map_err(|_| "POSTGRES_PASSWORD must be set when POSTGRES_USER is set")?;
        let database = std::env::var("POSTGRES_DB")
            .map_err(|_| "POSTGRES_DB must be set when POSTGRES_USER is set")?;
        let host = std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port: u16 = std::env::var("POSTGRES_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5432);
        return Ok(PgConnectOptions::new()
            .username(&user)
            .password(&password)
            .database(&database)
            .host(&host)
            .port(port));
    }
    let url = std::env::var("DATABASE_URL").map_err(
        |_| "Set POSTGRES_USER + POSTGRES_PASSWORD + POSTGRES_DB (preferred) or DATABASE_URL",
    )?;
    Ok(url.parse()?)
}
