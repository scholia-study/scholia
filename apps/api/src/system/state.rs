use sqlx::PgPool;

use crate::system::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub stripe: stripe::Client,
    /// Shared HTTP client used by `cache::invalidate` for fire-and-forget
    /// PURGE requests to the proxy. Cloning is cheap (Arc internally).
    pub purge_client: reqwest::Client,
}
