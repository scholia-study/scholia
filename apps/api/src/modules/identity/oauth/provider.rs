//! What distinguishes one OAuth provider from another: its endpoints, its
//! credentials, and how to read an identity out of its user API. Everything
//! else about the login flow is shared — see `flow`.

use crate::system::config::AppConfig;

use super::{github, google};

/// The provider-independent shape `flow` needs to link or create a user.
/// `email` is always one the provider has verified.
pub struct OAuthIdentity {
    pub provider_user_id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

pub struct Endpoints {
    pub auth_url: &'static str,
    pub token_url: &'static str,
    pub scopes: &'static [&'static str],
}

pub struct Credentials<'a> {
    pub client_id: &'a str,
    pub client_secret: &'a str,
    pub redirect_uri: &'a str,
}

#[derive(Clone, Copy)]
pub enum Provider {
    GitHub,
    Google,
}

impl Provider {
    /// Stored in `user_oauth_accounts.provider` and used to namespace the
    /// per-provider CSRF state in the session.
    pub fn slug(self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::Google => "google",
        }
    }

    pub fn endpoints(self) -> Endpoints {
        match self {
            Self::GitHub => github::endpoints(),
            Self::Google => google::endpoints(),
        }
    }

    pub fn credentials(self, config: &AppConfig) -> Credentials<'_> {
        match self {
            Self::GitHub => Credentials {
                client_id: &config.github_client_id,
                client_secret: &config.github_client_secret,
                redirect_uri: &config.github_redirect_uri,
            },
            Self::Google => Credentials {
                client_id: &config.google_client_id,
                client_secret: &config.google_client_secret,
                redirect_uri: &config.google_redirect_uri,
            },
        }
    }

    /// Errors are the `?error=` slug the login page will show.
    pub async fn fetch_identity(
        self,
        http: &reqwest::Client,
        access_token: &str,
    ) -> Result<OAuthIdentity, &'static str> {
        match self {
            Self::GitHub => github::fetch_identity(http, access_token).await,
            Self::Google => google::fetch_identity(http, access_token).await,
        }
    }
}
