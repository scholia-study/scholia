//! Admin user management: list users and grant/revoke roles, gated by
//! `Permission::UsersManage`. Only `MANAGEABLE_ROLES` are grantable here —
//! the paid tiers are owned by the Stripe webhook role-sync and stay
//! read-only, and the default `user` role is never toggled.

pub mod db;
pub mod handlers;
pub mod models;

use utoipa_axum::router::OpenApiRouter;

use crate::system::state::AppState;

/// Roles an admin may grant or revoke through this dashboard.
pub const MANAGEABLE_ROLES: &[&str] = &["admin", "editor", "scholiast", "honorary"];

pub fn admin_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(handlers::list_users))
        .routes(utoipa_axum::routes!(handlers::get_user))
        .routes(utoipa_axum::routes!(handlers::set_user_roles))
}
