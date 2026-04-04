use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub session_secret: String,
    pub resend_api_key: String,
    pub from_email: String,
    pub backend_url: String,
    pub frontend_url: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_redirect_uri: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            session_secret: env::var("SESSION_SECRET").expect("SESSION_SECRET must be set"),
            resend_api_key: env::var("RESEND_API_KEY").expect("RESEND_API_KEY must be set"),
            from_email: env::var("FROM_EMAIL").expect("FROM_EMAIL must be set"),
            backend_url: env::var("BACKEND_URL").expect("BACKEND_URL must be set"),
            frontend_url: env::var("FRONTEND_URL").expect("FRONTEND_URL must be set"),
            github_client_id: env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set"),
            github_client_secret: env::var("GITHUB_CLIENT_SECRET")
                .expect("GITHUB_CLIENT_SECRET must be set"),
            github_redirect_uri: env::var("GITHUB_REDIRECT_URI")
                .expect("GITHUB_REDIRECT_URI must be set"),
        }
    }
}
