use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::book::{BookDetail, BookSummary};

pub async fn list_books(pool: &PgPool) -> Result<Vec<BookSummary>, AppError> {
    let rows = sqlx::query_as!(
        BookRow,
        r#"SELECT id, slug, title, author, language, source, source_date
           FROM books
           ORDER BY title"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into_summary()).collect())
}

pub async fn get_book_by_slug(pool: &PgPool, slug: &str) -> Result<BookDetail, AppError> {
    let row = sqlx::query_as!(
        BookRow,
        r#"SELECT id, slug, title, author, language, source, source_date
           FROM books
           WHERE slug = $1"#,
        slug,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Book not found: {slug}")))?;

    Ok(row.into_detail())
}

struct BookRow {
    id: Uuid,
    slug: String,
    title: String,
    author: String,
    language: String,
    source: Option<String>,
    source_date: Option<String>,
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
        }
    }
}
