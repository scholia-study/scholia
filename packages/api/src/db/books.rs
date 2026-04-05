use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::book::{BookDetail, BookSummary};

pub async fn list_books(pool: &PgPool) -> Result<Vec<BookSummary>, AppError> {
    let rows = sqlx::query_as!(
        BookRow,
        r#"SELECT b.id, b.slug, b.title, b.author, b.language, b.source, b.source_date,
                  b.source_book_id,
                  src.slug AS "source_book_slug?"
           FROM books b
           LEFT JOIN books src ON src.id = b.source_book_id
           ORDER BY b.title"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into_summary()).collect())
}

pub async fn get_book_by_slug(pool: &PgPool, slug: &str) -> Result<BookDetail, AppError> {
    let row = sqlx::query_as!(
        BookRow,
        r#"SELECT b.id, b.slug, b.title, b.author, b.language, b.source, b.source_date,
                  b.source_book_id,
                  src.slug AS "source_book_slug?"
           FROM books b
           LEFT JOIN books src ON src.id = b.source_book_id
           WHERE b.slug = $1"#,
        slug,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Book not found: {slug}")))?;

    let book_id: Uuid = row.id;
    let mut detail = row.into_detail();

    // Fetch translations (books where source_book_id = this book)
    let translation_rows = sqlx::query_as!(
        TranslationRow,
        r#"SELECT id, slug, title, author, language
           FROM books
           WHERE source_book_id = $1
           ORDER BY title"#,
        book_id,
    )
    .fetch_all(pool)
    .await?;

    detail.translations = translation_rows
        .into_iter()
        .map(|r| BookSummary {
            id: r.id.to_string(),
            slug: r.slug,
            title: r.title,
            author: r.author,
            language: r.language,
        })
        .collect();

    Ok(detail)
}

pub async fn get_book_id_by_slug(pool: &PgPool, slug: &str) -> Result<Uuid, AppError> {
    let id = sqlx::query_scalar!(
        r#"SELECT id FROM books WHERE slug = $1"#,
        slug,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Book not found: {slug}")))?;

    Ok(id)
}

struct BookRow {
    id: Uuid,
    slug: String,
    title: String,
    author: String,
    language: String,
    source: Option<String>,
    source_date: Option<String>,
    source_book_id: Option<Uuid>,
    source_book_slug: Option<String>,
}

impl BookRow {
    fn into_summary(self) -> BookSummary {
        BookSummary {
            id: self.id.to_string(),
            slug: self.slug,
            title: self.title,
            author: self.author,
            language: self.language,
        }
    }

    fn into_detail(self) -> BookDetail {
        BookDetail {
            id: self.id.to_string(),
            slug: self.slug,
            title: self.title,
            author: self.author,
            language: self.language,
            source: self.source,
            source_date: self.source_date,
            source_book_id: self.source_book_id.map(|id| id.to_string()),
            source_book_slug: self.source_book_slug,
            translations: vec![],
        }
    }
}

struct TranslationRow {
    id: Uuid,
    slug: String,
    title: String,
    author: String,
    language: String,
}
