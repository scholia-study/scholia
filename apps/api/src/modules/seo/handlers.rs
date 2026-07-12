use std::borrow::Cow;
use std::fmt::Write as _;

use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};

use super::db;
use crate::system::error::AppError;
use crate::system::state::AppState;

const URLSET_OPEN: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8"?>"#,
    "\n",
    r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#,
    "\n",
);

/// Static, always-present pages listed in the site sitemap.
const STATIC_PAGES: &[&str] = &[
    "/",
    "/articles",
    "/about",
    "/contribute",
    "/membership",
    "/licence",
    "/privacy",
    "/terms",
];

/// Sitemap index: the site sitemap plus one child sitemap per book.
pub async fn sitemap_index(State(state): State<AppState>) -> Result<Response, AppError> {
    let origin = origin(&state);
    let books = db::book_entries(&state.pool).await?;

    let mut xml = String::from(concat!(
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        "\n",
        r#"<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#,
        "\n",
    ));
    let _ = writeln!(
        xml,
        "  <sitemap><loc>{origin}/sitemaps/site.xml</loc></sitemap>"
    );
    for book in books {
        let _ = writeln!(
            xml,
            "  <sitemap><loc>{origin}/sitemaps/books/{}.xml</loc><lastmod>{}</lastmod></sitemap>",
            xml_escape(&book.slug),
            book.lastmod.date(),
        );
    }
    xml.push_str("</sitemapindex>\n");
    Ok(xml_response(xml))
}

/// Per-book sitemap: the TOC page plus every content-bearing node.
/// Routed as `/sitemaps/books/{slug}` where the path segment is
/// `<book-slug>.xml` (axum path params span whole segments).
pub async fn book_sitemap(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Response, AppError> {
    let Some(book_slug) = slug.strip_suffix(".xml") else {
        return Err(AppError::NotFound(format!("No such sitemap: {slug}")));
    };
    let Some(book_lastmod) = db::book_lastmod(&state.pool, book_slug).await? else {
        return Err(AppError::NotFound(format!("Book not found: {book_slug}")));
    };
    let nodes = db::content_node_entries(&state.pool, book_slug).await?;

    let origin = origin(&state);
    let book_slug = xml_escape(book_slug);
    let mut xml = String::from(URLSET_OPEN);
    let _ = writeln!(
        xml,
        "  <url><loc>{origin}/books/{book_slug}</loc><lastmod>{}</lastmod></url>",
        book_lastmod.date(),
    );
    for node in nodes {
        let _ = writeln!(
            xml,
            "  <url><loc>{origin}/books/{book_slug}/{}</loc><lastmod>{}</lastmod></url>",
            xml_escape(&node.slug),
            node.lastmod.date(),
        );
    }
    xml.push_str("</urlset>\n");
    Ok(xml_response(xml))
}

/// Everything that isn't a book: static pages, published articles,
/// author profiles.
pub async fn site_sitemap(State(state): State<AppState>) -> Result<Response, AppError> {
    let origin = origin(&state);
    let articles = db::published_article_entries(&state.pool).await?;
    let authors = db::author_entries(&state.pool).await?;

    let mut xml = String::from(URLSET_OPEN);
    for path in STATIC_PAGES {
        let _ = writeln!(xml, "  <url><loc>{origin}{path}</loc></url>");
    }
    for article in articles {
        let _ = writeln!(
            xml,
            "  <url><loc>{origin}/articles/{}</loc><lastmod>{}</lastmod></url>",
            xml_escape(&article.slug),
            article.lastmod.date(),
        );
    }
    for author in authors {
        let _ = writeln!(
            xml,
            "  <url><loc>{origin}/users/{}</loc><lastmod>{}</lastmod></url>",
            xml_escape(&author.slug),
            author.lastmod.date(),
        );
    }
    xml.push_str("</urlset>\n");
    Ok(xml_response(xml))
}

fn origin(state: &AppState) -> String {
    state.config.frontend_url.trim_end_matches('/').to_owned()
}

fn xml_response(body: String) -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/xml; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        body,
    )
        .into_response()
}

/// Belt-and-braces: slugs and handles are `[a-z0-9-]` today, but the
/// sitemap must never emit malformed XML if that loosens.
fn xml_escape(s: &str) -> Cow<'_, str> {
    if s.contains(['&', '<', '>', '"', '\'']) {
        Cow::Owned(
            s.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&apos;"),
        )
    } else {
        Cow::Borrowed(s)
    }
}
