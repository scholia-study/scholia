use sqlx::PgPool;
use uuid::Uuid;

use crate::db::{citations, sources};
use crate::error::AppError;
use crate::models::book::{AboutThisTextResponse, BookDetail, BookSummary};

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

/// Resolve the bibliographic source for the "About this text" panel.
///
/// With `node_slug`, walks the toc-ancestor chain looking for the
/// deepest non-null `toc_nodes.source_id` (Bible-shape books-within-
/// books) and falls back to `books.source_id`. Without a node, uses
/// `books.source_id` directly. Also surfaces the hosted source-
/// language book when the resolved source is a translation of one.
pub async fn get_about_this_text(
    pool: &PgPool,
    book_slug: &str,
    node_slug: Option<&str>,
) -> Result<AboutThisTextResponse, AppError> {
    let book = sqlx::query!(
        r#"SELECT id, source_id, about_text FROM books WHERE slug = $1"#,
        book_slug,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Book not found: {book_slug}")))?;

    let (effective_source_id, resolved_from) = if let Some(slug) = node_slug {
        let node_id = sqlx::query_scalar!(
            r#"SELECT id FROM toc_nodes WHERE book_id = $1 AND slug = $2"#,
            book.id,
            slug,
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Node not found: {slug}")))?;

        let resolved = citations::resolve_effective_source(pool, book.id, node_id).await?;
        let from = if resolved == book.source_id {
            "book"
        } else {
            "node"
        };
        (resolved, from)
    } else {
        (book.source_id, "book")
    };

    let source = sources::get_source(pool, effective_source_id).await?;

    // If this source is a translation and the original is hosted, attach
    // a BookSummary for the source-language book so the panel can link to it.
    let source_book = if let Some(translation_of_id_str) = &source.translation_of_id {
        let translation_of_id = Uuid::parse_str(translation_of_id_str)
            .map_err(|_| AppError::Internal("invalid translation_of_id".into()))?;
        sqlx::query_as!(
            SourceBookRow,
            r#"SELECT b.id, b.slug, b.language,
                      COALESCE(s.title_display, s.title) AS "title!",
                      STRING_AGG(p.name, ', ' ORDER BY sp.position) AS author
               FROM books b
               JOIN sources s ON s.id = b.source_id
               LEFT JOIN source_persons sp ON sp.source_id = s.id AND sp.role = 'author'
               LEFT JOIN persons p ON p.id = sp.person_id
               WHERE b.source_id = $1
               GROUP BY b.id, b.slug, b.language, s.title_display, s.title
               LIMIT 1"#,
            translation_of_id,
        )
        .fetch_optional(pool)
        .await?
        .map(|r| BookSummary {
            id: r.id.to_string(),
            slug: r.slug,
            title: r.title,
            author: r.author.unwrap_or_default(),
            language: r.language,
        })
    } else {
        None
    };

    Ok(AboutThisTextResponse {
        source,
        resolved_from: resolved_from.to_string(),
        source_book,
        about_text: book.about_text,
    })
}

struct SourceBookRow {
    id: Uuid,
    slug: String,
    language: String,
    title: String,
    author: Option<String>,
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
