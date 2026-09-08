use axum::Json;
use axum::extract::{Multipart, State};
use image::ImageDecoder;
use sha2::{Digest, Sha256};

use crate::modules::writing::article_images::db::ArticleImageEntry;
use crate::modules::writing::article_images::models::{UploadImageForm, UploadedImageResponse};
use crate::system::auth::middleware::AuthUser;
use crate::system::auth::permissions::Permission;
use crate::system::error::AppError;
use crate::system::state::AppState;

/// Raw upload cap, enforced both by the route's `DefaultBodyLimit` and
/// explicitly here for a readable error message.
pub const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;

/// Longest image side after processing; larger uploads are downscaled.
const MAX_DIMENSION: u32 = 2000;

/// Lossy WebP quality for the stored rendition.
const WEBP_QUALITY: f32 = 82.0;

/// Upload an image for embedding in an article figure
///
/// Accepts JPEG, PNG, or WebP up to 5 MB. The image is re-encoded
/// server-side (downscaled to at most 2000px on the longest side, lossy
/// WebP, metadata stripped) and stored under a content-addressed key.
#[utoipa::path(
    post,
    path = "/api/user/article-images",
    request_body(content = UploadImageForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Image stored", body = UploadedImageResponse),
        (status = 400, description = "Missing, oversized, or invalid image"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "article-images"
)]
pub async fn upload_article_image(
    State(state): State<AppState>,
    user: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<UploadedImageResponse>, AppError> {
    user.require_permission(Permission::ArticlesCreate)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let mut file: Option<(Option<String>, Vec<u8>)> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::BadRequest("Malformed upload.".into()))?
    {
        if field.name() == Some("file") {
            let filename = field.file_name().map(|s| s.to_string());
            let bytes = field
                .bytes()
                .await
                .map_err(|_| AppError::BadRequest("Image exceeds the 5 MB upload limit.".into()))?;
            file = Some((filename, bytes.to_vec()));
            break;
        }
    }
    let (original_filename, bytes) =
        file.ok_or_else(|| AppError::BadRequest("Missing `file` field.".into()))?;

    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err(AppError::BadRequest(
            "Image exceeds the 5 MB upload limit.".into(),
        ));
    }

    let (webp_bytes, width, height) = tokio::task::spawn_blocking(move || process_image(&bytes))
        .await
        .map_err(|e| AppError::Internal(format!("Image processing task failed: {e}")))??;

    let hash = hex::encode(Sha256::digest(&webp_bytes));
    let storage_key = format!("articles/{}/{}.webp", user.id, &hash[..16]);
    let byte_size = webp_bytes.len() as i32;

    state
        .media
        .put(&storage_key, webp_bytes, "image/webp")
        .await
        .map_err(|e| AppError::Internal(format!("Media storage write failed: {e}")))?;

    crate::modules::writing::article_images::db::insert_article_image(
        &state.pool,
        &ArticleImageEntry {
            user_id: user.id,
            storage_key: &storage_key,
            original_filename: original_filename.as_deref(),
            content_type: "image/webp",
            byte_size,
            width: width as i32,
            height: height as i32,
        },
    )
    .await?;

    Ok(Json(UploadedImageResponse {
        url: format!("/media/{storage_key}"),
        storage_key,
        width: width as i32,
        height: height as i32,
        byte_size,
    }))
}

/// Decode, orient, downscale, and re-encode an upload as lossy WebP.
/// Re-encoding strips all metadata (EXIF/GPS); the decoder's orientation
/// is applied first so phone photos don't come out sideways. Decoder
/// limits guard against decompression bombs.
fn process_image(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), AppError> {
    let format = image::guess_format(bytes)
        .map_err(|_| AppError::BadRequest("Unrecognized image format.".into()))?;
    if !matches!(
        format,
        image::ImageFormat::Jpeg | image::ImageFormat::Png | image::ImageFormat::WebP
    ) {
        return Err(AppError::BadRequest(
            "Only JPEG, PNG, and WebP images are accepted.".into(),
        ));
    }

    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes));
    reader.set_format(format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(12_000);
    limits.max_image_height = Some(12_000);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);

    let mut decoder = reader
        .into_decoder()
        .map_err(|_| AppError::BadRequest("Image could not be decoded.".into()))?;
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut img = image::DynamicImage::from_decoder(decoder)
        .map_err(|_| AppError::BadRequest("Image could not be decoded.".into()))?;
    img.apply_orientation(orientation);

    if img.width() > MAX_DIMENSION || img.height() > MAX_DIMENSION {
        img = img.thumbnail(MAX_DIMENSION, MAX_DIMENSION);
    }

    let rgba = img.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());
    let webp_bytes = webp::Encoder::from_rgba(&rgba, width, height)
        .encode(WEBP_QUALITY)
        .to_vec();
    Ok((webp_bytes, width, height))
}

#[cfg(test)]
mod tests {
    use super::process_image;

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            width,
            height,
            image::Rgba([120, 80, 200, 255]),
        ));
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png)
            .expect("encode test png");
        out.into_inner()
    }

    #[test]
    fn reencodes_to_webp() {
        let (webp, w, h) = process_image(&png_bytes(400, 300)).expect("processes");
        assert_eq!((w, h), (400, 300));
        assert_eq!(
            image::guess_format(&webp).unwrap(),
            image::ImageFormat::WebP
        );
    }

    #[test]
    fn downscales_to_max_dimension() {
        let (_, w, h) = process_image(&png_bytes(3000, 1500)).expect("processes");
        assert_eq!((w, h), (2000, 1000));
    }

    #[test]
    fn never_upscales() {
        let (_, w, h) = process_image(&png_bytes(120, 80)).expect("processes");
        assert_eq!((w, h), (120, 80));
    }

    #[test]
    fn rejects_non_image_bytes() {
        assert!(process_image(b"<svg xmlns='http://www.w3.org/2000/svg'/>").is_err());
        assert!(process_image(&[0u8; 64]).is_err());
    }
}
