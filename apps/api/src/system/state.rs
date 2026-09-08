use sqlx::PgPool;

use crate::system::config::AppConfig;
use crate::system::media::MediaStore;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub stripe: stripe::Client,
    /// Shared HTTP client used by `cache::invalidate` for fire-and-forget
    /// PURGE requests to the proxy. Cloning is cheap (Arc internally).
    pub purge_client: reqwest::Client,
    /// Object storage for user-uploaded media (article figure images).
    pub media: MediaStore,
}
