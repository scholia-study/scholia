use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::system::auth::handle::MAX_HANDLE_LEN;
use crate::system::auth::permissions::filter_public_roles;
use crate::system::error::AppError;

// ── Handle uniqueness ──────────────────────────────────────

/// Returns true if `handle` is currently held by a *different* user, or
/// was previously released by a different user. Per the no-recycle rule
/// (see `released_handles`), a handle that any other user has ever
/// owned cannot be claimed by anyone else — but the original holder can
/// rename back to it.
pub async fn is_handle_taken_by_other(
    pool: &PgPool,
    handle: &str,
    exclude_user_id: Option<Uuid>,
) -> Result<bool, AppError> {
    let exclude = exclude_user_id.unwrap_or(Uuid::nil());
    let taken: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM users WHERE handle = $1 AND id != $2
               UNION ALL
               SELECT 1 FROM released_handles WHERE handle = $1 AND user_id != $2
           ) AS "taken!""#,
        handle,
        exclude,
    )
    .fetch_one(pool)
    .await?;
    Ok(taken)
}

/// Find an available handle starting from `base`, appending `-2`, `-3`,
/// … on collision. Used at signup to seed `users.handle` from a derived
/// candidate that may already be taken.
///
/// Falls back to `user-<8-hex>` if the base derivation produced an
/// empty string (e.g. an all-emoji display name).
pub async fn claim_unique_handle(pool: &PgPool, base: &str) -> Result<String, AppError> {
    let mut candidate = if base.is_empty() {
        format!("user-{:08x}", rand::random::<u32>())
    } else {
        base.to_string()
    };

    // Trim to leave room for a `-NNN` suffix on collision.
    if candidate.chars().count() > MAX_HANDLE_LEN - 4 {
        candidate = candidate.chars().take(MAX_HANDLE_LEN - 4).collect();
        while candidate.ends_with('-') {
            candidate.pop();
        }
    }

    let mut suffix: u32 = 1;
    loop {
        let attempt = if suffix == 1 {
            candidate.clone()
        } else {
            format!("{candidate}-{suffix}")
        };
        if !is_handle_taken_by_other(pool, &attempt, None).await? {
            return Ok(attempt);
        }
        suffix += 1;
        if suffix > 1_000 {
            // Astronomically unlikely; fail rather than spin.
            return Err(AppError::Internal(
                "Could not allocate a unique handle".into(),
            ));
        }
    }
}

/// Mark the previous handle as released so no one else can claim it,
/// while leaving the door open for the original owner to take it back.
pub async fn record_released_handle(
    pool: &PgPool,
    user_id: Uuid,
    handle: &str,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"INSERT INTO released_handles (handle, user_id)
           VALUES ($1, $2)
           ON CONFLICT (handle) DO NOTHING"#,
        handle,
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ── Public profile lookups ─────────────────────────────────

pub struct PublicProfileRow {
    pub id: Uuid,
    pub handle: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub title: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: time::OffsetDateTime,
}

pub async fn get_public_profile_by_handle(
    pool: &PgPool,
    handle: &str,
) -> Result<PublicProfileRow, AppError> {
    sqlx::query_as!(
        PublicProfileRow,
        r#"SELECT id,
                  handle           AS "handle!",
                  display_name,
                  bio,
                  title,
                  location,
                  website_url,
                  avatar_url,
                  created_at
           FROM users
           WHERE handle = $1"#,
        handle,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("User not found".into()))
}

pub async fn get_handle_by_id(pool: &PgPool, user_id: Uuid) -> Result<String, AppError> {
    let h = sqlx::query_scalar!(r#"SELECT handle FROM users WHERE id = $1"#, user_id,)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::NotFound("User not found".into()))?;
    h.ok_or_else(|| AppError::NotFound("User has no handle".into()))
}

/// Resolve the role names assigned to a user. Used to compute
/// `public_roles` on profile and article responses.
pub async fn list_role_names(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, AppError> {
    let rows: Vec<String> = sqlx::query_scalar!(
        r#"SELECT r.name
           FROM user_roles ur
           JOIN roles r ON r.id = ur.role_id
           WHERE ur.user_id = $1
           ORDER BY r.name"#,
        user_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Batched lookup of *public* role chips (filtered via
/// `filter_public_roles`) for a set of user ids. Returns a map keyed by
/// user id; users with no public roles are present with an empty Vec.
pub async fn list_public_roles_for(
    pool: &PgPool,
    user_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<String>>, AppError> {
    let mut out: HashMap<Uuid, Vec<String>> = user_ids.iter().map(|id| (*id, Vec::new())).collect();
    if user_ids.is_empty() {
        return Ok(out);
    }
    struct Row {
        user_id: Uuid,
        name: String,
    }
    let rows: Vec<Row> = sqlx::query_as!(
        Row,
        r#"SELECT ur.user_id, r.name
           FROM user_roles ur
           JOIN roles r ON r.id = ur.role_id
           WHERE ur.user_id = ANY($1)"#,
        user_ids,
    )
    .fetch_all(pool)
    .await?;

    let mut buckets: HashMap<Uuid, Vec<String>> = HashMap::new();
    for r in rows {
        buckets.entry(r.user_id).or_default().push(r.name);
    }
    for (id, roles) in buckets {
        out.insert(id, filter_public_roles(&roles));
    }
    Ok(out)
}
