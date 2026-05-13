//! Fire-and-forget cache invalidation. POSTs PURGE to the proxy's
//! cluster-internal admin port after writes that change cached content.
//!
//! Failures are logged, not returned. The cache is best-effort — a
//! missed PURGE means content is stale until its TTL elapses, not a
//! broken write. Handlers must not await this call.
//!
//! If `CACHE_PURGE_URL` is unset (local dev without proxy, ingest jobs
//! that don't talk to the cluster, etc.) all invalidations no-op.

use std::time::Duration;

use reqwest::Method;

use crate::state::AppState;

/// Build the shared reqwest client used for PURGE requests. Created
/// once at startup and held on `AppState`. The short timeout exists
/// because we never want a write handler to block on a misbehaving
/// or unreachable cache.
pub fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("failed to build cache PURGE client")
}

/// Fire-and-forget PURGE for each path. Returns immediately; each
/// request runs in its own tokio task.
///
/// `paths` are URI paths exactly as a public request would arrive at
/// the proxy — e.g. `/articles/foo`, `/api/articles`. They become the
/// cache key (which is `$request_uri` per the proxy config).
pub fn invalidate<I, S>(state: &AppState, paths: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let Some(base) = state.config.cache_purge_url.as_ref() else {
        return;
    };
    let base = base.trim_end_matches('/').to_owned();

    for path in paths {
        let path = path.as_ref().to_owned();
        let url = format!("{base}{path}");
        let client = state.purge_client.clone();
        tokio::spawn(async move {
            let method = Method::from_bytes(b"PURGE").expect("PURGE is a valid method");
            match client.request(method, &url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    // 412 Precondition Failed = key not in cache. Not
                    // an error — just nothing to do.
                    if status.is_success() || status.as_u16() == 412 {
                        tracing::debug!(target: "cache", "PURGE {path} → {status}");
                    } else {
                        tracing::warn!(target: "cache", "PURGE {path} → {status}");
                    }
                }
                Err(e) => tracing::warn!(target: "cache", "PURGE {path} failed: {e}"),
            }
        });
    }
}
