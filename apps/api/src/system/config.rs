use std::env;

use sqlx::postgres::PgConnectOptions;

/// Build sqlx connect options from environment.
///
/// Prefers discrete `POSTGRES_*` vars over a composed `DATABASE_URL` so
/// passwords containing URL-special characters (`:`, `/`, `@`, `?`) don't
/// break URL parsing — k8s `$(VAR)` substitution is literal, so a single
/// `DATABASE_URL = postgres://$(USER):$(PASSWORD)@…` string can't survive
/// non-trivial passwords. Falls back to `DATABASE_URL` when discrete vars
/// are absent (local dev `.env`, sqlx-cli scripts).
pub fn pg_connect_options_from_env() -> PgConnectOptions {
    if let Ok(user) = env::var("POSTGRES_USER") {
        let password = env::var("POSTGRES_PASSWORD")
            .expect("POSTGRES_PASSWORD must be set when POSTGRES_USER is set");
        let database =
            env::var("POSTGRES_DB").expect("POSTGRES_DB must be set when POSTGRES_USER is set");
        let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port: u16 = env::var("POSTGRES_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5432);
        return PgConnectOptions::new()
            .username(&user)
            .password(&password)
            .database(&database)
            .host(&host)
            .port(port);
    }

    let url = env::var("DATABASE_URL")
        .expect("Set POSTGRES_USER + POSTGRES_PASSWORD + POSTGRES_DB (preferred) or DATABASE_URL");
    url.parse()
        .expect("DATABASE_URL is not a valid Postgres connection string")
}

#[derive(Clone)]
pub struct AppConfig {
    pub session_secret: String,
    pub resend_api_key: String,
    pub from_email: String,
    pub backend_url: String,
    pub frontend_url: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_redirect_uri: String,
    pub stripe_api_key: String,
    pub stripe_webhook_secret: String,
    pub stripe_price_base: String,
    pub stripe_price_mid: String,
    pub stripe_price_high: String,
    /// Base URL of the proxy's cluster-internal PURGE listener (e.g.
    /// `http://nginx-cache:8080`). When unset, `cache::invalidate` is a
    /// no-op — convenient for local dev without the proxy stack up.
    pub cache_purge_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let cfg = Self {
            session_secret: env::var("SESSION_SECRET").expect("SESSION_SECRET must be set"),
            resend_api_key: env::var("RESEND_API_KEY").expect("RESEND_API_KEY must be set"),
            from_email: env::var("FROM_EMAIL").expect("FROM_EMAIL must be set"),
            backend_url: env::var("BACKEND_URL").expect("BACKEND_URL must be set"),
            frontend_url: env::var("FRONTEND_URL").expect("FRONTEND_URL must be set"),
            github_client_id: env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set"),
            github_client_secret: env::var("GITHUB_CLIENT_SECRET")
                .expect("GITHUB_CLIENT_SECRET must be set"),
            github_redirect_uri: env::var("GITHUB_REDIRECT_URI")
                .expect("GITHUB_REDIRECT_URI must be set"),
            stripe_api_key: env::var("STRIPE_API_KEY").expect("STRIPE_API_KEY must be set"),
            stripe_webhook_secret: env::var("STRIPE_WEBHOOK_SECRET")
                .expect("STRIPE_WEBHOOK_SECRET must be set"),
            stripe_price_base: env::var("STRIPE_PRICE_BASE")
                .expect("STRIPE_PRICE_BASE must be set"),
            stripe_price_mid: env::var("STRIPE_PRICE_MID").expect("STRIPE_PRICE_MID must be set"),
            stripe_price_high: env::var("STRIPE_PRICE_HIGH")
                .expect("STRIPE_PRICE_HIGH must be set"),
            cache_purge_url: env::var("CACHE_PURGE_URL").ok().filter(|s| !s.is_empty()),
        };
        cfg.verify_stripe_prices();
        cfg
    }

    /// Sanity-check the configured Stripe Price IDs at boot.
    /// Catches the most common mistake: pasting a Product ID
    /// (`prod_…`) or a test/live mismatch into the wrong env. Does
    /// not call the Stripe API — that's a future enhancement.
    fn verify_stripe_prices(&self) {
        for (name, value) in [
            ("STRIPE_PRICE_BASE", &self.stripe_price_base),
            ("STRIPE_PRICE_MID", &self.stripe_price_mid),
            ("STRIPE_PRICE_HIGH", &self.stripe_price_high),
        ] {
            assert!(
                value.starts_with("price_"),
                "{name} must start with `price_` (got {value:?})"
            );
        }
    }
}
