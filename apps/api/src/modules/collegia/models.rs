use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Who reviews submissions: everyone (`members`, the writing-circle
/// mode) or stewards only (`stewards`, the classroom mode). Stored as
/// the Postgres enum `collegium_review_visibility`; governs new
/// submissions only (each review request snapshots it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "collegium_review_visibility", rename_all = "lowercase")]
pub enum ReviewVisibility {
    Members,
    Stewards,
}

/// A member's standing in a collegium. Stored as the Postgres enum
/// `collegium_member_role`; stewards run the collegium (settings,
/// membership, invite links, closing rounds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "collegium_member_role", rename_all = "lowercase")]
pub enum CollegiumRole {
    Steward,
    Member,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum JoinRequestDecision {
    Approved,
    Rejected,
}

/// A collegium as shown on list surfaces (My collegia, Discover). `my_role`
/// and `my_pending_request` are relative to the requesting user.
#[derive(Debug, Serialize, ToSchema)]
pub struct CollegiumResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_private: bool,
    pub member_count: i64,
    /// Absent for non-members.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub my_role: Option<CollegiumRole>,
    pub my_pending_request: bool,
    pub review_visibility: ReviewVisibility,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MyCollegiaResponse {
    pub collegia: Vec<CollegiumResponse>,
    /// Lifetime collegia created (soft-deleted ones count).
    pub created_count: i64,
    /// Lifetime creation cap for this user's tier.
    pub max_created: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DiscoverCollegiaResponse {
    pub collegia: Vec<CollegiumResponse>,
    pub total: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CollegiumMemberResponse {
    pub user_id: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    pub role: CollegiumRole,
    pub joined_at: String,
}

/// Full collegium page payload. Member-only and admin-only fields are absent
/// for viewers below that tier.
#[derive(Debug, Serialize, ToSchema)]
pub struct CollegiumDetailResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_private: bool,
    pub member_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub my_role: Option<CollegiumRole>,
    pub my_pending_request: bool,
    pub review_visibility: ReviewVisibility,
    pub created_at: String,
    /// Members only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<CollegiumMemberResponse>>,
    /// Admins only; absent for stewards too when no invite link is active
    /// (`my_role` tells the frontend whether the field is authoritative).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invite_token: Option<String>,
    /// Admins only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_join_request_count: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCollegiumRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_private: Option<bool>,
    /// Defaults to `members`.
    #[serde(default)]
    pub review_visibility: Option<ReviewVisibility>,
}

/// Partial update; the slug never changes. An empty-string `description`
/// clears it.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCollegiumRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_private: Option<bool>,
    /// Changing it affects future submissions only — each request keeps
    /// the visibility it was submitted under.
    #[serde(default)]
    pub review_visibility: Option<ReviewVisibility>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InviteTokenResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invite_token: Option<String>,
}

/// Result of redeeming an invite link; the frontend navigates to the
/// collegium page at `slug`.
#[derive(Debug, Serialize, ToSchema)]
pub struct JoinByTokenResponse {
    pub slug: String,
    pub name: String,
    pub already_member: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JoinRequestResponse {
    pub id: String,
    pub user_id: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JoinRequestListResponse {
    pub requests: Vec<JoinRequestResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DecideJoinRequestRequest {
    pub status: JoinRequestDecision,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMemberRequest {
    /// `steward` promotes (stewards only); `member` steps down (self only).
    pub role: CollegiumRole,
}

/// Membership removal result; `collegium_deleted` is true when the last
/// member left and the collegium was soft-deleted.
#[derive(Debug, Serialize, ToSchema)]
pub struct RemoveMemberResponse {
    pub collegium_deleted: bool,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct DiscoverCollegiaQuery {
    /// Case-insensitive match against name and description.
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub per_page: Option<i32>,
}
