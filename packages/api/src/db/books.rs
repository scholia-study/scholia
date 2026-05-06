use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::book::{BookDetail, BookSummary};

pub async fn list_books(pool: &PgPool) -> Result<Vec<BookSummary>, AppError> {
    let rows = sqlx::query_as!(
        BookRow,
        r#"SELECT b.id, b.slug, b.language,
                  b.source_id,
                  COALESCE(s.title_display, s.title) AS "title!",
                  s.publication_year,
                  s.publisher,

                  STRING_AGG(p.name, ', ' ORDER BY sp.position) AS author,
                  src_book.slug AS "source_book_slug?"
           FROM books b
           JOIN sources s ON s.id = b.source_id
           LEFT JOIN source_persons sp ON sp.source_id = s.id AND sp.role = 'author'
           LEFT JOIN persons p ON p.id = sp.person_id
           LEFT JOIN books src_book ON src_book.source_id = s.translation_of_id
           GROUP BY b.id, b.slug, b.language,
                    b.source_id, s.title_display, s.title,
                    s.publication_year, s.publisher,
                    src_book.slug
           ORDER BY COALESCE(s.title_display, s.title)"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into_summary()).collect())
}

pub async fn get_book_by_slug(pool: &PgPool, slug: &str) -> Result<BookDetail, AppError> {
    let row = sqlx::query_as!(
        BookRow,
        r#"SELECT b.id, b.slug, b.language,
                  b.source_id,
                  COALESCE(s.title_display, s.title) AS "title!",
                  s.publication_year,
                  s.publisher,

                  STRING_AGG(p.name, ', ' ORDER BY sp.position) AS author,
                  src_book.slug AS "source_book_slug?"
           FROM books b
           JOIN sources s ON s.id = b.source_id
           LEFT JOIN source_persons sp ON sp.source_id = s.id AND sp.role = 'author'
           LEFT JOIN persons p ON p.id = sp.person_id
           LEFT JOIN books src_book ON src_book.source_id = s.translation_of_id
           WHERE b.slug = $1
           GROUP BY b.id, b.slug, b.language,
                    b.source_id, s.title_display, s.title,
                    s.publication_year, s.publisher,
                    src_book.slug"#,
        slug,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Book not found: {slug}")))?;

    let source_id: Uuid = row.source_id;
    let mut detail = row.into_detail();

    // Fetch translations: books whose source has translation_of_id = this book's source
    let translation_rows = sqlx::query_as!(
        TranslationRow,
        r#"SELECT b.id, b.slug, b.language,
                  COALESCE(s.title_display, s.title) AS "title!",
                  STRING_AGG(p.name, ', ' ORDER BY sp.position) AS author
           FROM books b
           JOIN sources s ON s.id = b.source_id
           LEFT JOIN source_persons sp ON sp.source_id = s.id AND sp.role = 'author'
           LEFT JOIN persons p ON p.id = sp.person_id
           WHERE s.translation_of_id = $1
           GROUP BY b.id, b.slug, b.language, s.title_display, s.title
           ORDER BY COALESCE(s.title_display, s.title)"#,
        source_id,
    )
    .fetch_all(pool)
    .await?;

    detail.translations = translation_rows
        .into_iter()
        .map(|r| BookSummary {
            id: r.id.to_string(),
            slug: r.slug,
            title: r.title,
            author: r.author.unwrap_or_default(),
            language: r.language,
        })
        .collect();

    // Sibling translations — peers under the same translation root,
    // excluding self. Used by Bible-shape books (no hosted source
    // language) so the reader can offer a flat translation picker.
    // Resolves the root via COALESCE so the query works whether the
    // current book is a translation (root = our translation_of_id) or
    // is itself a root with siblings (root = our id; rare in practice).
    let sibling_rows = sqlx::query_as!(
        TranslationRow,
        r#"WITH me AS (
               SELECT COALESCE(s.translation_of_id, s.id) AS root_id
               FROM books b
               JOIN sources s ON s.id = b.source_id
               WHERE b.id = $1
           )
           SELECT b.id, b.slug, b.language,
                  COALESCE(s.title_display, s.title) AS "title!",
                  STRING_AGG(p.name, ', ' ORDER BY sp.position) AS author
           FROM books b
           JOIN sources s ON s.id = b.source_id
           LEFT JOIN source_persons sp ON sp.source_id = s.id AND sp.role = 'author'
           LEFT JOIN persons p ON p.id = sp.person_id
           WHERE COALESCE(s.translation_of_id, s.id) = (SELECT root_id FROM me)
             AND b.id != $1
           GROUP BY b.id, b.slug, b.language, s.title_display, s.title
           ORDER BY COALESCE(s.title_display, s.title)"#,
        Uuid::parse_str(&detail.id).expect("detail.id is a valid UUID"),
    )
    .fetch_all(pool)
    .await?;
    detail.sibling_translations = sibling_rows
        .into_iter()
        .map(|r| BookSummary {
            id: r.id.to_string(),
            slug: r.slug,
            title: r.title,
            author: r.author.unwrap_or_default(),
            language: r.language,
        })
        .collect();

    Ok(detail)
}

pub async fn get_book_id_by_slug(pool: &PgPool, slug: &str) -> Result<Uuid, AppError> {
    let id = sqlx::query_scalar!(r#"SELECT id FROM books WHERE slug = $1"#, slug,)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Book not found: {slug}")))?;

    Ok(id)
}

struct BookRow {
    id: Uuid,
    slug: String,
    title: String,
    language: String,
    source_id: Uuid,
    publication_year: Option<i16>,
    publisher: Option<String>,
    author: Option<String>,
    source_book_slug: Option<String>,
}

impl BookRow {
    fn into_summary(self) -> BookSummary {
        BookSummary {
            id: self.id.to_string(),
            slug: self.slug,
            title: self.title,
            author: self.author.unwrap_or_default(),
            language: self.language,
        }
    }

    fn into_detail(self) -> BookDetail {
        BookDetail {
            id: self.id.to_string(),
            slug: self.slug,
            title: self.title,
            author: self.author.unwrap_or_default(),
            language: self.language,
            source_id: self.source_id.to_string(),
            publication_year: self.publication_year,
            publisher: self.publisher,
            source_book_slug: self.source_book_slug,
            translations: vec![],
            sibling_translations: vec![],
        }
    }
}

struct TranslationRow {
    id: Uuid,
    slug: String,
    title: String,
    author: Option<String>,
    language: String,
}
