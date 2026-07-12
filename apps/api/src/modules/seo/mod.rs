//! SEO surface: XML sitemaps generated from Postgres.
//!
//! Served on plain axum routes at the site root, outside the OpenAPI
//! surface (like the Stripe webhook): `main` merges `routes()` after
//! `split_for_parts`, before `with_state`. The nginx proxy routes
//! `/sitemap.xml` and `/sitemaps/*` here and caches responses in
//! `api_cache`; URLs are built from `FRONTEND_URL`.

mod db;
mod handlers;

use crate::system::state::AppState;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/sitemap.xml", axum::routing::get(handlers::sitemap_index))
        .route(
            "/sitemaps/site.xml",
            axum::routing::get(handlers::site_sitemap),
        )
        .route(
            "/sitemaps/books/{slug}",
            axum::routing::get(handlers::book_sitemap),
        )
}
