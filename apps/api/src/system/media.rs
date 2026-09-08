//! Object storage for user-uploaded media (article figure images).
//!
//! Backend is selected at boot: `MEDIA_BUCKET` set means the Hetzner
//! S3-compatible bucket (cluster mode — nginx serves `/media/*` straight
//! from the bucket); otherwise a local filesystem directory, with the
//! `/media/{*key}` route below doing the serving. Keys stored in the
//! database are logical (`articles/<user>/<hash>.webp`), so the backend
//! can change without touching rows or rendered article HTML.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use object_store::aws::AmazonS3Builder;
use object_store::local::LocalFileSystem;
use object_store::path::Path as ObjectPath;
use object_store::{Attribute, Attributes, GetOptions, ObjectStore, PutOptions, PutPayload};

use crate::system::state::AppState;

#[derive(Clone)]
pub struct MediaStore {
    store: Arc<dyn ObjectStore>,
    s3: bool,
}

impl MediaStore {
    pub fn from_env() -> Self {
        match std::env::var("MEDIA_BUCKET").ok().filter(|s| !s.is_empty()) {
            Some(bucket) => {
                let endpoint = std::env::var("MEDIA_S3_ENDPOINT")
                    .unwrap_or_else(|_| "https://fsn1.your-objectstorage.com".to_string());
                let region =
                    std::env::var("MEDIA_S3_REGION").unwrap_or_else(|_| "fsn1".to_string());
                // Credentials come from AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY
                // (the cluster-wide Hetzner keypair) via from_env. Hetzner is
                // path-style only.
                let store = AmazonS3Builder::from_env()
                    .with_bucket_name(&bucket)
                    .with_endpoint(&endpoint)
                    .with_region(&region)
                    .with_virtual_hosted_style_request(false)
                    .build()
                    .expect("Invalid media bucket configuration");
                Self {
                    store: Arc::new(store),
                    s3: true,
                }
            }
            None => {
                let root = std::env::var("MEDIA_LOCAL_DIR")
                    .ok()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| ".local/media".to_string());
                Self::local(&root)
            }
        }
    }

    fn local(root: &str) -> Self {
        std::fs::create_dir_all(root)
            .unwrap_or_else(|e| panic!("Cannot create media dir {root}: {e}"));
        let store = LocalFileSystem::new_with_prefix(root)
            .unwrap_or_else(|e| panic!("Cannot open media dir {root}: {e}"));
        Self {
            store: Arc::new(store),
            s3: false,
        }
    }

    pub async fn put(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), object_store::Error> {
        let path = ObjectPath::parse(key)?;
        let payload = PutPayload::from(bytes);
        if self.s3 {
            let mut attributes = Attributes::new();
            attributes.insert(Attribute::ContentType, content_type.to_string().into());
            let opts = PutOptions {
                attributes,
                ..Default::default()
            };
            self.store.put_opts(&path, payload, opts).await?;
        } else {
            // LocalFileSystem rejects attributes; the serving route infers
            // the content type from the key instead.
            self.store
                .put_opts(&path, payload, PutOptions::default())
                .await?;
        }
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Bytes, object_store::Error> {
        let path = ObjectPath::parse(key)?;
        self.store
            .get_opts(&path, GetOptions::default())
            .await?
            .bytes()
            .await
    }
}

/// Undocumented media-serving route. In cluster the nginx `/media/`
/// location serves the bucket directly and this route is unreachable; in
/// local dev (filesystem backend, vite proxying `/media` to the API) it is
/// the serving path.
pub fn routes() -> axum::Router<AppState> {
    axum::Router::new().route("/media/{*key}", axum::routing::get(serve_media))
}

async fn serve_media(State(state): State<AppState>, Path(key): Path<String>) -> Response {
    let content_type = if key.ends_with(".webp") {
        "image/webp"
    } else {
        "application/octet-stream"
    };
    match state.media.get(&key).await {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, content_type),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::MediaStore;

    #[tokio::test]
    async fn local_put_get_roundtrip() {
        let dir = std::env::temp_dir().join(format!("scholia-media-test-{}", std::process::id()));
        let store = MediaStore::local(dir.to_str().unwrap());

        let key = "articles/user/0123456789abcdef.webp";
        store
            .put(key, b"webp-bytes".to_vec(), "image/webp")
            .await
            .expect("put");
        let bytes = store.get(key).await.expect("get");
        assert_eq!(&bytes[..], b"webp-bytes");

        assert!(store.get("articles/user/missing.webp").await.is_err());
        // Traversal-shaped keys must not resolve.
        assert!(store.get("../outside").await.is_err());

        std::fs::remove_dir_all(&dir).ok();
    }
}
