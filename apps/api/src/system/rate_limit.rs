//! Rate-limiting policy for the auth tier.
//!
//! The *policy* (rates, key extraction) lives here; the *wiring* — turning
//! the config into a `GovernorLayer` and attaching it to the auth router —
//! stays at the router-assembly call site in `crate::api_router`.

use std::sync::Arc;

use governor::middleware::NoOpMiddleware;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::key_extractor::SmartIpKeyExtractor;

/// Auth endpoints are limited per client IP.
///
/// `SmartIpKeyExtractor` reads X-Forwarded-For (leftmost entry) before
/// falling back to the TCP peer address. In the cluster the peer is
/// always the nginx proxy — one shared bucket for the whole internet —
/// so the forwarded header is the only usable per-client key. It is
/// trustworthy ONLY because the proxy resolves the real client via the
/// realip module and collapses X-Forwarded-For to that single value
/// (see apps/proxy/templates/default.conf.template); never ship this
/// extractor behind an edge that forwards client-supplied XFF verbatim.
/// Local dev hits the API directly with no XFF header, where the peer
/// fallback keeps the limiter keyed on the real client.
type AuthGovernorConfig = GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware>;

/// Build the governor config for auth endpoints: ~10 requests per 60s per
/// IP (burst of 10, refilling one slot every 6 seconds).
pub fn auth_config() -> Arc<AuthGovernorConfig> {
    Arc::new(
        GovernorConfigBuilder::default()
            .per_second(6)
            .burst_size(10)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("Failed to build governor config"),
    )
}
