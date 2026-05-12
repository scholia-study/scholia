use std::env;

/// Resolve a facsimile storage key to a public image URL.
///
/// Returns `None` until `FACSIMILE_BUCKET_BASE_URL` is set in the
/// environment, so callers can plumb the field through and the frontend
/// will simply not render a link. Once the bucket is wired, the helper
/// concatenates `<base>/<storage_key>` (a single slash, regardless of
/// whether the base has a trailing one).
pub fn resolve_url(storage_key: &str) -> Option<String> {
    let base = env::var("FACSIMILE_BUCKET_BASE_URL").ok()?;
    Some(format!("{}/{}", base.trim_end_matches('/'), storage_key))
}
