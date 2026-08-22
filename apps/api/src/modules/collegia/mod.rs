//! Peer-review collegia: users form collegia and submit articles to a collegium
//! for feedback instead of to the editorial team (a review request's
//! audience is its `collegium_id`; NULL means editorial). Membership rules —
//! lifetime creation quota, the ≥1-steward invariant, soft-delete when the
//! last member leaves — live in `db::rules`. See
//! `db/migrations/0024_collegia.sql`.

pub mod db;
pub mod handlers;
pub mod models;

use utoipa_axum::router::OpenApiRouter;

use crate::system::state::AppState;

pub use db::{member_role, member_role_by_slug, review_visibility};
pub use models::{CollegiumRole, ReviewVisibility};

/// Authenticated endpoints: collegium CRUD, discovery, membership, join
/// requests, and invite links. Authorization (collegium steward / member) is
/// enforced in the db layer against live membership.
pub fn user_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(
            handlers::discover_collegia,
            handlers::create_collegium
        ))
        .routes(utoipa_axum::routes!(handlers::list_my_collegia))
        .routes(utoipa_axum::routes!(handlers::join_by_token))
        .routes(utoipa_axum::routes!(
            handlers::get_collegium,
            handlers::update_collegium
        ))
        .routes(utoipa_axum::routes!(
            handlers::rotate_invite_token,
            handlers::disable_invite_token
        ))
        .routes(utoipa_axum::routes!(
            handlers::list_join_requests,
            handlers::create_join_request
        ))
        .routes(utoipa_axum::routes!(handlers::decide_join_request))
        .routes(utoipa_axum::routes!(
            handlers::update_member_role,
            handlers::remove_member
        ))
}
