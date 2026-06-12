use std::sync::OnceLock;

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use time::{Duration, OffsetDateTime};
use tower_sessions::Session;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::system::auth::handle::{HANDLE_RENAME_COOLDOWN_DAYS, derive_handle, validate_handle};
use crate::system::auth::middleware::{AuthUser, invalidate_user_sessions, set_session_user};
use crate::system::auth::permissions::{Permission, resolve_permission_names};
use crate::system::auth::sort_name::derive_sort_name;
use crate::system::auth::tokens;
use crate::system::cache;
use crate::system::email;
use crate::system::error::AppError;
use crate::system::state::AppState;
use crate::system::validation::{
    MAX_DISPLAY_NAME, MAX_EMAIL, MAX_PASSWORD, MIN_PASSWORD, check_max_len,
};

// ── Request / response types ────────────────────────────────

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct ProfileResponse {
    pub id: String,
    pub email: String,
    pub display_name: String,
    /// Bibliography sort key, "Last, First" form. Auto-derived at signup.
    pub sort_name: Option<String>,
    /// Public URL identifier (`/users/<handle>`).
    pub handle: Option<String>,
    /// RFC3339 timestamp of the most recent handle change. Used by the
    /// frontend to compute remaining cooldown days.
    pub handle_changed_at: Option<String>,
    pub bio: Option<String>,
    pub title: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    pub avatar_url: Option<String>,
    pub has_password: bool,
    pub roles: Vec<String>,
    pub providers: Vec<LinkedProvider>,
}

#[derive(Serialize, ToSchema)]
pub struct LinkedProvider {
    pub provider: String,
    pub email: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub display_name: String,
    /// Optional. Whitespace-only or empty resets to the auto-derived form.
    #[serde(default)]
    pub sort_name: Option<String>,
    /// Optional. If present and different from current, attempts a rename
    /// — subject to charset, reservation, and 30-day cooldown rules.
    #[serde(default)]
    pub handle: Option<String>,
    /// Profile-page fields. Empty string clears.
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub website_url: Option<String>,
}

// ── Handlers ────────────────────────────────────────────────

/// Register a new user with email and password
#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User created", body = MessageResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Email already exists")
    ),
    tag = "auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Response {
    let email = body.email.trim().to_lowercase();
    let display_name = body.display_name.trim().to_string();

    if email.is_empty() || !email.contains('@') {
        return (StatusCode::BAD_REQUEST, "Invalid email").into_response();
    }
    if let Err(e) = check_max_len("Email", &email, MAX_EMAIL) {
        return e.into_response();
    }
    if display_name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Display name is required").into_response();
    }
    if let Err(e) = check_max_len("Display name", &display_name, MAX_DISPLAY_NAME) {
        return e.into_response();
    }
    if body.password.len() < MIN_PASSWORD {
        return (
            StatusCode::BAD_REQUEST,
            format!("Password must be at least {MIN_PASSWORD} characters"),
        )
            .into_response();
    }
    if body.password.len() > MAX_PASSWORD {
        return (
            StatusCode::BAD_REQUEST,
            format!("Password must be {MAX_PASSWORD} characters or fewer"),
        )
            .into_response();
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = match Argon2::default().hash_password(body.password.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // Insert user
    let sort_name = derive_sort_name(&display_name);
    let derived = derive_handle(&display_name);
    let handle =
        match crate::modules::identity::accounts::db::claim_unique_handle(&state.pool, &derived)
            .await
        {
            Ok(h) => h,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
    let user_id: Uuid = match sqlx::query_scalar(
        "INSERT INTO users (email, display_name, sort_name, handle, password_hash) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(&email)
    .bind(&display_name)
    .bind(&sort_name)
    .bind(&handle)
    .bind(&password_hash)
    .fetch_one(&state.pool)
    .await
    {
        Ok(id) => id,
        Err(e) if is_unique_violation(&e) => {
            return (StatusCode::CONFLICT, "Email already exists").into_response();
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // Assign default "user" role
    let _ = sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE name = 'user'",
    )
    .bind(user_id)
    .execute(&state.pool)
    .await;

    // Generate and store email verification token
    let (raw_token, token_hash) = tokens::generate_token();
    let expires_at = OffsetDateTime::now_utc() + Duration::hours(24);

    let _ = sqlx::query(
        "INSERT INTO email_verification_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(&state.pool)
    .await;

    // Send verification email (fire and forget)
    let config = state.config.clone();
    let email_addr = email.clone();
    tokio::spawn(async move {
        if let Err(e) = email::send_verification_email(&config, &email_addr, &raw_token).await {
            tracing::error!("Failed to send verification email: {e}");
        }
    });

    (
        StatusCode::CREATED,
        Json(MessageResponse {
            message: "Account created. Check your email to verify your account.".to_string(),
        }),
    )
        .into_response()
}

/// Log in with email and password
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Logged in", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 403, description = "Email not verified")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<LoginRequest>,
) -> Response {
    let email = body.email.trim().to_lowercase();

    if let Err(e) = check_max_len("Email", &email, MAX_EMAIL) {
        return e.into_response();
    }
    if body.password.len() > MAX_PASSWORD {
        return AppError::BadRequest(format!(
            "Password must be {MAX_PASSWORD} characters or fewer"
        ))
        .into_response();
    }

    let maybe_row = sqlx::query(
        "SELECT id, email, display_name, password_hash, avatar_url, email_verified_at FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    // Always run exactly one Argon2 verification — against the real hash when
    // the account exists, otherwise a dummy — so response time doesn't reveal
    // whether the email is registered. (forgot-password is already
    // enumeration-safe; this closes the login timing oracle.)
    let stored_hash: Option<String> = maybe_row
        .as_ref()
        .and_then(|r| r.get::<Option<String>, _>("password_hash"));
    let hash_to_verify = stored_hash
        .as_deref()
        .unwrap_or_else(|| dummy_password_hash());
    let password_ok = PasswordHash::new(hash_to_verify)
        .map(|parsed| {
            Argon2::default()
                .verify_password(body.password.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false);

    // Reject with the same generic response whether the account is missing,
    // is OAuth-only (no password hash), or the password simply didn't match.
    let (Some(row), true) = (maybe_row, password_ok && stored_hash.is_some()) else {
        return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
    };

    let user_id: Uuid = row.get("id");
    let user_email: String = row.get("email");
    let user_display_name: String = row.get("display_name");
    let avatar_url: Option<String> = row.get("avatar_url");
    let email_verified_at: Option<OffsetDateTime> = row.get("email_verified_at");

    // Check email verification
    if email_verified_at.is_none() {
        return (StatusCode::FORBIDDEN, "Email not verified").into_response();
    }

    // Create session
    if set_session_user(&session, &state.pool, user_id)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let roles: Vec<String> = sqlx::query_scalar(
        "SELECT r.name FROM user_roles ur JOIN roles r ON r.id = ur.role_id WHERE ur.user_id = $1 ORDER BY r.name",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let permissions = resolve_permission_names(&roles);

    Json(AuthResponse {
        id: user_id.to_string(),
        email: user_email,
        display_name: user_display_name,
        avatar_url,
        roles,
        permissions,
    })
    .into_response()
}

/// Log out (destroy session)
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    responses(
        (status = 200, description = "Logged out", body = MessageResponse)
    ),
    tag = "auth"
)]
pub async fn logout(State(state): State<AppState>, session: Session) -> Json<MessageResponse> {
    let session_id = session.id().map(|id| id.to_string());
    let _ = session.flush().await;

    // Clean up our mapping
    if let Some(id) = session_id {
        let _ = sqlx::query("DELETE FROM user_sessions WHERE session_id = $1")
            .bind(&id)
            .execute(&state.pool)
            .await;
    }

    Json(MessageResponse {
        message: "Logged out".to_string(),
    })
}

/// Get current authenticated user
#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "Current user", body = AuthResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "auth"
)]
pub async fn me(user: AuthUser) -> Json<AuthResponse> {
    let permissions = resolve_permission_names(&user.roles);
    Json(AuthResponse {
        id: user.id.to_string(),
        email: user.email,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        roles: user.roles,
        permissions,
    })
}

/// Request a password reset email
#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "If the email exists, a reset link was sent", body = MessageResponse)
    ),
    tag = "auth"
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(body): Json<ForgotPasswordRequest>,
) -> Json<MessageResponse> {
    let email = body.email.trim().to_lowercase();

    // Always return 200 to avoid leaking whether email exists
    let response = Json(MessageResponse {
        message: "If an account with that email exists, a reset link has been sent.".to_string(),
    });

    if check_max_len("Email", &email, MAX_EMAIL).is_err() {
        return response;
    }

    let row = match sqlx::query("SELECT id, email FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(r)) => r,
        _ => return response,
    };

    let user_id: Uuid = row.get("id");
    let user_email: String = row.get("email");

    let (raw_token, token_hash) = tokens::generate_token();
    let expires_at = OffsetDateTime::now_utc() + Duration::hours(1);

    // Invalidate any prior unused reset tokens so only the newest one works.
    let _ = sqlx::query(
        "UPDATE password_reset_tokens SET used_at = now() WHERE user_id = $1 AND used_at IS NULL",
    )
    .bind(user_id)
    .execute(&state.pool)
    .await;

    let _ = sqlx::query(
        "INSERT INTO password_reset_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(&state.pool)
    .await;

    let config = state.config.clone();
    tokio::spawn(async move {
        if let Err(e) = email::send_password_reset_email(&config, &user_email, &raw_token).await {
            tracing::error!("Failed to send password reset email: {e}");
        }
    });

    response
}

/// Reset password using a token
#[utoipa::path(
    post,
    path = "/api/auth/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset", body = MessageResponse),
        (status = 400, description = "Invalid or expired token")
    ),
    tag = "auth"
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(body): Json<ResetPasswordRequest>,
) -> Response {
    if body.password.len() < MIN_PASSWORD {
        return (
            StatusCode::BAD_REQUEST,
            format!("Password must be at least {MIN_PASSWORD} characters"),
        )
            .into_response();
    }
    if body.password.len() > MAX_PASSWORD {
        return (
            StatusCode::BAD_REQUEST,
            format!("Password must be {MAX_PASSWORD} characters or fewer"),
        )
            .into_response();
    }

    let token_hash = tokens::hash_token(&body.token);
    let now = OffsetDateTime::now_utc();

    // Find valid token
    let token_row = match sqlx::query(
        "SELECT id, user_id FROM password_reset_tokens WHERE token_hash = $1 AND expires_at > $2 AND used_at IS NULL",
    )
    .bind(&token_hash)
    .bind(now)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(t)) => t,
        _ => return (StatusCode::BAD_REQUEST, "Invalid or expired token").into_response(),
    };

    let token_id: Uuid = token_row.get("id");
    let user_id: Uuid = token_row.get("user_id");

    // Hash new password
    let salt = SaltString::generate(&mut OsRng);
    let new_hash = match Argon2::default().hash_password(body.password.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // Update password, verify email, and set password_changed_at to invalidate existing sessions
    let _ = sqlx::query(
        "UPDATE users SET password_hash = $1, email_verified_at = COALESCE(email_verified_at, now()), sessions_invalidated_at = now(), updated_at = now() WHERE id = $2",
    )
    .bind(&new_hash)
    .bind(user_id)
    .execute(&state.pool)
    .await;

    // Mark token as used
    let _ = sqlx::query("UPDATE password_reset_tokens SET used_at = now() WHERE id = $1")
        .bind(token_id)
        .execute(&state.pool)
        .await;

    // Purge all sessions for this user
    invalidate_user_sessions(&state.pool, user_id).await;

    Json(MessageResponse {
        message: "Password reset successfully. Please log in.".to_string(),
    })
    .into_response()
}

/// Verify email using a token
#[utoipa::path(
    get,
    path = "/api/auth/verify-email",
    params(("token" = String, Query, description = "Verification token")),
    responses(
        (status = 302, description = "Redirects to frontend"),
    ),
    tag = "auth"
)]
pub async fn verify_email(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<VerifyEmailQuery>,
) -> Response {
    let token_hash = tokens::hash_token(&query.token);
    let now = OffsetDateTime::now_utc();

    let token_row = match sqlx::query(
        "SELECT id, user_id FROM email_verification_tokens WHERE token_hash = $1 AND expires_at > $2 AND used_at IS NULL",
    )
    .bind(&token_hash)
    .bind(now)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(t)) => t,
        _ => {
            return Redirect::to(&format!(
                "{}/login?error=invalid_token",
                state.config.frontend_url
            ))
            .into_response();
        }
    };

    let token_id: Uuid = token_row.get("id");
    let user_id: Uuid = token_row.get("user_id");

    // Mark email as verified
    let _ =
        sqlx::query("UPDATE users SET email_verified_at = now(), updated_at = now() WHERE id = $1")
            .bind(user_id)
            .execute(&state.pool)
            .await;

    // Mark token as used
    let _ = sqlx::query("UPDATE email_verification_tokens SET used_at = now() WHERE id = $1")
        .bind(token_id)
        .execute(&state.pool)
        .await;

    // Auto-login
    let _ = set_session_user(&session, &state.pool, user_id).await;

    Redirect::to(&format!(
        "{}/verify-email?success=true",
        state.config.frontend_url
    ))
    .into_response()
}

/// Get current user's profile with linked providers
#[utoipa::path(
    get,
    path = "/api/auth/profile",
    responses(
        (status = 200, description = "User profile", body = ProfileResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "auth"
)]
pub async fn get_profile(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<ProfileResponse>, AppError> {
    let row = sqlx::query(
        "SELECT password_hash IS NOT NULL AS has_password,
                sort_name, handle, handle_changed_at,
                bio, title, location, website_url
         FROM users WHERE id = $1",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;

    let has_password: bool = row.get("has_password");
    let sort_name: Option<String> = row.get("sort_name");
    let handle: Option<String> = row.get("handle");
    let handle_changed_at_raw: Option<time::OffsetDateTime> = row.get("handle_changed_at");
    let handle_changed_at = handle_changed_at_raw.and_then(|t| {
        t.format(&time::format_description::well_known::Rfc3339)
            .ok()
    });
    let bio: Option<String> = row.get("bio");
    let title: Option<String> = row.get("title");
    let location: Option<String> = row.get("location");
    let website_url: Option<String> = row.get("website_url");

    let roles: Vec<String> = sqlx::query_scalar(
        "SELECT r.name FROM user_roles ur JOIN roles r ON r.id = ur.role_id WHERE ur.user_id = $1 ORDER BY r.name",
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let provider_rows =
        sqlx::query("SELECT provider, email FROM user_oauth_accounts WHERE user_id = $1")
            .bind(user.id)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

    let providers = provider_rows
        .iter()
        .map(|r| LinkedProvider {
            provider: r.get("provider"),
            email: r.get("email"),
        })
        .collect();

    Ok(Json(ProfileResponse {
        id: user.id.to_string(),
        email: user.email,
        display_name: user.display_name,
        sort_name,
        handle,
        handle_changed_at,
        bio,
        title,
        location,
        website_url,
        avatar_url: user.avatar_url,
        has_password,
        roles,
        providers,
    }))
}

/// Update display name
#[utoipa::path(
    put,
    path = "/api/auth/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = MessageResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated")
    ),
    tag = "auth"
)]
pub async fn update_profile(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<UpdateProfileRequest>,
) -> Response {
    let display_name = body.display_name.trim().to_string();

    if display_name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Display name is required").into_response();
    }
    if let Err(e) = check_max_len("Display name", &display_name, MAX_DISPLAY_NAME) {
        return e.into_response();
    }

    // Empty / whitespace-only sort_name resets to the auto-derived form so
    // users can fall back to the default without having to know what it is.
    let sort_name = match body.sort_name.as_deref().map(str::trim) {
        None | Some("") => derive_sort_name(&display_name),
        Some(s) => s.to_string(),
    };
    if let Err(e) = check_max_len("Sort name", &sort_name, MAX_DISPLAY_NAME) {
        return e.into_response();
    }

    // Profile-page fields. Empty string clears (-> NULL).
    let to_opt = |s: &Option<String>| -> Option<String> {
        s.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    let bio = to_opt(&body.bio);
    let title = to_opt(&body.title);
    let location = to_opt(&body.location);
    let website_url = to_opt(&body.website_url);

    if let Some(b) = &bio
        && let Err(e) = check_max_len("Bio", b, crate::system::validation::MAX_PROFILE_BIO)
    {
        return e.into_response();
    }

    if let Some(t) = &title
        && let Err(e) = check_max_len("Title", t, crate::system::validation::MAX_PROFILE_TITLE)
    {
        return e.into_response();
    }

    if let Some(l) = &location
        && let Err(e) = check_max_len(
            "Location",
            l,
            crate::system::validation::MAX_PROFILE_LOCATION,
        )
    {
        return e.into_response();
    }

    if let Some(w) = &website_url
        && let Err(e) = check_max_len(
            "Website URL",
            w,
            crate::system::validation::MAX_PROFILE_WEBSITE_URL,
        )
    {
        return e.into_response();
    }

    // Handle rename — only attempt if the request actually includes a
    // `handle` field that differs from the current value.
    let new_handle = body.handle.as_deref().map(str::trim);
    if let Some(requested) = new_handle {
        // Look up current handle + last-changed timestamp.
        let row = match sqlx::query("SELECT handle, handle_changed_at FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_one(&state.pool)
            .await
        {
            Ok(r) => r,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        let current_handle: Option<String> = row.get("handle");
        let last_changed: Option<time::OffsetDateTime> = row.get("handle_changed_at");

        if Some(requested) != current_handle.as_deref() {
            if let Err(e) = validate_handle(requested) {
                return e.into_response();
            }

            // Cooldown — admins bypass.
            let is_admin = user.permissions.contains(&Permission::AdminPanel);
            if !is_admin && let Some(last) = last_changed {
                let elapsed = time::OffsetDateTime::now_utc() - last;
                let cooldown = time::Duration::days(HANDLE_RENAME_COOLDOWN_DAYS);
                if elapsed < cooldown {
                    let remaining = cooldown - elapsed;
                    let days_left = remaining.whole_days() + 1;
                    return (
                            StatusCode::BAD_REQUEST,
                            format!(
                                "Handle can be changed once every {HANDLE_RENAME_COOLDOWN_DAYS} days. Try again in {days_left} day(s)."
                            ),
                        )
                            .into_response();
                }
            }

            // Recycle prevention.
            let taken = match crate::modules::identity::accounts::db::is_handle_taken_by_other(
                &state.pool,
                requested,
                Some(user.id),
            )
            .await
            {
                Ok(t) => t,
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };
            if taken {
                return (StatusCode::CONFLICT, "Handle is already taken").into_response();
            }

            // Stash old handle in released_handles so it stays reserved
            // for this user even after rename.
            if let Some(old) = current_handle.as_deref()
                && let Err(_) = crate::modules::identity::accounts::db::record_released_handle(
                    &state.pool,
                    user.id,
                    old,
                )
                .await
            {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }

            if sqlx::query(
                "UPDATE users SET handle = $1, handle_changed_at = now(), updated_at = now() WHERE id = $2",
            )
            .bind(requested)
            .bind(user.id)
            .execute(&state.pool)
            .await
            .is_err()
            {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }

    let _ = sqlx::query(
        "UPDATE users
         SET display_name = $1, sort_name = $2,
             bio = $3, title = $4, location = $5, website_url = $6,
             updated_at = now()
         WHERE id = $7",
    )
    .bind(&display_name)
    .bind(&sort_name)
    .bind(&bio)
    .bind(&title)
    .bind(&location)
    .bind(&website_url)
    .bind(user.id)
    .execute(&state.pool)
    .await;

    // Invalidate the user's public profile pages. The 90-day handle-rename
    // cooldown means stale entries under a previous handle are rare; if a
    // rename did just happen, those cached entries fall off via TTL.
    if let Ok(row) = sqlx::query("SELECT handle FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(&state.pool)
        .await
        && let Some(handle) = row.get::<Option<String>, _>("handle")
    {
        cache::invalidate(
            &state,
            [format!("/users/{handle}"), format!("/api/users/{handle}")],
        );
    }

    Json(MessageResponse {
        message: "Profile updated.".to_string(),
    })
    .into_response()
}

/// Request a password change email (for authenticated users)
#[utoipa::path(
    post,
    path = "/api/auth/request-password-change",
    responses(
        (status = 200, description = "Password change email sent", body = MessageResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "auth"
)]
pub async fn request_password_change(
    State(state): State<AppState>,
    user: AuthUser,
) -> Json<MessageResponse> {
    let (raw_token, token_hash) = tokens::generate_token();
    let expires_at = OffsetDateTime::now_utc() + Duration::hours(1);

    // Invalidate any prior unused reset tokens so only the newest one works.
    let _ = sqlx::query(
        "UPDATE password_reset_tokens SET used_at = now() WHERE user_id = $1 AND used_at IS NULL",
    )
    .bind(user.id)
    .execute(&state.pool)
    .await;

    let _ = sqlx::query(
        "INSERT INTO password_reset_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user.id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(&state.pool)
    .await;

    let config = state.config.clone();
    let email_addr = user.email.clone();
    tokio::spawn(async move {
        if let Err(e) = email::send_password_reset_email(&config, &email_addr, &raw_token).await {
            tracing::error!("Failed to send password change email: {e}");
        }
    });

    Json(MessageResponse {
        message: "Password change link sent to your email.".to_string(),
    })
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = e {
        return db_err.code().as_deref() == Some("23505");
    }
    false
}

/// A fixed, valid Argon2 PHC string used to equalize login timing when no
/// account (or no password) matches. Computed once on first use; the password
/// it encodes is irrelevant — it exists only so the verify cost is always paid.
fn dummy_password_hash() -> &'static str {
    static HASH: OnceLock<String> = OnceLock::new();
    HASH.get_or_init(|| {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(b"timing-equalization-placeholder", &salt)
            .expect("hash dummy password")
            .to_string()
    })
}
