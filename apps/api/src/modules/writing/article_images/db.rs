use sqlx::PgPool;
use uuid::Uuid;

use crate::system::error::AppError;

pub struct ArticleImageEntry<'a> {
    pub user_id: Uuid,
    pub storage_key: &'a str,
    pub original_filename: Option<&'a str>,
    pub content_type: &'a str,
    pub byte_size: i32,
    pub width: i32,
    pub height: i32,
}

/// Record an uploaded image. Keys are content-addressed, so re-uploading
/// the same image is a no-op that resolves to the existing object.
pub async fn insert_article_image(
    pool: &PgPool,
    entry: &ArticleImageEntry<'_>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO article_images
             (user_id, storage_key, original_filename, content_type,
              byte_size, width, height)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (storage_key) DO NOTHING",
    )
    .bind(entry.user_id)
    .bind(entry.storage_key)
    .bind(entry.original_filename)
    .bind(entry.content_type)
    .bind(entry.byte_size)
    .bind(entry.width)
    .bind(entry.height)
    .execute(pool)
    .await?;
    Ok(())
}
