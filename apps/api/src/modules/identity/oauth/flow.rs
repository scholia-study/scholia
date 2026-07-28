//! The provider-independent half of OAuth login: CSRF-protected redirect,
//! code exchange, then linking the returned identity to a Scholia account and
//! opening a session. Provider differences are confined to `provider`.

use axum::response::{IntoResponse, Redirect, Response};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointSet, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use serde::Deserialize;
use sqlx::PgPool;
use tower_sessions::Session;
use uuid::Uuid;

use crate::system::auth::handle::derive_handle;
use crate::system::auth::middleware::set_session_user;
use crate::system::auth::sort_name::derive_sort_name;
use crate::system::state::AppState;

use super::provider::{OAuthIdentity, Provider};

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

/// `code`/`state` are absent when the user cancels at the provider's consent
/// screen, which arrives as `?error=access_denied` instead.
#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

fn state_session_key(provider: Provider) -> String {
    format!("oauth_state_{}", provider.slug())
}

fn build_oauth_client(state: &AppState, provider: Provider) -> OAuthClient {
    let endpoints = provider.endpoints();
    let credentials = provider.credentials(&state.config);

    BasicClient::new(ClientId::new(credentials.client_id.to_string()))
        .set_client_secret(ClientSecret::new(credentials.client_secret.to_string()))
        .set_auth_uri(AuthUrl::new(endpoints.auth_url.to_string()).unwrap())
        .set_token_uri(TokenUrl::new(endpoints.token_url.to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(credentials.redirect_uri.to_string()).unwrap())
}

pub async fn start(state: AppState, session: Session, provider: Provider) -> Response {
    let client = build_oauth_client(&state, provider);

    let mut request = client.authorize_url(CsrfToken::new_random);
    for scope in provider.endpoints().scopes {
        request = request.add_scope(Scope::new((*scope).to_string()));
    }
    let (auth_url, csrf_state) = request.url();

    if session
        .insert(&state_session_key(provider), csrf_state.secret().clone())
        .await
        .is_err()
    {
        return axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Redirect::to(auth_url.as_str()).into_response()
}

pub async fn callback(
    state: AppState,
    session: Session,
    provider: Provider,
    query: CallbackQuery,
) -> Response {
    let error_redirect = |msg: &str| -> Response {
        Redirect::to(&format!(
            "{}/login?error={}",
            state.config.frontend_url, msg
        ))
        .into_response()
    };

    let key = state_session_key(provider);
    let stored_state: Option<String> = session.get(&key).await.ok().flatten();
    let _ = session.remove::<String>(&key).await;

    if query.error.is_some() {
        return error_redirect("oauth_cancelled");
    }

    let (Some(code), Some(returned_state)) = (query.code, query.state) else {
        return error_redirect("oauth_invalid_callback");
    };

    if stored_state.as_deref() != Some(returned_state.as_str()) {
        return error_redirect("oauth_state_mismatch");
    }

    let client = build_oauth_client(&state, provider);
    let http_client = reqwest::Client::new();

    let token = match client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(&http_client)
        .await
    {
        Ok(t) => t,
        Err(_) => return error_redirect("oauth_token_exchange_failed"),
    };

    let identity = match provider
        .fetch_identity(&http_client, token.access_token().secret())
        .await
    {
        Ok(identity) => identity,
        Err(msg) => return error_redirect(msg),
    };

    let user_id = match link_or_create_user(&state.pool, provider, &identity).await {
        Ok(id) => id,
        Err(msg) => return error_redirect(msg),
    };

    if set_session_user(&session, &state.pool, user_id)
        .await
        .is_err()
    {
        return error_redirect("session_failed");
    }

    Redirect::to(&state.config.frontend_url).into_response()
}

async fn link_or_create_user(
    pool: &PgPool,
    provider: Provider,
    identity: &OAuthIdentity,
) -> Result<Uuid, &'static str> {
    let avatar_url = identity.avatar_url.as_deref();

    let existing_oauth: Option<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM user_oauth_accounts WHERE provider = $1 AND provider_user_id = $2",
    )
    .bind(provider.slug())
    .bind(&identity.provider_user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    if let Some(uid) = existing_oauth {
        let _ = sqlx::query(
            "UPDATE users SET avatar_url = COALESCE($1, avatar_url), updated_at = now() WHERE id = $2",
        )
        .bind(avatar_url)
        .bind(uid)
        .execute(pool)
        .await;
        return Ok(uid);
    }

    let existing_user: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(&identity.email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

    let uid = if let Some(id) = existing_user {
        // Auto-link to the existing local account and let the provider's
        // verified email stand in for email verification.
        //
        // Security: if the account was NOT already verified, its password
        // was set by whoever registered the email — which we never
        // confirmed was this OAuth identity. An attacker can pre-register a
        // victim's email with an attacker-known password (the account sits
        // unverified, unusable) and wait for the victim to sign in with the
        // provider. To defeat that pre-hijack, we discard the untrusted
        // password on link (the real owner recovers it via forgot-password)
        // and stamp sessions_invalidated_at. Already-verified accounts keep
        // their password — the owner proved the email earlier.
        let _ = sqlx::query(
            "UPDATE users
             SET password_hash = CASE WHEN email_verified_at IS NULL THEN NULL ELSE password_hash END,
                 sessions_invalidated_at = CASE
                     WHEN email_verified_at IS NULL AND password_hash IS NOT NULL THEN now()
                     ELSE sessions_invalidated_at
                 END,
                 email_verified_at = COALESCE(email_verified_at, now()),
                 avatar_url = COALESCE($1, avatar_url),
                 updated_at = now()
             WHERE id = $2",
        )
        .bind(avatar_url)
        .bind(id)
        .execute(pool)
        .await;
        id
    } else {
        let sort_name = derive_sort_name(&identity.display_name);
        let derived = derive_handle(&identity.display_name);
        let handle = crate::modules::identity::accounts::db::claim_unique_handle(pool, &derived)
            .await
            .map_err(|_| "account_creation_failed")?;

        let id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO users (email, display_name, sort_name, handle, avatar_url, email_verified_at) VALUES ($1, $2, $3, $4, $5, now()) RETURNING id",
        )
        .bind(&identity.email)
        .bind(&identity.display_name)
        .bind(&sort_name)
        .bind(&handle)
        .bind(avatar_url)
        .fetch_one(pool)
        .await
        .map_err(|_| "account_creation_failed")?;

        let _ = sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE name = 'user'",
        )
        .bind(id)
        .execute(pool)
        .await;

        id
    };

    let _ = sqlx::query(
        "INSERT INTO user_oauth_accounts (user_id, provider, provider_user_id, email) VALUES ($1, $2, $3, $4)",
    )
    .bind(uid)
    .bind(provider.slug())
    .bind(&identity.provider_user_id)
    .bind(&identity.email)
    .execute(pool)
    .await;

    Ok(uid)
}
