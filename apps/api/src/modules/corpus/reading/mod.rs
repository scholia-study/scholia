//! Reading the hosted texts: books, library index, table of contents,
//! nodes, and paginated page content. `facsimile` is a db-only helper
//! shared by the nodes and page readers.

// `books` is `pub` so the bibliography subdomain (a sibling within corpus)
// can resolve a book slug; corpus keeps `reading` private, so it stays
// invisible outside the domain.
pub mod books;
mod facsimile;
mod library;
mod nodes;
mod page;
mod toc;

use utoipa_axum::router::OpenApiRouter;

use crate::system::state::AppState;

// Re-exported for the corpus facade: the writing domain resolves a book
// slug to its id when creating quotations.
pub use books::db::get_book_id_by_slug;

/// All reading endpoints are public (unauthenticated).
pub fn public_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(books::handlers::list_books))
        .routes(utoipa_axum::routes!(books::handlers::get_book))
        .routes(utoipa_axum::routes!(books::handlers::get_book_about))
        .routes(utoipa_axum::routes!(library::handlers::get_library))
        .routes(utoipa_axum::routes!(toc::handlers::get_toc))
        .routes(utoipa_axum::routes!(nodes::handlers::get_node))
        .routes(utoipa_axum::routes!(page::handlers::get_node_page))
}
