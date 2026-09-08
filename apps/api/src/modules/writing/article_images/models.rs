use serde::Serialize;
use utoipa::ToSchema;

/// Documentation-only shape of the multipart upload body; the handler
/// reads the `file` part via `axum::extract::Multipart`.
#[derive(ToSchema)]
#[allow(dead_code)]
pub struct UploadImageForm {
    #[schema(value_type = String, format = Binary)]
    pub file: String,
}

#[derive(Serialize, ToSchema)]
pub struct UploadedImageResponse {
    /// First-party URL to embed in the `::figure{}` directive.
    pub url: String,
    pub storage_key: String,
    pub width: i32,
    pub height: i32,
    /// Stored (re-encoded) size in bytes.
    pub byte_size: i32,
}
