//! Cache PURGE helpers shared by the ingest binaries (synchronous,
//! `purge_blocking`) and the API (fire-and-forget, builds on `send_purge`).
//!
//! The proxy's cache
//! key is `$request_uri`, so a trailing `*` on a path means prefix match.

/// Cache paths to invalidate after importing `slug`: the global listings plus
/// the book's own subtree (wildcard `*` = prefix match on the `$request_uri`
/// cache key) so a reconciled edit shows immediately instead of waiting out the
/// TTL.
pub fn purge_paths(slug: &str) -> Vec<String> {
    vec![
        "/api/library".into(),
        "/api/books".into(),
        "/books".into(),
        format!("/api/books/{slug}*"),
        format!("/books/{slug}*"),
    ]
}

/// Send one PURGE; returns the status (no logging — callers log in their own
/// style). `base` may have a trailing slash.
pub async fn send_purge(
    client: &reqwest::Client,
    base: &str,
    path: &str,
) -> Result<reqwest::StatusCode, reqwest::Error> {
    let url = format!("{}{}", base.trim_end_matches('/'), path);
    let method = reqwest::Method::from_bytes(b"PURGE").expect("PURGE is a valid method");
    Ok(client.request(method, &url).send().await?.status())
}

/// Synchronous ingest purge: awaits every PURGE (the binary exits right after,
/// so spawned tasks would be killed). No-op if `CACHE_PURGE_URL` is unset/empty
/// (local dev without a proxy). Best-effort: failures are logged to stderr,
/// never returned — a missed PURGE means stale-until-TTL, not a broken import.
pub async fn purge_blocking(paths: &[String]) {
    let Ok(base) = std::env::var("CACHE_PURGE_URL") else {
        return;
    };
    if base.is_empty() {
        return;
    }
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("PURGE client init failed: {e} (skipping cache invalidation)");
            return;
        }
    };
    for path in paths {
        match send_purge(&client, &base, path).await {
            Ok(status) => eprintln!("PURGE {} → {}", path, status),
            Err(e) => eprintln!("PURGE {} failed: {}", path, e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::purge_paths;

    #[test]
    fn purge_paths_lists_globals_and_book_subtree() {
        assert_eq!(
            purge_paths("kjv-bible"),
            vec![
                "/api/library".to_string(),
                "/api/books".to_string(),
                "/books".to_string(),
                "/api/books/kjv-bible*".to_string(),
                "/books/kjv-bible*".to_string(),
            ]
        );
    }
}
