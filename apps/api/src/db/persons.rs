use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::resource::PersonResponse;

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
