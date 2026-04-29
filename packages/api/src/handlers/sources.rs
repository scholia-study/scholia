use axum::Json;
use axum::extract::{Path, Query, State};

use crate::auth::middleware::AuthUser;
use crate::auth::permissions::Permission;
use crate::db;
use crate::error::AppError;
use crate::models::resource::{
    CreateSourceRequest, LinkSourcePersonRequest, ReferenceCheckResponse, SearchQuery,
    SourceBrowseQuery, SourceBrowseResponse, SourceResponse, SourceSearchResponse,
    UpdateSourceRequest,
};
use crate::state::AppState;
use crate::validation::{
    MAX_PUBLICATION_YEAR, MAX_SOURCE_DOI, MAX_SOURCE_EDITION, MAX_SOURCE_ISBN_LEN,
    MAX_SOURCE_ISBNS, MAX_SOURCE_JOURNAL_NAME, MAX_SOURCE_PAGE, MAX_SOURCE_PUBLISHER,
    MAX_SOURCE_TITLE, MAX_SOURCE_TITLE_DISPLAY, MAX_SOURCE_URL, MAX_SOURCE_VOLUME,
    MIN_PUBLICATION_YEAR, MIN_SOURCE_PAGE, check_count, check_int_range, check_max_len,
};

async fn guard_source_edit(
    pool: &sqlx::PgPool,
    user: &AuthUser,
    source_id: uuid::Uuid,
) -> Result<(), AppError> {
    let current = db::sources::get_source(pool, source_id).await?;
    let is_editor = user.has_permission(Permission::ResourcesManage);
    if current.protected && !is_editor {
        return Err(AppError::Forbidden("This source is protected".into()));
    }
    if !is_editor && current.created_by != user.id.to_string() {
        return Err(AppError::Forbidden(
            "You can only edit sources you created".into(),
        ));
    }
    Ok(())
}

fn validate_source_fields(
    title: Option<&str>,
    title_display: Option<&str>,
    publisher: Option<&str>,
    journal_name: Option<&str>,
    doi: Option<&str>,
    edition: Option<&str>,
    volume: Option<&str>,
    url: Option<&str>,
    isbn: Option<&[String]>,
    publication_year: Option<i16>,
    page_start: Option<i32>,
    page_end: Option<i32>,
) -> Result<(), AppError> {
    if let Some(t) = title {
        check_max_len("Title", t, MAX_SOURCE_TITLE)?;
    }
    if let Some(t) = title_display {
        check_max_len("Display title", t, MAX_SOURCE_TITLE_DISPLAY)?;
    }
    if let Some(p) = publisher {
        check_max_len("Publisher", p, MAX_SOURCE_PUBLISHER)?;
    }
    if let Some(j) = journal_name {
        check_max_len("Journal name", j, MAX_SOURCE_JOURNAL_NAME)?;
    }
    if let Some(d) = doi {
        check_max_len("DOI", d, MAX_SOURCE_DOI)?;
    }
    if let Some(e) = edition {
        check_max_len("Edition", e, MAX_SOURCE_EDITION)?;
    }
    if let Some(v) = volume {
        check_max_len("Volume", v, MAX_SOURCE_VOLUME)?;
    }
    if let Some(u) = url {
        check_max_len("URL", u, MAX_SOURCE_URL)?;
    }
    if let Some(list) = isbn {
        check_count("ISBNs", list, MAX_SOURCE_ISBNS)?;
        for v in list {
            check_max_len("ISBN", v, MAX_SOURCE_ISBN_LEN)?;
        }
    }
    if let Some(y) = publication_year {
        check_int_range(
            "Publication year",
            y,
            MIN_PUBLICATION_YEAR,
            MAX_PUBLICATION_YEAR,
        )?;
    }
    if let Some(p) = page_start {
        check_int_range("Page start", p, MIN_SOURCE_PAGE, MAX_SOURCE_PAGE)?;
    }
    if let Some(p) = page_end {
        check_int_range("Page end", p, MIN_SOURCE_PAGE, MAX_SOURCE_PAGE)?;
    }
    Ok(())
}

/// Search sources by title
#[utoipa::path(
    get,
    path = "/api/sources",
    params(SearchQuery),
    responses(
        (status = 200, description = "List of matching sources", body = Vec<SourceSearchResponse>),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "sources"
)]
pub async fn search_sources(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<SourceSearchResponse>>, AppError> {
    if !user.has_permission(Permission::ResourcesManage)
        && !user.has_permission(Permission::SourcesCreate)
    {
        return Err(AppError::Forbidden("Insufficient permissions".into()));
    }

    let results = db::sources::search_sources(&state.pool, &params.q).await?;
    Ok(Json(results))
}

/// Browse sources (paginated, with filters)
#[utoipa::path(
    get,
    path = "/api/sources/browse",
    params(SourceBrowseQuery),
    responses(
        (status = 200, description = "Paginated sources", body = SourceBrowseResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "sources"
)]
pub async fn browse_sources(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<SourceBrowseQuery>,
) -> Result<Json<SourceBrowseResponse>, AppError> {
    if !user.has_permission(Permission::ResourcesManage)
        && !user.has_permission(Permission::SourcesCreate)
    {
        return Err(AppError::Forbidden("Insufficient permissions".into()));
    }

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let created_by = params.created_by_me.unwrap_or(false).then_some(user.id);

    // Protected filter is editor-only; silently ignore for non-editors.
    let protected = if user.has_permission(Permission::ResourcesManage) {
        params.protected
    } else {
        None
    };

    let q_trimmed = params.q.as_deref().map(str::trim).filter(|s| !s.is_empty());

    let (sources, total) = db::sources::browse_sources(
        &state.pool,
        q_trimmed,
        params.source_type.as_deref(),
        created_by,
        protected,
        page,
        per_page,
    )
    .await?;

    Ok(Json(SourceBrowseResponse { sources, total }))
}

/// Get a source by ID
#[utoipa::path(
    get,
    path = "/api/sources/{id}",
    params(("id" = String, Path, description = "Source ID")),
    responses(
        (status = 200, description = "Source details", body = SourceResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Source not found")
    ),
    tag = "sources"
)]
pub async fn get_source(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<SourceResponse>, AppError> {
    if !user.has_permission(Permission::ResourcesManage)
        && !user.has_permission(Permission::SourcesCreate)
    {
        return Err(AppError::Forbidden("Insufficient permissions".into()));
    }

    let source_id =
        uuid::Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid source ID".into()))?;

    let source = db::sources::get_source(&state.pool, source_id).await?;
    Ok(Json(source))
}

/// Create a new source
#[utoipa::path(
    post,
    path = "/api/sources",
    request_body = CreateSourceRequest,
    responses(
        (status = 200, description = "Source created", body = SourceResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "sources"
)]
pub async fn create_source(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateSourceRequest>,
) -> Result<Json<SourceResponse>, AppError> {
    if !user.has_permission(Permission::ResourcesManage)
        && !user.has_permission(Permission::SourcesCreate)
    {
        return Err(AppError::Forbidden("Insufficient permissions".into()));
    }

    let parent_source_id = body
        .parent_source_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid parent_source_id".into()))?;

    let translation_of_id = body
        .translation_of_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid translation_of_id".into()))?;

    validate_source_fields(
        Some(&body.title),
        body.title_display.as_deref(),
        body.publisher.as_deref(),
        body.journal_name.as_deref(),
        body.doi.as_deref(),
        body.edition.as_deref(),
        body.volume.as_deref(),
        body.url.as_deref(),
        body.isbn.as_deref(),
        body.publication_year,
        body.page_start,
        body.page_end,
    )?;

    let source = db::sources::create_source(
        &state.pool,
        &body.source_type,
        &body.title,
        body.title_display.as_deref(),
        body.publication_year,
        body.publisher.as_deref(),
        body.isbn.as_deref(),
        body.doi.as_deref(),
        body.edition.as_deref(),
        body.volume.as_deref(),
        body.journal_name.as_deref(),
        body.url.as_deref(),
        body.page_start,
        body.page_end,
        parent_source_id,
        translation_of_id,
        user.id,
    )
    .await?;

    Ok(Json(source))
}

/// Update an existing source
#[utoipa::path(
    put,
    path = "/api/sources/{id}",
    params(("id" = String, Path, description = "Source ID")),
    request_body = UpdateSourceRequest,
    responses(
        (status = 200, description = "Source updated", body = SourceResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Source not found")
    ),
    tag = "sources"
)]
pub async fn update_source(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<UpdateSourceRequest>,
) -> Result<Json<SourceResponse>, AppError> {
    let source_id =
        uuid::Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid source ID".into()))?;

    if body.source_type.is_some() {
        return Err(AppError::BadRequest(
            "source_type cannot be changed after creation".into(),
        ));
    }

    let current = db::sources::get_source(&state.pool, source_id).await?;
    let is_editor = user.has_permission(Permission::ResourcesManage);
    let is_creator = current.created_by == user.id.to_string();

    if current.protected && !is_editor {
        return Err(AppError::Forbidden("This source is protected".into()));
    }
    if !is_editor && !is_creator {
        return Err(AppError::Forbidden(
            "You can only edit sources you created".into(),
        ));
    }
    if body.protected.is_some() && !is_editor {
        return Err(AppError::Forbidden(
            "Only editors can change the protected flag".into(),
        ));
    }

    let parent_source_id = body
        .parent_source_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid parent_source_id".into()))?;

    let translation_of_id = body
        .translation_of_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid translation_of_id".into()))?;

    validate_source_fields(
        body.title.as_deref(),
        body.title_display.as_deref(),
        body.publisher.as_deref(),
        body.journal_name.as_deref(),
        body.doi.as_deref(),
        body.edition.as_deref(),
        body.volume.as_deref(),
        body.url.as_deref(),
        body.isbn.as_deref(),
        body.publication_year,
        body.page_start,
        body.page_end,
    )?;

    let source = db::sources::update_source(
        &state.pool,
        source_id,
        body.title.as_deref(),
        body.title_display.as_deref(),
        body.publication_year,
        body.publisher.as_deref(),
        body.isbn.as_deref(),
        body.doi.as_deref(),
        body.edition.as_deref(),
        body.volume.as_deref(),
        body.journal_name.as_deref(),
        body.url.as_deref(),
        body.page_start,
        body.page_end,
        parent_source_id,
        translation_of_id,
        body.protected,
    )
    .await?;

    Ok(Json(source))
}

/// Link a person to a source with a role
#[utoipa::path(
    post,
    path = "/api/sources/{id}/persons",
    params(("id" = String, Path, description = "Source ID")),
    request_body = LinkSourcePersonRequest,
    responses(
        (status = 200, description = "Person linked to source"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "sources"
)]
pub async fn add_source_person(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<LinkSourcePersonRequest>,
) -> Result<Json<()>, AppError> {
    let source_id =
        uuid::Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid source ID".into()))?;

    guard_source_edit(&state.pool, &user, source_id).await?;

    let person_id = uuid::Uuid::parse_str(&body.person_id)
        .map_err(|_| AppError::BadRequest("Invalid person_id".into()))?;

    db::sources::link_source_person(
        &state.pool,
        source_id,
        person_id,
        &body.role,
        body.position.unwrap_or(0),
    )
    .await?;

    Ok(Json(()))
}

/// Remove a person from a source
#[utoipa::path(
    delete,
    path = "/api/sources/{id}/persons/{person_id}/{role}",
    params(
        ("id" = String, Path, description = "Source ID"),
        ("person_id" = String, Path, description = "Person ID"),
        ("role" = String, Path, description = "Role to remove")
    ),
    responses(
        (status = 200, description = "Person unlinked from source"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "sources"
)]
pub async fn remove_source_person(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, person_id, role)): Path<(String, String, String)>,
) -> Result<Json<()>, AppError> {
    let source_id =
        uuid::Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid source ID".into()))?;

    guard_source_edit(&state.pool, &user, source_id).await?;

    let person_uuid = uuid::Uuid::parse_str(&person_id)
        .map_err(|_| AppError::BadRequest("Invalid person ID".into()))?;

    db::sources::unlink_source_person(&state.pool, source_id, person_uuid, &role).await?;

    Ok(Json(()))
}

/// Delete a source. Blocks when any references exist (resources, child
/// sources, or article citations). Creator-only for non-editors.
#[utoipa::path(
    delete,
    path = "/api/sources/{id}",
    params(("id" = String, Path, description = "Source ID")),
    responses(
        (status = 200, description = "Source deleted"),
        (status = 400, description = "Source has active references"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Source not found")
    ),
    tag = "sources"
)]
pub async fn delete_source(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<()>, AppError> {
    let source_id =
        uuid::Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid source ID".into()))?;

    let current = db::sources::get_source(&state.pool, source_id).await?;
    let is_editor = user.has_permission(Permission::ResourcesManage);
    if current.protected && !is_editor {
        return Err(AppError::Forbidden("This source is protected".into()));
    }
    if !is_editor && current.created_by != user.id.to_string() {
        return Err(AppError::Forbidden(
            "You can only delete sources you created".into(),
        ));
    }

    let refs = db::sources::check_source_references(&state.pool, source_id, user.id).await?;
    if refs.total > 0 {
        return Err(AppError::BadRequest(format!(
            "Cannot delete: source has {} active reference(s). Remove references first.",
            refs.total
        )));
    }

    db::sources::delete_source(&state.pool, source_id).await?;
    Ok(Json(()))
}

/// Check if a source can be deleted (has active references)
#[utoipa::path(
    get,
    path = "/api/sources/{id}/references",
    params(("id" = String, Path, description = "Source ID")),
    responses(
        (status = 200, description = "Reference check result", body = ReferenceCheckResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    tag = "sources"
)]
pub async fn check_source_references(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<ReferenceCheckResponse>, AppError> {
    if !user.has_permission(Permission::ResourcesManage)
        && !user.has_permission(Permission::SourcesCreate)
    {
        return Err(AppError::Forbidden("Insufficient permissions".into()));
    }

    let source_id =
        uuid::Uuid::parse_str(&id).map_err(|_| AppError::BadRequest("Invalid source ID".into()))?;

    let response = db::sources::check_source_references(&state.pool, source_id, user.id).await?;

    Ok(Json(response))
}
