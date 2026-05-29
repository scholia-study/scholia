use tower_governor::GovernorLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::system::state::AppState;

pub mod modules;
pub mod system;

#[derive(OpenApi)]
#[openapi(
    info(title = "Scholia API", version = "0.1.0"),
    // Orphan schemas: no documented endpoint references them
    // (`list_all_quotations` returns `UnifiedQuotationListResponse`), so
    // the router's automatic schema collection won't pick them up. They
    // were historically pinned in the manual `components(schemas(...))`
    // list and are emitted into the generated web client, so we keep
    // registering them explicitly to hold the OpenAPI surface
    // byte-identical across the modular refactor. Removing them is a
    // separate cleanup, intentionally out of scope here.
    components(schemas(
        modules::writing::QuotationWithContextResponse,
        modules::writing::QuotationWithContextListResponse,
    ))
)]
pub struct ApiDoc;

/// Assemble the full documented API surface into one `OpenApiRouter`.
pub fn api_router() -> OpenApiRouter<AppState> {
    let rate_limit_layer = GovernorLayer::new(system::rate_limit::auth_config());
    let auth_router = crate::modules::identity::rate_limited_router().layer(rate_limit_layer);
    let user_router = OpenApiRouter::new()
        .merge(crate::modules::identity::user_router())
        .merge(crate::modules::writing::user_router())
        .merge(crate::modules::feedback::user_router())
        .merge(crate::modules::billing::user_router());

    // Admin routes — gated per-handler via Permission::AdminPanel.
    // Note: the `articles/{slug}/labels` endpoints live here for URL
    // consistency with other admin routes, but they're gated by
    // Permission::ArticleLabelsManage, not AdminPanel — editors qualify.
    let admin_router = OpenApiRouter::new()
        .merge(crate::modules::feedback::admin_router())
        .merge(crate::modules::writing::admin_router());

    let public_router = OpenApiRouter::new()
        .merge(crate::modules::corpus::public_router())
        .merge(crate::modules::writing::public_router())
        .merge(crate::modules::identity::public_router());

    let editor_router = crate::modules::corpus::editor_router();

    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(auth_router)
        .merge(user_router)
        .merge(public_router)
        .merge(editor_router)
        .merge(admin_router)
}
