use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::modules::writing::articles::models::EditorialLabelResponse;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateReviewRequestRequest {
    /// `feedback` — the author wants comments; the editor closes the
    /// request as `resolved`. `publication` — a hand-off: approving
    /// publishes the article (if still a draft) and applies the
    /// `imprimatur` label.
    pub intent: String,
    /// Optional note posted as the first message in the article's
    /// review channel.
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ArticleReviewRequestResponse {
    pub id: String,
    pub article_id: String,
    pub intent: String,
    pub status: String,
    pub submitted_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
}

/// Article metadata shown on the review page. Distinct from the public
/// article DTOs because the article may still be a draft here.
#[derive(Debug, Serialize, ToSchema)]
pub struct ReviewArticleMeta {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub status: String,
    pub author_user_id: String,
    pub author_display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_handle: Option<String>,
    pub labels: Vec<EditorialLabelResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleReviewDetailResponse {
    pub request: ArticleReviewRequestResponse,
    /// Editor currently assigned to this request, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<ReviewParticipant>,
    /// The article HTML frozen at submission, annotated with
    /// `data-block` / `data-s` markers that comments anchor into.
    pub snapshot_html: String,
    pub article: ReviewArticleMeta,
    /// Every review round for this article (withdrawn rounds only for
    /// the author), newest first — for round navigation on the page.
    pub requests: Vec<ArticleReviewRequestResponse>,
    /// True when the article body was edited after this request was
    /// submitted — the snapshot no longer matches the live draft.
    pub draft_changed: bool,
}

/// Who wrote a message or comment. Absent when the account was deleted.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ReviewParticipant {
    pub user_id: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleReviewCommentResponse {
    pub id: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<ReviewParticipant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentence_start: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentence_end: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quoted_text: Option<String>,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleReviewCommentListResponse {
    pub comments: Vec<ArticleReviewCommentResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateReviewCommentRequest {
    /// `data-block` index of the snapshot block this comment anchors to.
    pub block_index: i32,
    /// Sentence range within the block (`data-s` indices, inclusive).
    /// Omit both for a block-level comment (embeds, figures).
    #[serde(default)]
    pub sentence_start: Option<i32>,
    #[serde(default)]
    pub sentence_end: Option<i32>,
    /// The anchored text as the editor saw it, for the comment rail.
    #[serde(default)]
    pub quoted_text: Option<String>,
    pub body: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateReviewReplyRequest {
    pub body: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateReviewCommentRequest {
    pub resolved: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleReviewMessageResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<ReviewParticipant>,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleReviewMessageListResponse {
    pub messages: Vec<ArticleReviewMessageResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateReviewMessageRequest {
    pub body: String,
}

/// A cheap change-detection stamp for the review page's polling: the
/// client treats the whole object as an opaque token and invalidates
/// its queries when any field changes. Counts alone would miss
/// un-resolving a comment; timestamps alone would miss deletions.
#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleReviewActivityResponse {
    pub messages: i64,
    pub comments: i64,
    pub resolved_comments: i64,
    pub requests_updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleReviewQueueItem {
    pub id: String,
    pub article_id: String,
    pub article_title: String,
    pub article_slug: String,
    pub article_status: String,
    pub author_user_id: String,
    pub author_display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_handle: Option<String>,
    pub intent: String,
    pub status: String,
    pub submitted_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    /// Unresolved top-level comments on this request.
    pub open_comment_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<ReviewParticipant>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArticleReviewQueueResponse {
    pub items: Vec<ArticleReviewQueueItem>,
    pub total: i64,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ReviewQueueQuery {
    /// `pending` (default) | `approved` | `declined` | `resolved` | `all`.
    #[serde(default)]
    pub filter: Option<String>,
    /// Filter by assignee: a reviewer's user id, or `unassigned`.
    /// Absent = everyone.
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub per_page: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignReviewRequest {
    /// Editor to assign, or null/omitted to unassign.
    #[serde(default)]
    pub assignee_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReviewerListResponse {
    pub reviewers: Vec<ReviewParticipant>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReviewDecisionRequest {
    /// `approved` | `declined` (publication intent) or `resolved`
    /// (feedback intent).
    pub status: String,
    /// Optional message posted to the article's review channel alongside
    /// the decision.
    #[serde(default)]
    pub message: Option<String>,
}
