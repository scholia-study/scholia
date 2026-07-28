//! GitHub specifics: endpoints and the two calls it takes to get a verified
//! email (the user API omits it unless the profile is public).

use serde::Deserialize;

use super::provider::{Endpoints, OAuthIdentity};

const AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const USER_API: &str = "https://api.github.com/user";
const EMAILS_API: &str = "https://api.github.com/user/emails";

pub fn endpoints() -> Endpoints {
    Endpoints {
        auth_url: AUTH_URL,
        token_url: TOKEN_URL,
        scopes: &["user:email"],
    }
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

pub async fn fetch_identity(
    http: &reqwest::Client,
    access_token: &str,
) -> Result<OAuthIdentity, &'static str> {
    let user: GitHubUser = http
        .get(USER_API)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", "Scholia")
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|_| "oauth_user_fetch_failed")?
        .json()
        .await
        .map_err(|_| "oauth_user_fetch_failed")?;

    let emails: Vec<GitHubEmail> = http
        .get(EMAILS_API)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", "Scholia")
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|_| "oauth_email_fetch_failed")?
        .json()
        .await
        .unwrap_or_default();

    let email = emails
        .iter()
        .find(|e| e.primary && e.verified)
        .or_else(|| emails.iter().find(|e| e.verified))
        .ok_or("no_verified_email")?;

    Ok(OAuthIdentity {
        provider_user_id: user.id.to_string(),
        email: email.email.clone(),
        display_name: user.name.unwrap_or(user.login),
        avatar_url: user.avatar_url,
    })
}
