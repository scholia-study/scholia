use axum::Json;
use axum::extract::{Path, Query, State};
use uuid::Uuid;

use super::MANAGEABLE_ROLES;
use super::db;
use super::models::{AdminUserListQuery, AdminUserListResponse, AdminUserRow, SetUserRolesRequest};
use crate::system::auth::middleware::{AuthUser, invalidate_user_sessions};
use crate::system::auth::permissions::Permission;
use crate::system::error::AppError;
use crate::system::state::AppState;

/// Reject non-managers as if the route doesn't exist, matching the feedback
/// admin queue: don't signal that `/api/admin/*` endpoints are real.
fn require_users_manage(user: &AuthUser) -> Result<(), AppError> {
    user.require_permission(Permission::UsersManage)
        .map_err(|_| AppError::NotFound("Not found".into()))
}

/// List users with their roles, paginated and optionally searched.
#[utoipa::path(
    get,
    path = "/api/admin/users",
    params(AdminUserListQuery),
    responses(
        (status = 200, description = "User list", body = AdminUserListResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Insufficient permissions")
    ),
    tag = "admin-users"
)]
pub async fn list_users(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<AdminUserListQuery>,
) -> Result<Json<AdminUserListResponse>, AppError> {
    require_users_manage(&user)?;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(25).clamp(1, 100);
    let search = params
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let list = db::list_users(&state.pool, search, page, per_page).await?;
    Ok(Json(list))
}

/// Get a single user with their roles.
#[utoipa::path(
    get,
    path = "/api/admin/users/{id}",
    params(("id" = String, Path, description = "User ID")),
    responses(
        (status = 200, description = "User detail", body = AdminUserRow),
        (status = 400, description = "Invalid user ID"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Not found / insufficient permissions")
    ),
    tag = "admin-users"
)]
pub async fn get_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<AdminUserRow>, AppError> {
    require_users_manage(&user)?;
    let id = Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid user ID".into()))?;
    let u = db::get_user(&state.pool, id).await?;
    Ok(Json(u))
}

/// Replace a user's manageable roles. Rejects paid/unknown roles, refuses
/// self-demotion out of admin, and (in the DB tx) refuses to remove the
/// last admin. On success, the target's sessions are invalidated so the
/// change takes effect on their next request.
#[utoipa::path(
    put,
    path = "/api/admin/users/{id}/roles",
    params(("id" = String, Path, description = "User ID")),
    request_body = SetUserRolesRequest,
    responses(
        (status = 200, description = "Roles updated", body = AdminUserRow),
        (status = 400, description = "Invalid input / non-manageable role / last admin"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Not found / insufficient permissions")
    ),
    tag = "admin-users"
)]
pub async fn set_user_roles(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<SetUserRolesRequest>,
) -> Result<Json<AdminUserRow>, AppError> {
    require_users_manage(&user)?;
    let id = Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid user ID".into()))?;

    for role in &body.roles {
        if !MANAGEABLE_ROLES.contains(&role.as_str()) {
            return Err(AppError::BadRequest(format!(
                "Role '{role}' cannot be assigned here."
            )));
        }
    }
    if id == user.id && !body.roles.iter().any(|r| r == "admin") {
        return Err(AppError::BadRequest(
            "You cannot remove your own admin role.".into(),
        ));
    }

    let updated = db::set_user_roles(&state.pool, id, &body.roles).await?;
    invalidate_user_sessions(&state.pool, id).await;
    Ok(Json(updated))
}
