use axum::Json;
use axum::extract::{Path, Query, State};

use crate::modules::corpus::bibliography::models::{
    CreatePersonRequest, PersonResponse, SearchQuery, UpdatePersonRequest,
};
use crate::system::auth::middleware::AuthUser;
use crate::system::auth::permissions::Permission;
use crate::system::error::AppError;
use crate::system::state::AppState;
use crate::system::validation::{MAX_PERSON_NAME, MAX_PERSON_SORT_NAME, check_max_len};

fn validate_person_fields(name: Option<&str>, sort_name: Option<&str>) -> Result<(), AppError> {
    if let Some(n) = name {
        check_max_len("Name", n, MAX_PERSON_NAME)?;
    }
    if let Some(s) = sort_name {
        check_max_len("Sort name", s, MAX_PERSON_SORT_NAME)?;
    }
    Ok(())
}

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
    user.require_any_permission(&[Permission::ResourcesManage, Permission::SourcesCreate])
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    let results =
        crate::modules::corpus::bibliography::persons::db::search_persons(&state.pool, &params.q)
            .await?;
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
    user.require_any_permission(&[Permission::ResourcesManage, Permission::SourcesCreate])
        .map_err(|_| AppError::Forbidden("Insufficient permissions".into()))?;

    validate_person_fields(Some(&body.name), body.sort_name.as_deref())?;

    let person = crate::modules::corpus::bibliography::persons::db::create_person(
        &state.pool,
        &body.name,
        body.sort_name.as_deref(),
        user.id,
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
    let person_id =
        uuid::Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid person ID".into()))?;

    let current =
        crate::modules::corpus::bibliography::persons::db::get_person(&state.pool, person_id)
            .await?;
    let is_editor = user.has_permission(Permission::ResourcesManage);
    if current.protected && !is_editor {
        return Err(AppError::Forbidden("This person is protected".into()));
    }
    if !is_editor && current.created_by != user.id.to_string() {
        return Err(AppError::Forbidden(
            "You can only edit persons you created".into(),
        ));
    }

    validate_person_fields(body.name.as_deref(), body.sort_name.as_deref())?;

    let person = crate::modules::corpus::bibliography::persons::db::update_person(
        &state.pool,
        person_id,
        body.name.as_deref(),
        body.sort_name.as_deref(),
    )
    .await?;

    Ok(Json(person))
}
