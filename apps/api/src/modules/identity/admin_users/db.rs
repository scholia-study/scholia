use sqlx::PgPool;
use uuid::Uuid;

use super::MANAGEABLE_ROLES;
use super::models::{AdminUserListResponse, AdminUserRow};
use crate::system::error::{AppError, SqlxResultExt};

struct UserRow {
    id: Uuid,
    email: String,
    display_name: String,
    handle: Option<String>,
    email_verified: bool,
    created_at: time::OffsetDateTime,
    roles: Vec<String>,
}

fn user_from_row(r: UserRow) -> AdminUserRow {
    AdminUserRow {
        id: r.id.to_string(),
        email: r.email,
        display_name: r.display_name,
        handle: r.handle,
        roles: r.roles,
        email_verified: r.email_verified,
        created_at: r
            .created_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
    }
}

/// Escape LIKE wildcards so a search term is matched literally.
fn like_pattern(search: &str) -> String {
    let escaped = search
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

pub async fn list_users(
    pool: &PgPool,
    search: Option<&str>,
    page: u32,
    per_page: u32,
) -> Result<AdminUserListResponse, AppError> {
    let offset = ((page.saturating_sub(1)) as i64) * per_page as i64;
    let limit = per_page as i64;
    let pattern = search.map(like_pattern);

    let rows = sqlx::query_as!(
        UserRow,
        r#"SELECT u.id,
                  u.email,
                  u.display_name,
                  u.handle,
                  (u.email_verified_at IS NOT NULL) AS "email_verified!",
                  u.created_at,
                  COALESCE(ARRAY(
                      SELECT r.name
                      FROM user_roles ur
                      JOIN roles r ON r.id = ur.role_id
                      WHERE ur.user_id = u.id
                      ORDER BY r.name
                  ), '{}') AS "roles!: Vec<String>"
           FROM users u
           WHERE $1::text IS NULL
              OR u.email ILIKE $2
              OR u.display_name ILIKE $2
              OR u.handle ILIKE $2
           ORDER BY u.created_at DESC
           LIMIT $3 OFFSET $4"#,
        search,
        pattern,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM users u
           WHERE $1::text IS NULL
              OR u.email ILIKE $2
              OR u.display_name ILIKE $2
              OR u.handle ILIKE $2"#,
        search,
        pattern,
    )
    .fetch_one(pool)
    .await?;

    Ok(AdminUserListResponse {
        users: rows.into_iter().map(user_from_row).collect(),
        total,
        page,
        per_page,
    })
}

pub async fn get_user(pool: &PgPool, id: Uuid) -> Result<AdminUserRow, AppError> {
    let row = sqlx::query_as!(
        UserRow,
        r#"SELECT u.id,
                  u.email,
                  u.display_name,
                  u.handle,
                  (u.email_verified_at IS NOT NULL) AS "email_verified!",
                  u.created_at,
                  COALESCE(ARRAY(
                      SELECT r.name
                      FROM user_roles ur
                      JOIN roles r ON r.id = ur.role_id
                      WHERE ur.user_id = u.id
                      ORDER BY r.name
                  ), '{}') AS "roles!: Vec<String>"
           FROM users u
           WHERE u.id = $1"#,
        id,
    )
    .fetch_one(pool)
    .await
    .on_missing(|| AppError::NotFound("User not found".into()))?;
    Ok(user_from_row(row))
}

/// Replace the user's *manageable* roles with `desired` in a single
/// transaction. Paid tiers and the default `user` role are left as-is.
/// Aborts (rolling back) if the change would leave the system with no
/// admin. `desired` is assumed pre-validated against `MANAGEABLE_ROLES`.
pub async fn set_user_roles(
    pool: &PgPool,
    target_id: Uuid,
    desired: &[String],
) -> Result<AdminUserRow, AppError> {
    let mut tx = pool.begin().await?;

    let exists: Option<Uuid> =
        sqlx::query_scalar!(r#"SELECT id FROM users WHERE id = $1"#, target_id)
            .fetch_optional(&mut *tx)
            .await?;
    if exists.is_none() {
        return Err(AppError::NotFound("User not found".into()));
    }

    let manageable: Vec<String> = MANAGEABLE_ROLES.iter().map(|s| (*s).to_string()).collect();

    // Clear only the manageable slice; paid tiers and `user` are preserved.
    sqlx::query!(
        r#"DELETE FROM user_roles
           WHERE user_id = $1
             AND role_id IN (SELECT id FROM roles WHERE name = ANY($2))"#,
        target_id,
        &manageable,
    )
    .execute(&mut *tx)
    .await?;

    if !desired.is_empty() {
        sqlx::query!(
            r#"INSERT INTO user_roles (user_id, role_id)
               SELECT $1, id FROM roles WHERE name = ANY($2)
               ON CONFLICT DO NOTHING"#,
            target_id,
            desired,
        )
        .execute(&mut *tx)
        .await?;
    }

    // Guardrail: never let the last admin be demoted away.
    let admin_count: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM user_roles ur
           JOIN roles r ON r.id = ur.role_id
           WHERE r.name = 'admin'"#,
    )
    .fetch_one(&mut *tx)
    .await?;
    if admin_count == 0 {
        // tx drops without commit → rollback.
        return Err(AppError::BadRequest(
            "Cannot remove the last remaining admin.".into(),
        ));
    }

    tx.commit().await?;
    get_user(pool, target_id).await
}
