use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

// ── Status enum (matches db `feedback_status`) ─────────────

#[derive(Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[sqlx(type_name = "feedback_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum FeedbackStatus {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

// ── Request types ──────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFeedbackRequest {
    pub body: String,
    /// Optional. Full URL (path + query) the user was on when submitting.
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub viewport_w: Option<i32>,
    #[serde(default)]
    pub viewport_h: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFeedbackRequest {
    /// Omit either field to leave it unchanged.
    #[serde(default)]
    pub status: Option<FeedbackStatus>,
    #[serde(default)]
    pub admin_notes: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct FeedbackListQuery {
    /// Filter set: "active" (todo + in_progress, default), "all", or a
    /// specific status name ("todo", "in_progress", "done", "cancelled").
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub per_page: Option<u32>,
}

// ── Response types ─────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct FeedbackSubmitter {
    pub id: String,
    pub display_name: String,
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FeedbackHandler {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FeedbackResponse {
    pub id: String,
    /// `None` when the submitter's account has been deleted; the admin UI
    /// renders "User deleted" in that case.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitter: Option<FeedbackSubmitter>,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport_w: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport_h: Option<i32>,
    pub status: FeedbackStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handled_by: Option<FeedbackHandler>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FeedbackListResponse {
    pub feedback: Vec<FeedbackResponse>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}
