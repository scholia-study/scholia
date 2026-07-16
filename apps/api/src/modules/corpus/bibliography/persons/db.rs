use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::corpus::bibliography::models::PersonResponse;
use crate::system::error::AppError;

pub async fn search_persons(pool: &PgPool, query: &str) -> Result<Vec<PersonResponse>, AppError> {
    let pattern = format!("%{query}%");

    let rows = sqlx::query_as!(
        PersonRow,
        r#"SELECT id, name, sort_name, created_by AS "created_by!", protected
           FROM persons
           WHERE name ILIKE $1
           ORDER BY name
           LIMIT 20"#,
        pattern,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(row_to_response).collect())
}

/// Whether the person is linked to any source the given user does not
/// control — a source created by someone else, or one of the user's own
/// sources that is already cited on a public book page. Used to stop a
/// non-editor creator from renaming a person once it is displayed in
/// content they don't control.
pub async fn person_used_by_others(
    pool: &PgPool,
    person_id: Uuid,
    user_id: Uuid,
) -> Result<bool, AppError> {
    let used = sqlx::query_scalar!(
        r#"SELECT EXISTS(
               SELECT 1
               FROM source_persons sp
               JOIN sources s ON s.id = sp.source_id
               WHERE sp.person_id = $1
                 AND (
                     s.created_by <> $2
                     OR EXISTS(SELECT 1 FROM resources r
                               WHERE r.source_id = s.id AND r.archived_at IS NULL)
                 )
           ) AS "used!""#,
        person_id,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(used)
}

pub async fn get_person(pool: &PgPool, person_id: Uuid) -> Result<PersonResponse, AppError> {
    let row = sqlx::query_as!(
        PersonRow,
        r#"SELECT id, name, sort_name, created_by AS "created_by!", protected
           FROM persons
           WHERE id = $1"#,
        person_id,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Person not found".into()))?;

    Ok(row_to_response(row))
}

pub async fn create_person(
    pool: &PgPool,
    name: &str,
    sort_name: Option<&str>,
    created_by: Uuid,
) -> Result<PersonResponse, AppError> {
    let row = sqlx::query_as!(
        PersonRow,
        r#"INSERT INTO persons (name, sort_name, created_by)
           VALUES ($1, $2, $3)
           RETURNING id, name, sort_name, created_by AS "created_by!", protected"#,
        name,
        sort_name,
        created_by,
    )
    .fetch_one(pool)
    .await?;

    Ok(row_to_response(row))
}

pub async fn update_person(
    pool: &PgPool,
    person_id: Uuid,
    name: Option<&str>,
    sort_name: Option<&str>,
) -> Result<PersonResponse, AppError> {
    let row = sqlx::query_as!(
        PersonRow,
        r#"UPDATE persons
           SET name = COALESCE($2, name),
               sort_name = COALESCE($3, sort_name)
           WHERE id = $1
           RETURNING id, name, sort_name, created_by AS "created_by!", protected"#,
        person_id,
        name,
        sort_name,
    )
    .fetch_one(pool)
    .await?;

    Ok(row_to_response(row))
}

struct PersonRow {
    id: Uuid,
    name: String,
    sort_name: Option<String>,
    created_by: Uuid,
    protected: bool,
}

fn row_to_response(row: PersonRow) -> PersonResponse {
    PersonResponse {
        id: row.id.to_string(),
        name: row.name,
        sort_name: row.sort_name,
        created_by: row.created_by.to_string(),
        protected: row.protected,
    }
}
