use std::collections::HashSet;

use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use sqlx::{PgPool, Row};
use tower_sessions::Session;
use uuid::Uuid;

use super::permissions::{Permission, resolve_permissions};
use crate::state::AppState;

const USER_ID_KEY: &str = "user_id";

/// Store user ID in session.
pub async fn set_session_user(session: &Session, user_id: Uuid) -> Result<(), StatusCode> {
    session
        .insert(USER_ID_KEY, user_id.to_string())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// The authenticated user, extracted from the session.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub permissions: HashSet<Permission>,
}

impl AuthUser {
    pub fn has_permission(&self, perm: Permission) -> bool {
        self.permissions.contains(&perm)
    }

    pub fn require_permission(&self, perm: Permission) -> Result<(), StatusCode> {
        if self.has_permission(perm) {
            Ok(())
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let val: Option<String> = session
            .get(USER_ID_KEY)
            .await
            .ok()
            .flatten();

        let user_id = val
            .and_then(|s| Uuid::parse_str(&s).ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        load_auth_user(&state.pool, user_id)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

async fn load_auth_user(pool: &PgPool, user_id: Uuid) -> Option<AuthUser> {
    let row = sqlx::query(
        "SELECT id, email, display_name, avatar_url, email_verified_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    let email_verified_at: Option<time::OffsetDateTime> = row.get("email_verified_at");
    if email_verified_at.is_none() {
        return None;
    }

    let role_names: Vec<String> = sqlx::query_scalar(
        "SELECT r.name FROM user_roles ur JOIN roles r ON r.id = ur.role_id WHERE ur.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .ok()?;

    Some(AuthUser {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        avatar_url: row.get("avatar_url"),
        permissions: resolve_permissions(&role_names),
    })
}
