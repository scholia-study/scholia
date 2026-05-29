//! The feedback domain: user-submitted feedback and the admin queue for
//! triaging it. A flat, single-feature domain.

pub mod db;
pub mod handlers;
pub mod models;

use utoipa_axum::router::OpenApiRouter;

use crate::system::state::AppState;

/// Authenticated submission of feedback.
pub fn user_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(utoipa_axum::routes!(handlers::create_feedback))
}

/// Admin queue: list, view, and update feedback (gated by
/// `Permission::AdminPanel`).
pub fn admin_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(handlers::list_feedback))
        .routes(utoipa_axum::routes!(
            handlers::get_feedback,
            handlers::update_feedback
        ))
}
