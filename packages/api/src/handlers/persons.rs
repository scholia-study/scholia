use axum::Json;
use axum::extract::{Path, Query, State};

use crate::auth::middleware::AuthUser;
use crate::auth::permissions::Permission;
use crate::db;
use crate::error::AppError;
use crate::models::resource::{CreatePersonRequest, PersonResponse, SearchQuery, UpdatePersonRequest};
use crate::state::AppState;

/// Search persons by name
#[utoipa::path(
    get,
    path = "/api/persons",
    params(SearchQuery),
    responses(
        (status = 200, description = "List of matching persons", body = Vec<PersonResponse>),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "persons"
)]
pub async fn search_persons(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<PersonResponse>>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let results = db::persons::search_persons(&state.pool, &params.q).await?;
    Ok(Json(results))
}

/// Create a new person
#[utoipa::path(
    post,
    path = "/api/persons",
    request_body = CreatePersonRequest,
    responses(
        (status = 200, description = "Person created", body = PersonResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "persons"
)]
pub async fn create_person(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreatePersonRequest>,
) -> Result<Json<PersonResponse>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let person = db::persons::create_person(
        &state.pool,
        &body.name,
        body.sort_name.as_deref(),
    )
    .await?;

    Ok(Json(person))
}

/// Update an existing person
#[utoipa::path(
    put,
    path = "/api/persons/{id}",
    params(("id" = String, Path, description = "Person ID")),
    request_body = UpdatePersonRequest,
    responses(
        (status = 200, description = "Person updated", body = PersonResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Person not found")
    ),
    tag = "persons"
)]
pub async fn update_person(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<UpdatePersonRequest>,
) -> Result<Json<PersonResponse>, AppError> {
    user.require_permission(Permission::ResourcesManage)
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let person_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid person ID".into()))?;

    let person = db::persons::update_person(
        &state.pool,
        person_id,
        body.name.as_deref(),
        body.sort_name.as_deref(),
    )
    .await?;

    Ok(Json(person))
}
