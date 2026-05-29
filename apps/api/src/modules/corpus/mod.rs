//! The corpus domain: the hosted primary-source texts (`reading`) and
//! their bibliographic apparatus (`bibliography`), bridged by `core`
//! (effective-source resolution). `reading` and `bibliography` are mutually
//! coupled — a text node carries a `source_id` — so they live in one domain
//! with the cycle internalised. The subdomains are private; other domains
//! reach corpus only through the re-exports below.

mod bibliography;
mod core;
mod reading;

use utoipa_axum::router::OpenApiRouter;

use crate::system::state::AppState;

// Cross-domain facade — the only corpus internals visible outside the
// domain. The writing domain uses both when creating quotations.
pub use core::db::resolve_effective_source;
pub use reading::get_book_id_by_slug;

/// Public (unauthenticated) corpus endpoints: reading + the one public
/// bibliography endpoint.
pub fn public_router() -> OpenApiRouter<AppState> {
    reading::public_router().merge(bibliography::public_router())
}

/// Editor endpoints — entirely bibliographic CRUD.
pub fn editor_router() -> OpenApiRouter<AppState> {
    bibliography::editor_router()
}
