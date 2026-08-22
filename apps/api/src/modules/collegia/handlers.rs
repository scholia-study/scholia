use axum::Json;
use axum::extract::{Path, Query, State};

use crate::modules::collegia::db;
use crate::modules::collegia::models::{
    CollegiumDetailResponse, CreateCollegiumRequest, DecideJoinRequestRequest,
    DiscoverCollegiaQuery, DiscoverCollegiaResponse, InviteTokenResponse, JoinByTokenResponse,
    JoinRequestDecision, JoinRequestListResponse, MyCollegiaResponse, RemoveMemberResponse,
    ReviewVisibility, UpdateCollegiumRequest, UpdateMemberRequest,
};
use crate::system::auth::middleware::AuthUser;
use crate::system::auth::permissions::Permission;
use crate::system::error::AppError;
use crate::system::state::AppState;
use crate::system::validation::{
    MAX_COLLEGIUM_DESCRIPTION, MAX_COLLEGIUM_JOIN_REQUESTS_PER_DAY, MAX_COLLEGIUM_NAME,
    check_max_len,
};

fn validated_collegium_name(name: &str) -> Result<&str, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest(
            "Collegium name cannot be empty".into(),
        ));
    }
    check_max_len("Collegium name", name, MAX_COLLEGIUM_NAME)?;
    Ok(name)
}

/// Trims and length-checks; keeps an empty string (the clear-description
/// signal on PATCH — the create path maps it to None instead).
fn validated_collegium_description(description: &str) -> Result<&str, AppError> {
    let description = description.trim();
    check_max_len("Group description", description, MAX_COLLEGIUM_DESCRIPTION)?;
    Ok(description)
}

fn parse_uuid(value: &str, what: &str) -> Result<uuid::Uuid, AppError> {
    uuid::Uuid::parse_str(value).map_err(|_| AppError::BadRequest(format!("Invalid {what}")))
}

fn max_created(user: &AuthUser) -> i64 {
    if user.has_permission(Permission::CollegiaLimit5) {
        db::PAID_COLLEGIA_CREATED
    } else {
        db::FREE_COLLEGIA_CREATED
    }
}

/// Create a collegium; the creator becomes its first admin
#[utoipa::path(
    post,
    path = "/api/collegia",
    request_body = CreateCollegiumRequest,
    responses(
        (status = 200, description = "Collegium created", body = CollegiumDetailResponse),
        (status = 400, description = "Invalid name or creation limit reached"),
        (status = 401, description = "Not authenticated")
    ),
    tag = "collegia"
)]
pub async fn create_collegium(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateCollegiumRequest>,
) -> Result<Json<CollegiumDetailResponse>, AppError> {
    let name = validated_collegium_name(&body.name)?;
    let description = body
        .description
        .as_deref()
        .map(validated_collegium_description)
        .transpose()?
        .filter(|d| !d.is_empty());

    // Lifetime cap — soft-deleted collegia count, so leaving-and-recreating
    // never frees a slot.
    let created = db::count_created_by(&state.pool, user.id).await?;
    let max = max_created(&user);
    if created >= max {
        return Err(AppError::BadRequest(format!(
            "Collegium creation limit reached ({max} lifetime)"
        )));
    }

    let slug = db::create_collegium(
        &state.pool,
        db::CollegiumCreate {
            name,
            description,
            is_private: body.is_private.unwrap_or(false),
            review_visibility: body.review_visibility.unwrap_or(ReviewVisibility::Members),
            created_by: user.id,
        },
    )
    .await?;

    let collegium = db::get_collegium_detail(&state.pool, user.id, &slug).await?;
    Ok(Json(collegium))
}

/// List the collegiums the current user belongs to
#[utoipa::path(
    get,
    path = "/api/user/collegia",
    responses(
        (status = 200, description = "My collegia", body = MyCollegiaResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "collegia"
)]
pub async fn list_my_collegia(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<MyCollegiaResponse>, AppError> {
    let collegia = db::list_my_collegia(&state.pool, user.id).await?;
    let created_count = db::count_created_by(&state.pool, user.id).await?;
    Ok(Json(MyCollegiaResponse {
        collegia,
        created_count,
        max_created: max_created(&user),
    }))
}

/// Browse public collegia (Discover)
#[utoipa::path(
    get,
    path = "/api/collegia",
    params(DiscoverCollegiaQuery),
    responses(
        (status = 200, description = "Public collegia", body = DiscoverCollegiaResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "collegia"
)]
pub async fn discover_collegia(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<DiscoverCollegiaQuery>,
) -> Result<Json<DiscoverCollegiaResponse>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let q = params.q.as_deref().map(str::trim).filter(|q| !q.is_empty());
    let (collegia, total) = db::discover_collegia(&state.pool, user.id, q, page, per_page).await?;
    Ok(Json(DiscoverCollegiaResponse { collegia, total }))
}

/// Get a collegium's page (member list for members, steward fields for stewards)
#[utoipa::path(
    get,
    path = "/api/collegia/{slug}",
    params(("slug" = String, Path, description = "Collegium slug")),
    responses(
        (status = 200, description = "Collegium detail", body = CollegiumDetailResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Collegium not found")
    ),
    tag = "collegia"
)]
pub async fn get_collegium(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<CollegiumDetailResponse>, AppError> {
    let collegium = db::get_collegium_detail(&state.pool, user.id, &slug).await?;
    Ok(Json(collegium))
}

/// Update collegium metadata (name, description, privacy) — collegium stewards only
#[utoipa::path(
    patch,
    path = "/api/collegia/{slug}",
    params(("slug" = String, Path, description = "Collegium slug")),
    request_body = UpdateCollegiumRequest,
    responses(
        (status = 200, description = "Collegium updated", body = CollegiumDetailResponse),
        (status = 400, description = "Invalid name"),
        (status = 403, description = "Not a collegium steward"),
        (status = 404, description = "Collegium not found")
    ),
    tag = "collegia"
)]
pub async fn update_collegium(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<UpdateCollegiumRequest>,
) -> Result<Json<CollegiumDetailResponse>, AppError> {
    let name = body
        .name
        .as_deref()
        .map(validated_collegium_name)
        .transpose()?;
    let description = body
        .description
        .as_deref()
        .map(validated_collegium_description)
        .transpose()?;

    db::update_collegium(
        &state.pool,
        &slug,
        user.id,
        db::CollegiumUpdate {
            name,
            description,
            is_private: body.is_private,
            review_visibility: body.review_visibility,
        },
    )
    .await?;

    let collegium = db::get_collegium_detail(&state.pool, user.id, &slug).await?;
    Ok(Json(collegium))
}

/// Generate or rotate the collegium's invite link token — collegium stewards only
#[utoipa::path(
    post,
    path = "/api/collegia/{slug}/invite-token",
    params(("slug" = String, Path, description = "Collegium slug")),
    responses(
        (status = 200, description = "New invite token", body = InviteTokenResponse),
        (status = 403, description = "Not a collegium steward"),
        (status = 404, description = "Collegium not found")
    ),
    tag = "collegia"
)]
pub async fn rotate_invite_token(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<InviteTokenResponse>, AppError> {
    let token = db::rotate_invite_token(&state.pool, &slug, user.id).await?;
    Ok(Json(InviteTokenResponse {
        invite_token: Some(token),
    }))
}

/// Disable the collegium's invite link — collegium stewards only
#[utoipa::path(
    delete,
    path = "/api/collegia/{slug}/invite-token",
    params(("slug" = String, Path, description = "Collegium slug")),
    responses(
        (status = 200, description = "Invite link disabled", body = InviteTokenResponse),
        (status = 403, description = "Not a collegium steward"),
        (status = 404, description = "Collegium not found")
    ),
    tag = "collegia"
)]
pub async fn disable_invite_token(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<InviteTokenResponse>, AppError> {
    db::disable_invite_token(&state.pool, &slug, user.id).await?;
    Ok(Json(InviteTokenResponse { invite_token: None }))
}

/// Join a collegium via an invite link
#[utoipa::path(
    post,
    path = "/api/collegia/join/{token}",
    params(("token" = String, Path, description = "Invite token")),
    responses(
        (status = 200, description = "Joined (or already a member)", body = JoinByTokenResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Invalid or revoked invite link")
    ),
    tag = "collegia"
)]
pub async fn join_by_token(
    State(state): State<AppState>,
    user: AuthUser,
    Path(token): Path<String>,
) -> Result<Json<JoinByTokenResponse>, AppError> {
    let joined = db::join_by_token(&state.pool, user.id, &token).await?;
    Ok(Json(JoinByTokenResponse {
        slug: joined.slug,
        name: joined.name,
        already_member: joined.already_member,
    }))
}

/// Ask to join a public collegium
#[utoipa::path(
    post,
    path = "/api/collegia/{slug}/join-requests",
    params(("slug" = String, Path, description = "Collegium slug")),
    responses(
        (status = 200, description = "Request filed"),
        (status = 400, description = "Already a member or daily limit reached"),
        (status = 404, description = "Collegium not found"),
        (status = 409, description = "A request is already pending")
    ),
    tag = "collegia"
)]
pub async fn create_join_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<()>, AppError> {
    let recent = db::count_recent_join_requests(&state.pool, user.id).await?;
    if recent >= MAX_COLLEGIUM_JOIN_REQUESTS_PER_DAY {
        return Err(AppError::BadRequest(
            "Daily join-request limit reached; try again tomorrow".into(),
        ));
    }
    db::create_join_request(&state.pool, user.id, &slug).await?;
    Ok(Json(()))
}

/// List pending join requests — collegium stewards only
#[utoipa::path(
    get,
    path = "/api/collegia/{slug}/join-requests",
    params(("slug" = String, Path, description = "Collegium slug")),
    responses(
        (status = 200, description = "Pending requests", body = JoinRequestListResponse),
        (status = 403, description = "Not a collegium steward"),
        (status = 404, description = "Collegium not found")
    ),
    tag = "collegia"
)]
pub async fn list_join_requests(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<JoinRequestListResponse>, AppError> {
    let requests = db::list_join_requests(&state.pool, &slug, user.id).await?;
    Ok(Json(JoinRequestListResponse { requests }))
}

/// Approve or reject a join request — collegium stewards only
#[utoipa::path(
    patch,
    path = "/api/collegia/{slug}/join-requests/{id}",
    params(
        ("slug" = String, Path, description = "Collegium slug"),
        ("id" = String, Path, description = "Join request ID"),
    ),
    request_body = DecideJoinRequestRequest,
    responses(
        (status = 200, description = "Request decided"),
        (status = 400, description = "Invalid status"),
        (status = 403, description = "Not a collegium steward"),
        (status = 404, description = "Collegium or request not found"),
        (status = 409, description = "Request already decided")
    ),
    tag = "collegia"
)]
pub async fn decide_join_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path((slug, id)): Path<(String, String)>,
    Json(body): Json<DecideJoinRequestRequest>,
) -> Result<Json<()>, AppError> {
    let approve = body.status == JoinRequestDecision::Approved;
    let request_id = parse_uuid(&id, "join request ID")?;
    db::decide_join_request(&state.pool, &slug, user.id, request_id, approve).await?;
    Ok(Json(()))
}

/// Leave a collegium (self) or remove a member (collegium stewards)
#[utoipa::path(
    delete,
    path = "/api/collegia/{slug}/members/{user_id}",
    params(
        ("slug" = String, Path, description = "Collegium slug"),
        ("user_id" = String, Path, description = "Member's user ID"),
    ),
    responses(
        (status = 200, description = "Member removed", body = RemoveMemberResponse),
        (status = 400, description = "Last steward must promote a successor first"),
        (status = 403, description = "Not allowed to remove this member"),
        (status = 404, description = "Collegium or member not found")
    ),
    tag = "collegia"
)]
pub async fn remove_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path((slug, user_id)): Path<(String, String)>,
) -> Result<Json<RemoveMemberResponse>, AppError> {
    let target = parse_uuid(&user_id, "user ID")?;
    let collegium_deleted = db::remove_member(&state.pool, &slug, user.id, target).await?;
    Ok(Json(RemoveMemberResponse { collegium_deleted }))
}

/// Promote a member to admin, or step down as steward (self only)
#[utoipa::path(
    patch,
    path = "/api/collegia/{slug}/members/{user_id}",
    params(
        ("slug" = String, Path, description = "Collegium slug"),
        ("user_id" = String, Path, description = "Member's user ID"),
    ),
    request_body = UpdateMemberRequest,
    responses(
        (status = 200, description = "Role updated"),
        (status = 400, description = "Invalid role or last-steward violation"),
        (status = 403, description = "Not allowed to change this role"),
        (status = 404, description = "Collegium or member not found")
    ),
    tag = "collegia"
)]
pub async fn update_member_role(
    State(state): State<AppState>,
    user: AuthUser,
    Path((slug, user_id)): Path<(String, String)>,
    Json(body): Json<UpdateMemberRequest>,
) -> Result<Json<()>, AppError> {
    let target = parse_uuid(&user_id, "user ID")?;
    db::update_member_role(&state.pool, &slug, user.id, target, body.role).await?;
    Ok(Json(()))
}
