//! Rate-limiting policy for the auth tier.
//!
//! The *policy* (rates, key extraction) lives here; the *wiring* — turning
//! the config into a `GovernorLayer` and attaching it to the auth router —
//! stays at the router-assembly call site in `crate::api_router`.

use std::sync::Arc;

use governor::middleware::NoOpMiddleware;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::key_extractor::PeerIpKeyExtractor;

/// Auth endpoints are limited per client IP using the default peer-IP key
/// extractor and no extra response middleware.
type AuthGovernorConfig = GovernorConfig<PeerIpKeyExtractor, NoOpMiddleware>;

/// Build the governor config for auth endpoints: ~10 requests per 60s per
/// IP (burst of 10, refilling one slot every 6 seconds).
pub fn auth_config() -> Arc<AuthGovernorConfig> {
    Arc::new(
        GovernorConfigBuilder::default()
            .per_second(6)
            .burst_size(10)
            .finish()
            .expect("Failed to build governor config"),
    )
}
