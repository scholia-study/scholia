use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::modules::writing::articles::models::EditorialLabelResponse;
use crate::system::error::AppError;

struct LabelRow {
    id: Uuid,
    name: String,
    slug: String,
    revokes_on_edit: bool,
}

fn to_response(r: LabelRow) -> EditorialLabelResponse {
    EditorialLabelResponse {
        id: r.id.to_string(),
        name: r.name,
        slug: r.slug,
        revokes_on_edit: r.revokes_on_edit,
    }
}

/// All editorial labels, ordered by `sort_order`. Public — used by both
/// readers (to know what chips exist) and editors (to populate the manage
/// modal).
pub async fn list_labels(pool: &PgPool) -> Result<Vec<EditorialLabelResponse>, AppError> {
    let rows = sqlx::query_as!(
        LabelRow,
        r#"SELECT id, name, slug, revokes_on_edit FROM editorial_labels
           ORDER BY sort_order, name"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(to_response).collect())
}

/// Labels currently applied to one article, ordered by `sort_order`.
pub async fn list_for_article(
    pool: &PgPool,
    article_id: Uuid,
) -> Result<Vec<EditorialLabelResponse>, AppError> {
    let rows = sqlx::query_as!(
        LabelRow,
        r#"SELECT el.id, el.name, el.slug, el.revokes_on_edit
           FROM editorial_labels el
           JOIN article_editorial_labels ael ON ael.label_id = el.id
           WHERE ael.article_id = $1
           ORDER BY el.sort_order, el.name"#,
        article_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(to_response).collect())
}

struct ArticleLabelRow {
    article_id: Uuid,
    id: Uuid,
    name: String,
    slug: String,
    revokes_on_edit: bool,
}

/// Batch-load labels for many articles in one query. Mirrors
/// `load_articles_topics` in `db::articles`.
pub async fn list_for_articles(
    pool: &PgPool,
    article_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<EditorialLabelResponse>>, AppError> {
    if article_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as!(
        ArticleLabelRow,
        r#"SELECT ael.article_id, el.id, el.name, el.slug, el.revokes_on_edit
           FROM article_editorial_labels ael
           JOIN editorial_labels el ON el.id = ael.label_id
           WHERE ael.article_id = ANY($1)
           ORDER BY el.sort_order, el.name"#,
        article_ids,
    )
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<Uuid, Vec<EditorialLabelResponse>> = HashMap::new();
    for r in rows {
        map.entry(r.article_id)
            .or_default()
            .push(EditorialLabelResponse {
                id: r.id.to_string(),
                name: r.name,
                slug: r.slug,
                revokes_on_edit: r.revokes_on_edit,
            });
    }
    Ok(map)
}

/// Apply a label to an article. Editor/admin only — auth is enforced at
/// the handler level. Returns the freshly-applied label for response.
/// Requires the article to be `published` — drafts/archived may not carry chips.
pub async fn apply_label(
    pool: &PgPool,
    article_slug: &str,
    label_slug: &str,
    applied_by: Uuid,
) -> Result<EditorialLabelResponse, AppError> {
    let article_id: Uuid = sqlx::query_scalar!(
        r#"SELECT id FROM articles
           WHERE slug = $1 AND status = 'published'"#,
        article_slug,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AppError::BadRequest("Only published articles can carry editorial labels".into())
    })?;

    let label = sqlx::query_as!(
        LabelRow,
        r#"SELECT id, name, slug, revokes_on_edit
           FROM editorial_labels WHERE slug = $1"#,
        label_slug,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Editorial label not found".into()))?;

    sqlx::query!(
        r#"INSERT INTO article_editorial_labels (article_id, label_id, applied_by)
           VALUES ($1, $2, $3)
           ON CONFLICT DO NOTHING"#,
        article_id,
        label.id,
        applied_by,
    )
    .execute(pool)
    .await?;

    Ok(to_response(label))
}

/// Remove a label from an article. Idempotent — succeeds even if the
/// label wasn't applied (the editor saw a stale UI; no need to error).
pub async fn remove_label(
    pool: &PgPool,
    article_slug: &str,
    label_slug: &str,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"DELETE FROM article_editorial_labels ael
           USING articles a, editorial_labels el
           WHERE ael.article_id = a.id
             AND ael.label_id = el.id
             AND a.slug = $1
             AND el.slug = $2"#,
        article_slug,
        label_slug,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Strip every applied label with `revokes_on_edit = true` from this
/// article. Called from the article-update handler when the author
/// modifies `markdown`. Returns the labels that were removed so the
/// frontend can toast.
pub async fn revoke_on_edit(
    pool: &PgPool,
    article_id: Uuid,
) -> Result<Vec<EditorialLabelResponse>, AppError> {
    let removed = sqlx::query_as!(
        LabelRow,
        r#"DELETE FROM article_editorial_labels ael
           USING editorial_labels el
           WHERE ael.label_id = el.id
             AND ael.article_id = $1
             AND el.revokes_on_edit = true
           RETURNING el.id, el.name, el.slug, el.revokes_on_edit"#,
        article_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(removed.into_iter().map(to_response).collect())
}
