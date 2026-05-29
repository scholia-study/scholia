//! Bibliographic apparatus for the hosted texts: resources (external
//! references attached to anchors), sources (works/editions), and persons
//! (authors/translators). The three features share a single DTO module
//! (`models`) because their response shapes are mutually referential.

mod persons;
mod resources;
// `sources` is `pub` so the reading subdomain (books) can resolve a
// source for a quotation anchor; corpus keeps `bibliography` private, so
// it stays invisible outside the domain.
pub mod sources;

pub mod models;

use utoipa_axum::router::OpenApiRouter;

use crate::system::state::AppState;

/// The single public bibliography endpoint: resources for an anchor.
pub fn public_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(utoipa_axum::routes!(resources::handlers::list_resources))
}

/// Editor CRUD over resources, sources, and persons. Auth is enforced
/// inside each handler via `Permission::*`.
pub fn editor_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(resources::handlers::create_resource))
        .routes(utoipa_axum::routes!(
            resources::handlers::update_resource,
            resources::handlers::delete_resource
        ))
        .routes(utoipa_axum::routes!(
            sources::handlers::search_sources,
            sources::handlers::create_source
        ))
        .routes(utoipa_axum::routes!(sources::handlers::browse_sources))
        .routes(utoipa_axum::routes!(
            sources::handlers::get_source,
            sources::handlers::update_source,
            sources::handlers::delete_source
        ))
        .routes(utoipa_axum::routes!(sources::handlers::add_source_person))
        .routes(utoipa_axum::routes!(
            sources::handlers::remove_source_person
        ))
        .routes(utoipa_axum::routes!(
            sources::handlers::check_source_references
        ))
        .routes(utoipa_axum::routes!(
            persons::handlers::search_persons,
            persons::handlers::create_person
        ))
        .routes(utoipa_axum::routes!(persons::handlers::update_person))
}
