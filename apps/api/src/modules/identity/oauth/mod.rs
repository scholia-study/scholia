//! OAuth login/callback endpoints, one pair per provider. The handlers are
//! thin: they name a `Provider` and hand off to the shared `flow`.

mod flow;
mod github;
mod google;
mod provider;

use axum::extract::{Query, State};
use axum::response::Response;
use tower_sessions::Session;

use crate::system::state::AppState;

use flow::CallbackQuery;
use provider::Provider;

/// Redirect to GitHub OAuth
#[utoipa::path(
    get,
    path = "/api/auth/github",
    responses(
        (status = 302, description = "Redirect to GitHub")
    ),
    tag = "auth"
)]
pub async fn github_login(State(state): State<AppState>, session: Session) -> Response {
    flow::start(state, session, Provider::GitHub).await
}

/// Handle GitHub OAuth callback
#[utoipa::path(
    get,
    path = "/api/auth/github/callback",
    params(
        ("code" = Option<String>, Query, description = "Authorization code"),
        ("state" = Option<String>, Query, description = "CSRF state"),
        ("error" = Option<String>, Query, description = "Provider-side error (e.g. user cancelled)"),
    ),
    responses(
        (status = 302, description = "Redirect to frontend")
    ),
    tag = "auth"
)]
pub async fn github_callback(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<CallbackQuery>,
) -> Response {
    flow::callback(state, session, Provider::GitHub, query).await
}

/// Redirect to Google OAuth
#[utoipa::path(
    get,
    path = "/api/auth/google",
    responses(
        (status = 302, description = "Redirect to Google")
    ),
    tag = "auth"
)]
pub async fn google_login(State(state): State<AppState>, session: Session) -> Response {
    flow::start(state, session, Provider::Google).await
}

/// Handle Google OAuth callback
#[utoipa::path(
    get,
    path = "/api/auth/google/callback",
    params(
        ("code" = Option<String>, Query, description = "Authorization code"),
        ("state" = Option<String>, Query, description = "CSRF state"),
        ("error" = Option<String>, Query, description = "Provider-side error (e.g. user cancelled)"),
    ),
    responses(
        (status = 302, description = "Redirect to frontend")
    ),
    tag = "auth"
)]
pub async fn google_callback(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<CallbackQuery>,
) -> Response {
    flow::callback(state, session, Provider::Google, query).await
}
