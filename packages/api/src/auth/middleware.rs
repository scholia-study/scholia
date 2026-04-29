use std::collections::HashSet;

use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use tower_sessions::Session;
use uuid::Uuid;

use super::permissions::{Permission, resolve_permissions};
use crate::state::AppState;

const USER_ID_KEY: &str = "user_id";
const SESSION_CREATED_AT_KEY: &str = "session_created_at";

/// Store user ID and creation time in session, and record the mapping in user_sessions.
pub async fn set_session_user(
    session: &Session,
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), StatusCode> {
    session
        .insert(USER_ID_KEY, user_id.to_string())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    session
        .insert(
            SESSION_CREATED_AT_KEY,
            OffsetDateTime::now_utc().unix_timestamp(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save the session first so it gets an ID
    session
        .save()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(session_id) = session.id() {
        let _ = sqlx::query(
            "INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2) ON CONFLICT (session_id) DO NOTHING",
        )
        .bind(session_id.to_string())
        .bind(user_id)
        .execute(pool)
        .await;
    }

    Ok(())
}

/// Delete all sessions for a user from both user_sessions and tower_sessions.
pub async fn invalidate_user_sessions(pool: &PgPool, user_id: Uuid) {
    // Get all session IDs for this user
    let session_ids: Vec<String> =
        sqlx::query_scalar("SELECT session_id FROM user_sessions WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    if !session_ids.is_empty() {
        // Delete from tower_sessions store
        for id in &session_ids {
            let _ = sqlx::query("DELETE FROM tower_sessions.session WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await;
        }

        // Delete from our mapping table
        let _ = sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await;
    }
}

/// The authenticated user, extracted from the session.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub roles: Vec<String>,
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

        let val: Option<String> = session.get(USER_ID_KEY).await.ok().flatten();

        let user_id = val
            .and_then(|s| Uuid::parse_str(&s).ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let session_created_at: Option<i64> =
            session.get(SESSION_CREATED_AT_KEY).await.ok().flatten();

        let user = load_auth_user(&state.pool, user_id, session_created_at)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)?;

        Ok(user)
    }
}

async fn load_auth_user(
    pool: &PgPool,
    user_id: Uuid,
    session_created_at: Option<i64>,
) -> Option<AuthUser> {
    let row = sqlx::query(
        "SELECT id, email, display_name, avatar_url, email_verified_at, sessions_invalidated_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    let email_verified_at: Option<OffsetDateTime> = row.get("email_verified_at");
    if email_verified_at.is_none() {
        return None;
    }

    // Reject sessions created before the last password change
    let sessions_invalidated_at: Option<OffsetDateTime> = row.get("sessions_invalidated_at");
    if let (Some(changed), Some(created)) = (sessions_invalidated_at, session_created_at) {
        if created < changed.unix_timestamp() {
            return None;
        }
    }

    let role_names: Vec<String> = sqlx::query_scalar(
        "SELECT r.name FROM user_roles ur JOIN roles r ON r.id = ur.role_id WHERE ur.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .ok()?;

    let permissions = resolve_permissions(&role_names);

    Some(AuthUser {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        avatar_url: row.get("avatar_url"),
        roles: role_names,
        permissions,
    })
}
