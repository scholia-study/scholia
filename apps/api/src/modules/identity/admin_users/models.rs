use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminUserRow {
    pub id: String,
    pub email: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    /// All roles the user holds, including the default `user` role and any
    /// paid (Stripe-managed) tiers. The UI renders paid roles read-only.
    pub roles: Vec<String>,
    pub email_verified: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminUserListResponse {
    pub users: Vec<AdminUserRow>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct AdminUserListQuery {
    /// Case-insensitive substring match on email, display name, or handle.
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetUserRolesRequest {
    /// The full desired set of *manageable* roles for the user (see
    /// `MANAGEABLE_ROLES`). Paid tiers and the default `user` role are
    /// left untouched; passing a non-manageable role is rejected.
    pub roles: Vec<String>,
}
