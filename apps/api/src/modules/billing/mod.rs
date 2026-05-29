//! The billing domain: Stripe Embedded Checkout and Customer Portal
//! sessions, plus the Stripe webhook. A flat, single-feature domain whose
//! queries are inline in `handlers`.

mod handlers;

use utoipa_axum::router::OpenApiRouter;

use crate::system::state::AppState;

/// Authenticated billing actions: start a checkout or open the portal.
pub fn user_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(handlers::create_checkout_session))
        .routes(utoipa_axum::routes!(handlers::create_portal_session))
}

/// The Stripe webhook lives on a plain axum router, outside the documented
/// surface: it deliberately bypasses the session and CORS layers and is
/// authenticated solely by the `Stripe-Signature` header. `main` merges
/// this after `split_for_parts`, before `with_state`.
pub fn webhook_routes() -> axum::Router<AppState> {
    axum::Router::new().route(
        "/api/webhooks/stripe",
        axum::routing::post(handlers::stripe_webhook),
    )
}
