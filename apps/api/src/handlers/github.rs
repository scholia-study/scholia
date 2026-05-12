use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointSet, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use serde::Deserialize;
use tower_sessions::Session;
use uuid::Uuid;

use crate::auth::handle::derive_handle;
use crate::auth::middleware::set_session_user;
use crate::auth::sort_name::derive_sort_name;
use crate::db;
use crate::state::AppState;

const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USER_API: &str = "https://api.github.com/user";
const GITHUB_EMAILS_API: &str = "https://api.github.com/user/emails";
const OAUTH_STATE_KEY: &str = "oauth_state";

type OAuthClient = oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<
        oauth2::EmptyExtraTokenFields,
        oauth2::basic::BasicTokenType,
    >,
    oauth2::StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    EndpointSet,
>;

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Deserialize)]
struct GitHubUser {
    id: u64,
    login: String,
    name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

fn build_oauth_client(state: &AppState) -> OAuthClient {
    BasicClient::new(ClientId::new(state.config.github_client_id.clone()))
        .set_client_secret(ClientSecret::new(state.config.github_client_secret.clone()))
        .set_auth_uri(AuthUrl::new(GITHUB_AUTH_URL.to_string()).unwrap())
        .set_token_uri(TokenUrl::new(GITHUB_TOKEN_URL.to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(state.config.github_redirect_uri.clone()).unwrap())
}

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
    let client = build_oauth_client(&state);

    let (auth_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    if session
        .insert(OAUTH_STATE_KEY, csrf_state.secret().clone())
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Redirect::to(auth_url.as_str()).into_response()
}

/// Handle GitHub OAuth callback
#[utoipa::path(
    get,
    path = "/api/auth/github/callback",
    params(
        ("code" = String, Query, description = "Authorization code"),
        ("state" = String, Query, description = "CSRF state"),
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
    let error_redirect = |msg: &str| -> Response {
        Redirect::to(&format!(
            "{}/login?error={}",
            state.config.frontend_url, msg
        ))
        .into_response()
    };

    // Verify CSRF state
    let stored_state: Option<String> = session.get(OAUTH_STATE_KEY).await.ok().flatten();
    let _ = session.remove::<String>(OAUTH_STATE_KEY).await;

    if stored_state.as_deref() != Some(&query.state) {
        return error_redirect("oauth_state_mismatch");
    }

    // Exchange code for token
    let client = build_oauth_client(&state);
    let http_client = reqwest::Client::new();

    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&http_client)
        .await;

    let token = match token_result {
        Ok(t) => t,
        Err(_) => return error_redirect("oauth_token_exchange_failed"),
    };

    let access_token = token.access_token().secret();

    // Fetch GitHub user info
    let github_user: GitHubUser = match http_client
        .get(GITHUB_USER_API)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", "Scholia")
        .send()
        .await
        .and_then(|r| r.error_for_status())
    {
        Ok(resp) => match resp.json().await {
            Ok(u) => u,
            Err(_) => return error_redirect("oauth_user_fetch_failed"),
        },
        Err(_) => return error_redirect("oauth_user_fetch_failed"),
    };

    // Fetch verified primary email
    let emails: Vec<GitHubEmail> = match http_client
        .get(GITHUB_EMAILS_API)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", "Scholia")
        .send()
        .await
        .and_then(|r| r.error_for_status())
    {
        Ok(resp) => resp.json().await.unwrap_or_default(),
        Err(_) => return error_redirect("oauth_email_fetch_failed"),
    };

    let primary_email = emails
        .iter()
        .find(|e| e.primary && e.verified)
        .or_else(|| emails.iter().find(|e| e.verified));

    let email = match primary_email {
        Some(e) => e.email.clone(),
        None => return error_redirect("no_verified_email"),
    };

    let provider_user_id = github_user.id.to_string();
    let display_name = github_user
        .name
        .unwrap_or_else(|| github_user.login.clone());

    // Check if OAuth account already linked
    let existing_oauth: Option<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM user_oauth_accounts WHERE provider = 'github' AND provider_user_id = $1",
    )
    .bind(&provider_user_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();

    let avatar_url = github_user.avatar_url.as_deref();

    let user_id = if let Some(uid) = existing_oauth {
        // Update avatar on each login
        let _ = sqlx::query(
            "UPDATE users SET avatar_url = COALESCE($1, avatar_url), updated_at = now() WHERE id = $2",
        )
        .bind(avatar_url)
        .bind(uid)
        .execute(&state.pool)
        .await;
        uid
    } else {
        // Check if a user with this email exists (auto-link)
        let existing_user: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
                .bind(&email)
                .fetch_optional(&state.pool)
                .await
                .ok()
                .flatten();

        let uid = if let Some(id) = existing_user {
            // Auto-link: also verify email and update avatar if not verified
            let _ = sqlx::query(
                "UPDATE users SET email_verified_at = COALESCE(email_verified_at, now()), avatar_url = COALESCE($1, avatar_url), updated_at = now() WHERE id = $2",
            )
            .bind(avatar_url)
            .bind(id)
            .execute(&state.pool)
            .await;
            id
        } else {
            // Create new user (GitHub-verified email)
            let sort_name = derive_sort_name(&display_name);
            let derived = derive_handle(&display_name);
            let handle = match db::users::claim_unique_handle(&state.pool, &derived).await {
                Ok(h) => h,
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };
            match sqlx::query_scalar::<_, Uuid>(
                "INSERT INTO users (email, display_name, sort_name, handle, avatar_url, email_verified_at) VALUES ($1, $2, $3, $4, $5, now()) RETURNING id",
            )
            .bind(&email)
            .bind(&display_name)
            .bind(&sort_name)
            .bind(&handle)
            .bind(avatar_url)
            .fetch_one(&state.pool)
            .await
            {
                Ok(id) => {
                    // Assign default "user" role
                    let _ = sqlx::query(
                        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE name = 'user'",
                    )
                    .bind(id)
                    .execute(&state.pool)
                    .await;
                    id
                }
                Err(_) => return error_redirect("account_creation_failed"),
            }
        };

        // Link OAuth account
        let _ = sqlx::query(
            "INSERT INTO user_oauth_accounts (user_id, provider, provider_user_id, email) VALUES ($1, 'github', $2, $3)",
        )
        .bind(uid)
        .bind(&provider_user_id)
        .bind(&email)
        .execute(&state.pool)
        .await;

        uid
    };

    // Create session
    if set_session_user(&session, &state.pool, user_id)
        .await
        .is_err()
    {
        return error_redirect("session_failed");
    }

    Redirect::to(&state.config.frontend_url).into_response()
}
