use std::collections::HashSet;

use std::convert::Infallible;

use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::StatusCode;
use axum::http::request::Parts;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use tower_sessions::Session;
use uuid::Uuid;

use super::permissions::{Permission, resolve_permissions};
use crate::system::state::AppState;

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

    // Rotate the session ID on login/elevation to defeat session fixation:
    // any pre-auth session (e.g. one carrying OAuth CSRF state) is discarded
    // and the authenticated session gets a fresh ID.
    session
        .cycle_id()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save so the session gets its (new) ID before we record the mapping.
    session
        .save()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(session_id) = session.id()
        && let Err(e) = sqlx::query(
            "INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2) ON CONFLICT (session_id) DO NOTHING",
        )
        .bind(session_id.to_string())
        .bind(user_id)
        .execute(pool)
        .await
    {
        // The session still works, but it won't be reachable by
        // invalidate_user_sessions; sessions_invalidated_at is the backstop.
        tracing::error!("failed to record user_sessions mapping: {e}");
    }

    Ok(())
}

/// Delete all sessions for a user from both user_sessions and tower_sessions.
pub async fn invalidate_user_sessions(pool: &PgPool, user_id: Uuid) {
    let session_ids: Vec<String> =
        match sqlx::query_scalar("SELECT session_id FROM user_sessions WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(pool)
            .await
        {
            Ok(ids) => ids,
            Err(e) => {
                tracing::error!("failed to list sessions for user {user_id} to invalidate: {e}");
                Vec::new()
            }
        };

    if !session_ids.is_empty() {
        for id in &session_ids {
            if let Err(e) = sqlx::query("DELETE FROM tower_sessions.session WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await
            {
                tracing::error!("failed to delete session {id}: {e}");
            }
        }

        if let Err(e) = sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
        {
            tracing::error!("failed to clear user_sessions for user {user_id}: {e}");
        }
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

    pub fn has_any_permission(&self, perms: &[Permission]) -> bool {
        perms.iter().any(|p| self.permissions.contains(p))
    }

    pub fn require_any_permission(&self, perms: &[Permission]) -> Result<(), StatusCode> {
        if self.has_any_permission(perms) {
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

/// Optional variant for endpoints that are public but adjust their response
/// for authenticated callers (e.g. exposing editor-only fields). Any reason
/// the required extractor would reject — no session, unverified email,
/// invalidated session — yields `None` rather than a 401.
impl OptionalFromRequestParts<AppState> for AuthUser {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        Ok(
            <AuthUser as FromRequestParts<AppState>>::from_request_parts(parts, state)
                .await
                .ok(),
        )
    }
}

async fn load_auth_user(
    pool: &PgPool,
    user_id: Uuid,
    session_created_at: Option<i64>,
) -> Option<AuthUser> {
    let row = match sqlx::query(
        "SELECT id, email, display_name, avatar_url, email_verified_at, sessions_invalidated_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return None,
        Err(e) => {
            // A DB failure here must not silently look like "logged out";
            // log it so an outage is visible rather than mass-401s.
            tracing::error!("auth: failed to load user {user_id}: {e}");
            return None;
        }
    };

    let email_verified_at: Option<OffsetDateTime> = row.get("email_verified_at");
    email_verified_at?;

    // Reject any session that predates the last invalidation (e.g. a password
    // reset). Fail closed when the session carries no creation time — an
    // unknown age can't be proven newer than the invalidation.
    let sessions_invalidated_at: Option<OffsetDateTime> = row.get("sessions_invalidated_at");
    if let Some(changed) = sessions_invalidated_at
        && session_created_at.is_none_or(|created| created < changed.unix_timestamp())
    {
        return None;
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
