use serde::Serialize;
use utoipa::ToSchema;

use crate::models::article::ArticleResponse;

/// Public-facing user profile, returned by `GET /api/users/{handle}`.
/// Distinct from `ProfileResponse` (the self-edit shape under
/// `GET /auth/profile`) because it strips email and operational fields.
#[derive(Debug, Serialize, ToSchema)]
pub struct PublicProfileResponse {
    pub id: String,
    pub handle: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// Roles to render as chips: filtered server-side to public-facing
    /// roles only (excludes `admin` and the default `user`).
    pub public_roles: Vec<String>,
    /// Member-since timestamp (RFC3339).
    pub created_at: String,
    /// First page of the user's published articles, newest first.
    pub articles: Vec<ArticleResponse>,
    pub article_total: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserHandleResponse {
    pub handle: String,
}
