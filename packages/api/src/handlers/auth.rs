use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use time::{Duration, OffsetDateTime};
use tower_sessions::Session;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::middleware::{set_session_user, AuthUser};
use crate::auth::tokens;
use crate::email;
use crate::state::AppState;

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
    pub avatar_url: Option<String>,
    pub has_password: bool,
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
}

// ── Handlers ────────────────────────────────────────────────

/// Register a new user with email and password
#[utoipa::path(
    post,
    path = "/auth/register",
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
    if display_name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Display name is required").into_response();
    }
    if body.password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            "Password must be at least 8 characters",
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
    let user_id: Uuid = match sqlx::query_scalar(
        "INSERT INTO users (email, display_name, password_hash) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&email)
    .bind(&display_name)
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
    path = "/auth/login",
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

    let row = match sqlx::query(
        "SELECT id, email, display_name, password_hash, avatar_url, email_verified_at FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(r)) => r,
        _ => return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
    };

    let user_id: Uuid = row.get("id");
    let user_email: String = row.get("email");
    let user_display_name: String = row.get("display_name");
    let password_hash: Option<String> = row.get("password_hash");
    let avatar_url: Option<String> = row.get("avatar_url");
    let email_verified_at: Option<OffsetDateTime> = row.get("email_verified_at");

    // Verify password
    let hash_str = match &password_hash {
        Some(h) => h,
        None => return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
    };

    let parsed_hash = match PasswordHash::new(hash_str) {
        Ok(h) => h,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
    }

    // Check email verification
    if email_verified_at.is_none() {
        return (StatusCode::FORBIDDEN, "Email not verified").into_response();
    }

    // Create session
    if set_session_user(&session, user_id).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Json(AuthResponse {
        id: user_id.to_string(),
        email: user_email,
        display_name: user_display_name,
        avatar_url,
    })
    .into_response()
}

/// Log out (destroy session)
#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "Logged out", body = MessageResponse)
    ),
    tag = "auth"
)]
pub async fn logout(session: Session) -> Json<MessageResponse> {
    let _ = session.flush().await;
    Json(MessageResponse {
        message: "Logged out".to_string(),
    })
}

/// Get current authenticated user
#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "Current user", body = AuthResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "auth"
)]
pub async fn me(user: AuthUser) -> Json<AuthResponse> {
    Json(AuthResponse {
        id: user.id.to_string(),
        email: user.email,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
    })
}

/// Request a password reset email
#[utoipa::path(
    post,
    path = "/auth/forgot-password",
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
    path = "/auth/reset-password",
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
    if body.password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            "Password must be at least 8 characters",
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

    // Update password and verify email (two birds, one stone)
    let _ = sqlx::query(
        "UPDATE users SET password_hash = $1, email_verified_at = COALESCE(email_verified_at, now()), updated_at = now() WHERE id = $2",
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

    // Invalidate all sessions for this user (best effort — search session data for user ID)
    let _ = sqlx::query("DELETE FROM tower_sessions WHERE data::text LIKE $1")
        .bind(format!("%{}%", user_id))
        .execute(&state.pool)
        .await;

    Json(MessageResponse {
        message: "Password reset successfully. Please log in.".to_string(),
    })
    .into_response()
}

/// Verify email using a token
#[utoipa::path(
    get,
    path = "/auth/verify-email",
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
    let _ = sqlx::query("UPDATE users SET email_verified_at = now(), updated_at = now() WHERE id = $1")
        .bind(user_id)
        .execute(&state.pool)
        .await;

    // Mark token as used
    let _ = sqlx::query("UPDATE email_verification_tokens SET used_at = now() WHERE id = $1")
        .bind(token_id)
        .execute(&state.pool)
        .await;

    // Auto-login
    let _ = set_session_user(&session, user_id).await;

    Redirect::to(&format!(
        "{}/verify-email?success=true",
        state.config.frontend_url
    ))
    .into_response()
}

/// Get current user's profile with linked providers
#[utoipa::path(
    get,
    path = "/auth/profile",
    responses(
        (status = 200, description = "User profile", body = ProfileResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "auth"
)]
pub async fn get_profile(
    State(state): State<AppState>,
    user: AuthUser,
) -> Json<ProfileResponse> {
    let row = sqlx::query(
        "SELECT password_hash IS NOT NULL as has_password FROM users WHERE id = $1",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await
    .unwrap();

    let has_password: bool = row.get("has_password");

    let provider_rows = sqlx::query("SELECT provider, email FROM user_oauth_accounts WHERE user_id = $1")
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

    Json(ProfileResponse {
        id: user.id.to_string(),
        email: user.email,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        has_password,
        providers,
    })
}

/// Update display name
#[utoipa::path(
    put,
    path = "/auth/profile",
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

    let _ = sqlx::query("UPDATE users SET display_name = $1, updated_at = now() WHERE id = $2")
        .bind(&display_name)
        .bind(user.id)
        .execute(&state.pool)
        .await;

    Json(MessageResponse {
        message: "Profile updated.".to_string(),
    })
    .into_response()
}

/// Request a password change email (for authenticated users)
#[utoipa::path(
    post,
    path = "/auth/request-password-change",
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
