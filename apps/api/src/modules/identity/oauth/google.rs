//! Google specifics. Google is an OIDC provider, so one call to the standard
//! userinfo endpoint carries the stable subject id, the email, and whether
//! Google has verified it — an unverified email must never reach the
//! account-linking step in `flow`.

use serde::Deserialize;

use super::provider::{Endpoints, OAuthIdentity};

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_API: &str = "https://openidconnect.googleapis.com/v1/userinfo";

pub fn endpoints() -> Endpoints {
    Endpoints {
        auth_url: AUTH_URL,
        token_url: TOKEN_URL,
        scopes: &["openid", "email", "profile"],
    }
}

#[derive(Deserialize)]
struct GoogleUser {
    sub: String,
    email: Option<String>,
    #[serde(default)]
    email_verified: bool,
    name: Option<String>,
    picture: Option<String>,
}

pub async fn fetch_identity(
    http: &reqwest::Client,
    access_token: &str,
) -> Result<OAuthIdentity, &'static str> {
    let user: GoogleUser = http
        .get(USERINFO_API)
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|_| "oauth_user_fetch_failed")?
        .json()
        .await
        .map_err(|_| "oauth_user_fetch_failed")?;

    let email = match user.email {
        Some(email) if user.email_verified => email,
        _ => return Err("no_verified_email"),
    };

    let display_name = user
        .name
        .unwrap_or_else(|| email.split('@').next().unwrap_or(&email).to_string());

    Ok(OAuthIdentity {
        provider_user_id: user.sub,
        email,
        display_name,
        avatar_url: user.picture,
    })
}
