use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::modules::collegia::models::{
    CollegiumDetailResponse, CollegiumMemberResponse, CollegiumResponse, CollegiumRole,
    JoinRequestResponse, ReviewVisibility,
};
use crate::system::error::{AppError, SqlxResultExt};

/// Lifetime creation caps (soft-deleted collegia count — `collegia` rows are
/// permanent, so counting `created_by` is the ledger).
pub const FREE_COLLEGIA_CREATED: i64 = 1;
pub const PAID_COLLEGIA_CREATED: i64 = 5;

/// Base slugs that would shadow static frontend routes under
/// `/user/collegia/…`.
const RESERVED_SLUGS: &[&str] = &["join", "discover", "new", "by-id"];

fn fmt_time(t: time::OffsetDateTime) -> String {
    t.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| t.to_string())
}

fn generate_slug(name: &str) -> String {
    let slug = slug::slugify(name);
    if RESERVED_SLUGS.contains(&slug.as_str()) {
        let suffix: u32 = rand::random::<u32>() % 999999;
        return format!("{slug}-{suffix:06}");
    }
    slug
}

fn generate_invite_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Membership-rule validation, pure so the invariants are unit-testable.
/// `steward_count`/`member_count` describe the collegium before the change.
mod rules {
    use crate::modules::collegia::models::CollegiumRole;
    use crate::system::error::AppError;

    pub fn validate_removal(
        is_self: bool,
        actor_role: CollegiumRole,
        target_role: CollegiumRole,
        steward_count: i64,
        member_count: i64,
    ) -> Result<(), AppError> {
        if is_self {
            if target_role == CollegiumRole::Steward && steward_count == 1 && member_count > 1 {
                return Err(AppError::BadRequest(
                    "Promote another steward before leaving the collegium".into(),
                ));
            }
            return Ok(());
        }
        if actor_role != CollegiumRole::Steward {
            return Err(AppError::Forbidden(
                "Only collegium stewards can remove members".into(),
            ));
        }
        if target_role == CollegiumRole::Steward {
            return Err(AppError::Forbidden(
                "Stewards cannot be removed; they must step down first".into(),
            ));
        }
        Ok(())
    }

    pub fn validate_role_change(
        is_self: bool,
        actor_role: CollegiumRole,
        target_role: CollegiumRole,
        new_role: CollegiumRole,
        steward_count: i64,
    ) -> Result<(), AppError> {
        if new_role == target_role {
            return Ok(());
        }
        match new_role {
            CollegiumRole::Steward => {
                if actor_role != CollegiumRole::Steward {
                    return Err(AppError::Forbidden(
                        "Only collegium stewards can promote members".into(),
                    ));
                }
                Ok(())
            }
            CollegiumRole::Member => {
                if !is_self {
                    return Err(AppError::Forbidden(
                        "Stewards can only step down themselves".into(),
                    ));
                }
                if steward_count == 1 {
                    return Err(AppError::BadRequest(
                        "Promote another steward before stepping down".into(),
                    ));
                }
                Ok(())
            }
        }
    }
}

struct CollegiumRow {
    id: Uuid,
    name: String,
    slug: String,
    description: Option<String>,
    is_private: bool,
    review_visibility: ReviewVisibility,
    created_at: time::OffsetDateTime,
    member_count: i64,
    my_role: Option<CollegiumRole>,
    my_pending_request: bool,
}

fn collegium_response(r: CollegiumRow) -> CollegiumResponse {
    CollegiumResponse {
        id: r.id.to_string(),
        name: r.name,
        slug: r.slug,
        description: r.description,
        is_private: r.is_private,
        member_count: r.member_count,
        my_role: r.my_role,
        my_pending_request: r.my_pending_request,
        review_visibility: r.review_visibility,
        created_at: fmt_time(r.created_at),
    }
}

pub async fn count_created_by(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM collegia WHERE created_by = $1"#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub struct CollegiumCreate<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub is_private: bool,
    pub review_visibility: ReviewVisibility,
    pub created_by: Uuid,
}

pub async fn create_collegium(
    pool: &PgPool,
    entry: CollegiumCreate<'_>,
) -> Result<String, AppError> {
    let mut slug = generate_slug(entry.name);
    if slug.is_empty() {
        return Err(AppError::BadRequest(
            "Collegium name must contain letters or numbers".into(),
        ));
    }
    // Names may repeat across collegia; only the slug must be unique.
    let taken = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM collegia WHERE slug = $1) AS "taken!""#,
        slug,
    )
    .fetch_one(pool)
    .await?;
    if taken {
        let suffix: u32 = rand::random::<u32>() % 999999;
        slug = format!("{slug}-{suffix:06}");
    }

    let mut tx = pool.begin().await?;
    let collegium_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO collegia
               (name, slug, description, is_private, review_visibility, created_by)
           VALUES ($1, $2, $3, $4, $5::collegium_review_visibility, $6)
           RETURNING id"#,
        entry.name,
        slug,
        entry.description,
        entry.is_private,
        entry.review_visibility as _,
        entry.created_by,
    )
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query!(
        r#"INSERT INTO collegium_members (collegium_id, user_id, role)
           VALUES ($1, $2, 'steward')"#,
        collegium_id,
        entry.created_by,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(slug)
}

pub async fn list_my_collegia(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<CollegiumResponse>, AppError> {
    let rows = sqlx::query_as!(
        CollegiumRow,
        r#"SELECT g.id, g.name, g.slug, g.description, g.is_private,
               g.review_visibility AS "review_visibility: ReviewVisibility", g.created_at,
               (SELECT COUNT(*) FROM collegium_members gm
                WHERE gm.collegium_id = g.id) AS "member_count!",
               my.role AS "my_role?: CollegiumRole",
               false AS "my_pending_request!"
           FROM collegia g
           JOIN collegium_members my ON my.collegium_id = g.id AND my.user_id = $1
           WHERE g.deleted_at IS NULL
           ORDER BY my.joined_at DESC"#,
        user_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(collegium_response).collect())
}

pub async fn discover_collegia(
    pool: &PgPool,
    user_id: Uuid,
    q: Option<&str>,
    page: i32,
    per_page: i32,
) -> Result<(Vec<CollegiumResponse>, i64), AppError> {
    let offset = i64::from((page - 1) * per_page);
    let limit = i64::from(per_page);
    let rows = sqlx::query_as!(
        CollegiumRow,
        r#"SELECT g.id, g.name, g.slug, g.description, g.is_private,
               g.review_visibility AS "review_visibility: ReviewVisibility", g.created_at,
               (SELECT COUNT(*) FROM collegium_members gm
                WHERE gm.collegium_id = g.id) AS "member_count!",
               my.role AS "my_role?: CollegiumRole",
               EXISTS(SELECT 1 FROM collegium_join_requests jr
                      WHERE jr.collegium_id = g.id AND jr.user_id = $1
                        AND jr.status = 'pending') AS "my_pending_request!"
           FROM collegia g
           LEFT JOIN collegium_members my ON my.collegium_id = g.id AND my.user_id = $1
           WHERE g.deleted_at IS NULL AND NOT g.is_private
             AND ($2::TEXT IS NULL
                  OR g.name ILIKE '%' || $2 || '%'
                  OR COALESCE(g.description, '') ILIKE '%' || $2 || '%')
           ORDER BY g.name
           LIMIT $3 OFFSET $4"#,
        user_id,
        q,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM collegia g
           WHERE g.deleted_at IS NULL AND NOT g.is_private
             AND ($1::TEXT IS NULL
                  OR g.name ILIKE '%' || $1 || '%'
                  OR COALESCE(g.description, '') ILIKE '%' || $1 || '%')"#,
        q,
    )
    .fetch_one(pool)
    .await?;

    Ok((rows.into_iter().map(collegium_response).collect(), total))
}

/// Full detail with visibility applied: private collegia 404 for
/// non-members (existence must not leak), the member list is member-only,
/// invite token and pending-join count are admin-only.
pub async fn get_collegium_detail(
    pool: &PgPool,
    user_id: Uuid,
    slug: &str,
) -> Result<CollegiumDetailResponse, AppError> {
    let row = sqlx::query!(
        r#"SELECT g.id, g.name, g.slug, g.description, g.is_private,
               g.review_visibility AS "review_visibility: ReviewVisibility",
               g.invite_token, g.created_at,
               (SELECT COUNT(*) FROM collegium_members gm
                WHERE gm.collegium_id = g.id) AS "member_count!",
               my.role AS "my_role?: CollegiumRole",
               EXISTS(SELECT 1 FROM collegium_join_requests jr
                      WHERE jr.collegium_id = g.id AND jr.user_id = $2
                        AND jr.status = 'pending') AS "my_pending_request!"
           FROM collegia g
           LEFT JOIN collegium_members my ON my.collegium_id = g.id AND my.user_id = $2
           WHERE g.slug = $1 AND g.deleted_at IS NULL"#,
        slug,
        user_id,
    )
    .fetch_one(pool)
    .await
    .on_missing(|| AppError::NotFound("Collegium not found".into()))?;

    let is_member = row.my_role.is_some();
    let is_steward = row.my_role == Some(CollegiumRole::Steward);
    if row.is_private && !is_member {
        return Err(AppError::NotFound("Collegium not found".into()));
    }

    let members = if is_member {
        Some(list_members(pool, row.id).await?)
    } else {
        None
    };
    let pending_join_request_count = if is_steward {
        Some(
            sqlx::query_scalar!(
                r#"SELECT COUNT(*) AS "count!" FROM collegium_join_requests
                   WHERE collegium_id = $1 AND status = 'pending'"#,
                row.id,
            )
            .fetch_one(pool)
            .await?,
        )
    } else {
        None
    };

    Ok(CollegiumDetailResponse {
        id: row.id.to_string(),
        name: row.name,
        slug: row.slug,
        description: row.description,
        is_private: row.is_private,
        member_count: row.member_count,
        my_role: row.my_role,
        my_pending_request: row.my_pending_request,
        review_visibility: row.review_visibility,
        created_at: fmt_time(row.created_at),
        members,
        invite_token: if is_steward { row.invite_token } else { None },
        pending_join_request_count,
    })
}

async fn list_members(
    pool: &PgPool,
    collegium_id: Uuid,
) -> Result<Vec<CollegiumMemberResponse>, AppError> {
    struct Row {
        user_id: Uuid,
        display_name: String,
        handle: Option<String>,
        role: CollegiumRole,
        joined_at: time::OffsetDateTime,
    }
    let rows = sqlx::query_as!(
        Row,
        r#"SELECT gm.user_id, u.display_name, u.handle,
               gm.role AS "role: CollegiumRole", gm.joined_at
           FROM collegium_members gm
           JOIN users u ON u.id = gm.user_id
           WHERE gm.collegium_id = $1
           ORDER BY gm.role, gm.joined_at"#,
        collegium_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| CollegiumMemberResponse {
            user_id: r.user_id.to_string(),
            display_name: r.display_name,
            handle: r.handle,
            role: r.role,
            joined_at: fmt_time(r.joined_at),
        })
        .collect())
}

/// Serialize membership mutations per collegium; also the not-deleted gate.
async fn lock_collegium(conn: &mut PgConnection, slug: &str) -> Result<Uuid, AppError> {
    sqlx::query_scalar!(
        r#"SELECT id FROM collegia WHERE slug = $1 AND deleted_at IS NULL FOR UPDATE"#,
        slug,
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or_else(|| AppError::NotFound("Collegium not found".into()))
}

async fn member_role_conn(
    conn: &mut PgConnection,
    collegium_id: Uuid,
    user_id: Uuid,
) -> Result<Option<CollegiumRole>, AppError> {
    let role = sqlx::query_scalar!(
        r#"SELECT role AS "role: CollegiumRole" FROM collegium_members
           WHERE collegium_id = $1 AND user_id = $2"#,
        collegium_id,
        user_id,
    )
    .fetch_optional(&mut *conn)
    .await?;
    Ok(role)
}

/// Group id + the viewer's live role, by slug — the workshop-queue
/// access check. None when the collegium is missing, deleted, or the viewer
/// is not a member (all indistinguishable, deliberately).
pub async fn member_role_by_slug(
    pool: &PgPool,
    slug: &str,
    user_id: Uuid,
) -> Result<Option<(Uuid, CollegiumRole)>, AppError> {
    let row = sqlx::query!(
        r#"SELECT g.id, gm.role AS "role: CollegiumRole"
           FROM collegia g
           JOIN collegium_members gm ON gm.collegium_id = g.id AND gm.user_id = $2
           WHERE g.slug = $1 AND g.deleted_at IS NULL"#,
        slug,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| (r.id, r.role)))
}

/// The collegium's current review visibility (`members` | `stewards`), None
/// when the collegium is missing or deleted. Submission snapshots this onto
/// the request as `member_visible`.
pub async fn review_visibility(
    pool: &PgPool,
    collegium_id: Uuid,
) -> Result<Option<ReviewVisibility>, AppError> {
    let visibility = sqlx::query_scalar!(
        r#"SELECT review_visibility AS "review_visibility: ReviewVisibility"
           FROM collegia WHERE id = $1 AND deleted_at IS NULL"#,
        collegium_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(visibility)
}

/// Live membership role — the writing domain's review-access check.
pub async fn member_role(
    pool: &PgPool,
    collegium_id: Uuid,
    user_id: Uuid,
) -> Result<Option<CollegiumRole>, AppError> {
    let role = sqlx::query_scalar!(
        r#"SELECT gm.role AS "role: CollegiumRole"
           FROM collegium_members gm
           JOIN collegia g ON g.id = gm.collegium_id
           WHERE gm.collegium_id = $1 AND gm.user_id = $2 AND g.deleted_at IS NULL"#,
        collegium_id,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(role)
}

struct MemberCounts {
    stewards: i64,
    members: i64,
}

async fn member_counts(
    conn: &mut PgConnection,
    collegium_id: Uuid,
) -> Result<MemberCounts, AppError> {
    let row = sqlx::query!(
        r#"SELECT COUNT(*) FILTER (WHERE role = 'steward') AS "stewards!",
               COUNT(*) AS "members!"
           FROM collegium_members WHERE collegium_id = $1"#,
        collegium_id,
    )
    .fetch_one(&mut *conn)
    .await?;
    Ok(MemberCounts {
        stewards: row.stewards,
        members: row.members,
    })
}

pub struct CollegiumUpdate<'a> {
    pub name: Option<&'a str>,
    /// `Some("")` clears the description.
    pub description: Option<&'a str>,
    pub is_private: Option<bool>,
    /// Governs future submissions only; existing requests keep the
    /// visibility they were submitted under (`member_visible`).
    pub review_visibility: Option<ReviewVisibility>,
}

/// Admin-only metadata update; the slug is immutable (it lives in
/// shareable URLs).
pub async fn update_collegium(
    pool: &PgPool,
    slug: &str,
    actor: Uuid,
    patch: CollegiumUpdate<'_>,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let collegium_id = lock_collegium(&mut tx, slug).await?;
    require_steward(&mut tx, collegium_id, actor).await?;
    sqlx::query!(
        r#"UPDATE collegia SET
               name = COALESCE($2, name),
               description = CASE
                   WHEN $3::TEXT IS NULL THEN description
                   WHEN $3 = '' THEN NULL
                   ELSE $3
               END,
               is_private = COALESCE($4, is_private),
               review_visibility = COALESCE(
                   $5::collegium_review_visibility, review_visibility)
           WHERE id = $1"#,
        collegium_id,
        patch.name,
        patch.description,
        patch.is_private,
        patch.review_visibility as _,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

async fn require_steward(
    conn: &mut PgConnection,
    collegium_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    match member_role_conn(conn, collegium_id, user_id).await? {
        Some(CollegiumRole::Steward) => Ok(()),
        // Non-members get the same 404 as for a private collegium they can't
        // see; members without steward get a plain 403.
        Some(_) => Err(AppError::Forbidden("Collegium steward required".into())),
        None => Err(AppError::NotFound("Collegium not found".into())),
    }
}

/// Generate (or rotate) the invite link token. Rotation revokes any
/// previously shared link.
pub async fn rotate_invite_token(
    pool: &PgPool,
    slug: &str,
    actor: Uuid,
) -> Result<String, AppError> {
    let token = generate_invite_token();
    let mut tx = pool.begin().await?;
    let collegium_id = lock_collegium(&mut tx, slug).await?;
    require_steward(&mut tx, collegium_id, actor).await?;
    sqlx::query!(
        r#"UPDATE collegia SET invite_token = $2 WHERE id = $1"#,
        collegium_id,
        token,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(token)
}

pub async fn disable_invite_token(pool: &PgPool, slug: &str, actor: Uuid) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let collegium_id = lock_collegium(&mut tx, slug).await?;
    require_steward(&mut tx, collegium_id, actor).await?;
    sqlx::query!(
        r#"UPDATE collegia SET invite_token = NULL WHERE id = $1"#,
        collegium_id,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub struct JoinByToken {
    pub slug: String,
    pub name: String,
    pub already_member: bool,
}

/// Redeem an invite link: immediate membership, valid for private and
/// public collegia alike. A pending ask-to-join request from the same user
/// is closed as approved so it doesn't linger in the steward queue.
pub async fn join_by_token(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
) -> Result<JoinByToken, AppError> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query!(
        r#"SELECT id, slug, name FROM collegia
           WHERE invite_token = $1 AND deleted_at IS NULL
           FOR UPDATE"#,
        token,
    )
    .fetch_one(&mut *tx)
    .await
    .on_missing(|| AppError::NotFound("Invite link is invalid or has been revoked".into()))?;

    let inserted = sqlx::query!(
        r#"INSERT INTO collegium_members (collegium_id, user_id)
           VALUES ($1, $2)
           ON CONFLICT (collegium_id, user_id) DO NOTHING"#,
        row.id,
        user_id,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if inserted > 0 {
        sqlx::query!(
            r#"UPDATE collegium_join_requests
               SET status = 'approved', decided_at = now()
               WHERE collegium_id = $1 AND user_id = $2 AND status = 'pending'"#,
            row.id,
            user_id,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    Ok(JoinByToken {
        slug: row.slug,
        name: row.name,
        already_member: inserted == 0,
    })
}

/// Ask-to-join requests this user filed in the last 24h (rate limiting).
pub async fn count_recent_join_requests(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM collegium_join_requests
           WHERE user_id = $1 AND created_at > now() - INTERVAL '24 hours'"#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// File an ask-to-join request. Public collegia only — private collegia are
/// joinable via invite link alone, and 404 here so they don't leak.
pub async fn create_join_request(pool: &PgPool, user_id: Uuid, slug: &str) -> Result<(), AppError> {
    let row = sqlx::query!(
        r#"SELECT g.id, g.is_private,
               EXISTS(SELECT 1 FROM collegium_members gm
                      WHERE gm.collegium_id = g.id AND gm.user_id = $2) AS "is_member!"
           FROM collegia g
           WHERE g.slug = $1 AND g.deleted_at IS NULL"#,
        slug,
        user_id,
    )
    .fetch_one(pool)
    .await
    .on_missing(|| AppError::NotFound("Collegium not found".into()))?;
    if row.is_private {
        return Err(AppError::NotFound("Collegium not found".into()));
    }
    if row.is_member {
        return Err(AppError::BadRequest(
            "You are already a member of this collegium".into(),
        ));
    }

    sqlx::query!(
        r#"INSERT INTO collegium_join_requests (collegium_id, user_id) VALUES ($1, $2)"#,
        row.id,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            AppError::Conflict("You already have a pending request for this collegium".into())
        }
        _ => AppError::from(e),
    })?;
    Ok(())
}

pub async fn list_join_requests(
    pool: &PgPool,
    slug: &str,
    actor: Uuid,
) -> Result<Vec<JoinRequestResponse>, AppError> {
    let mut tx = pool.begin().await?;
    let collegium_id = lock_collegium(&mut tx, slug).await?;
    require_steward(&mut tx, collegium_id, actor).await?;
    tx.commit().await?;

    struct Row {
        id: Uuid,
        user_id: Uuid,
        display_name: String,
        handle: Option<String>,
        created_at: time::OffsetDateTime,
    }
    let rows = sqlx::query_as!(
        Row,
        r#"SELECT jr.id, jr.user_id, u.display_name, u.handle, jr.created_at
           FROM collegium_join_requests jr
           JOIN users u ON u.id = jr.user_id
           WHERE jr.collegium_id = $1 AND jr.status = 'pending'
           ORDER BY jr.created_at"#,
        collegium_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| JoinRequestResponse {
            id: r.id.to_string(),
            user_id: r.user_id.to_string(),
            display_name: r.display_name,
            handle: r.handle,
            created_at: fmt_time(r.created_at),
        })
        .collect())
}

/// Admin decision on a pending join request; approval creates the
/// membership in the same transaction.
pub async fn decide_join_request(
    pool: &PgPool,
    slug: &str,
    actor: Uuid,
    request_id: Uuid,
    approve: bool,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let collegium_id = lock_collegium(&mut tx, slug).await?;
    require_steward(&mut tx, collegium_id, actor).await?;

    let requester = sqlx::query_scalar!(
        r#"UPDATE collegium_join_requests
           SET status = CASE WHEN $3 THEN 'approved'::collegium_join_request_status
                             ELSE 'rejected'::collegium_join_request_status END,
               decided_by = $4, decided_at = now()
           WHERE id = $1 AND collegium_id = $2 AND status = 'pending'
           RETURNING user_id"#,
        request_id,
        collegium_id,
        approve,
        actor,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::Conflict("This request has already been decided".into()))?;

    if approve {
        sqlx::query!(
            r#"INSERT INTO collegium_members (collegium_id, user_id)
               VALUES ($1, $2)
               ON CONFLICT (collegium_id, user_id) DO NOTHING"#,
            collegium_id,
            requester,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Leave or kick. Enforces the steward invariant, withdraws the departing
/// member's pending review requests to this collegium, and soft-deletes the
/// collegium (withdrawing all its pending requests) when the last member
/// leaves. Returns true when the collegium was deleted.
pub async fn remove_member(
    pool: &PgPool,
    slug: &str,
    actor: Uuid,
    target: Uuid,
) -> Result<bool, AppError> {
    let mut tx = pool.begin().await?;
    let collegium_id = lock_collegium(&mut tx, slug).await?;

    let actor_role = member_role_conn(&mut tx, collegium_id, actor)
        .await?
        .ok_or_else(|| AppError::NotFound("Collegium not found".into()))?;
    let target_role = member_role_conn(&mut tx, collegium_id, target)
        .await?
        .ok_or_else(|| AppError::NotFound("Not a member of this collegium".into()))?;
    let counts = member_counts(&mut tx, collegium_id).await?;

    rules::validate_removal(
        actor == target,
        actor_role,
        target_role,
        counts.stewards,
        counts.members,
    )?;

    sqlx::query!(
        r#"DELETE FROM collegium_members WHERE collegium_id = $1 AND user_id = $2"#,
        collegium_id,
        target,
    )
    .execute(&mut *tx)
    .await?;
    crate::modules::writing::withdraw_pending_collegium_requests(
        &mut tx,
        collegium_id,
        Some(target),
    )
    .await?;

    let collegium_deleted = counts.members == 1;
    if collegium_deleted {
        sqlx::query!(
            r#"UPDATE collegia SET deleted_at = now(), invite_token = NULL WHERE id = $1"#,
            collegium_id,
        )
        .execute(&mut *tx)
        .await?;
        crate::modules::writing::withdraw_pending_collegium_requests(&mut tx, collegium_id, None)
            .await?;
    }
    tx.commit().await?;
    Ok(collegium_deleted)
}

/// Promote (steward action) or self-demote, keeping the ≥1-steward invariant.
pub async fn update_member_role(
    pool: &PgPool,
    slug: &str,
    actor: Uuid,
    target: Uuid,
    new_role: CollegiumRole,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let collegium_id = lock_collegium(&mut tx, slug).await?;

    let actor_role = member_role_conn(&mut tx, collegium_id, actor)
        .await?
        .ok_or_else(|| AppError::NotFound("Collegium not found".into()))?;
    let target_role = member_role_conn(&mut tx, collegium_id, target)
        .await?
        .ok_or_else(|| AppError::NotFound("Not a member of this collegium".into()))?;
    let counts = member_counts(&mut tx, collegium_id).await?;

    rules::validate_role_change(
        actor == target,
        actor_role,
        target_role,
        new_role,
        counts.stewards,
    )?;

    sqlx::query!(
        r#"UPDATE collegium_members SET role = $3::collegium_member_role
           WHERE collegium_id = $1 AND user_id = $2"#,
        collegium_id,
        target,
        new_role as _,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::rules::{validate_removal, validate_role_change};
    use crate::modules::collegia::models::CollegiumRole::{Member, Steward};

    #[test]
    fn member_may_always_leave() {
        assert!(validate_removal(true, Member, Member, 1, 5).is_ok());
    }

    #[test]
    fn steward_may_leave_when_another_steward_remains() {
        assert!(validate_removal(true, Steward, Steward, 2, 5).is_ok());
    }

    #[test]
    fn last_steward_cannot_leave_a_populated_collegium() {
        assert!(validate_removal(true, Steward, Steward, 1, 5).is_err());
    }

    #[test]
    fn sole_member_steward_may_leave() {
        assert!(validate_removal(true, Steward, Steward, 1, 1).is_ok());
    }

    #[test]
    fn only_stewards_kick_and_only_members_are_kickable() {
        assert!(validate_removal(false, Member, Member, 1, 3).is_err());
        assert!(validate_removal(false, Steward, Steward, 2, 3).is_err());
        assert!(validate_removal(false, Steward, Member, 1, 3).is_ok());
    }

    #[test]
    fn promote_requires_steward() {
        assert!(validate_role_change(false, Steward, Member, Steward, 1).is_ok());
        assert!(validate_role_change(false, Member, Member, Steward, 1).is_err());
    }

    #[test]
    fn step_down_is_self_only_and_never_the_last_steward() {
        assert!(validate_role_change(true, Steward, Steward, Member, 2).is_ok());
        assert!(validate_role_change(false, Steward, Steward, Member, 2).is_err());
        assert!(validate_role_change(true, Steward, Steward, Member, 1).is_err());
    }

    #[test]
    fn same_role_is_a_noop() {
        assert!(validate_role_change(false, Member, Member, Member, 1).is_ok());
    }
}
