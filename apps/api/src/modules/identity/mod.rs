//! The identity domain: account lifecycle (registration, login, sessions,
//! email verification, password reset, profile), GitHub OAuth, and public
//! user profiles. The `accounts` feature owns the user persistence and DTOs
//! that `oauth` and `profiles` build on. Cross-cutting auth *plumbing*
//! (session middleware, the `AuthSession`/`AuthUser` extractors, the
//! `Permission` enum, token primitives) lives in `crate::system::auth`.

mod accounts;
mod oauth;
mod profiles;

use utoipa_axum::router::OpenApiRouter;

use crate::system::state::AppState;

// Cross-domain facade — the writing domain attaches public role badges to
// article authors.
pub use accounts::db::list_public_roles_for;

/// Unauthenticated, rate-limited entry points: account lifecycle and OAuth.
/// The rate-limit layer itself is applied by `crate::api_router`.
pub fn rate_limited_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(accounts::handlers::register))
        .routes(utoipa_axum::routes!(accounts::handlers::login))
        .routes(utoipa_axum::routes!(accounts::handlers::logout))
        .routes(utoipa_axum::routes!(accounts::handlers::me))
        .routes(utoipa_axum::routes!(accounts::handlers::forgot_password))
        .routes(utoipa_axum::routes!(accounts::handlers::reset_password))
        .routes(utoipa_axum::routes!(accounts::handlers::verify_email))
        .routes(utoipa_axum::routes!(oauth::handlers::github_login))
        .routes(utoipa_axum::routes!(oauth::handlers::github_callback))
}

/// Authenticated profile management.
pub fn user_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(
            accounts::handlers::get_profile,
            accounts::handlers::update_profile
        ))
        .routes(utoipa_axum::routes!(
            accounts::handlers::request_password_change
        ))
}

/// Public user profiles + by-id → handle redirect resolver.
pub fn public_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(profiles::handlers::get_public_profile))
        .routes(utoipa_axum::routes!(profiles::handlers::get_handle_by_id))
}
