use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::resource::PersonResponse;

pub async fn search_persons(pool: &PgPool, query: &str) -> Result<Vec<PersonResponse>, AppError> {
    let pattern = format!("%{query}%");

    let rows = sqlx::query_as!(
        PersonRow,
        r#"SELECT id, name, sort_name
           FROM persons
           WHERE name ILIKE $1
           ORDER BY name
           LIMIT 20"#,
        pattern,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| PersonResponse {
            id: r.id.to_string(),
            name: r.name,
            sort_name: r.sort_name,
        })
        .collect())
}

pub async fn create_person(
    pool: &PgPool,
    name: &str,
    sort_name: Option<&str>,
) -> Result<PersonResponse, AppError> {
    let row = sqlx::query_as!(
        PersonRow,
        r#"INSERT INTO persons (name, sort_name)
           VALUES ($1, $2)
           RETURNING id, name, sort_name"#,
        name,
        sort_name,
    )
    .fetch_one(pool)
    .await?;

    Ok(PersonResponse {
        id: row.id.to_string(),
        name: row.name,
        sort_name: row.sort_name,
    })
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
           RETURNING id, name, sort_name"#,
        person_id,
        name,
        sort_name,
    )
    .fetch_one(pool)
    .await?;

    Ok(PersonResponse {
        id: row.id.to_string(),
        name: row.name,
        sort_name: row.sort_name,
    })
}

struct PersonRow {
    id: Uuid,
    name: String,
    sort_name: Option<String>,
}
