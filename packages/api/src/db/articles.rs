use regex::Regex;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::article::{
    ArticleDetailResponse, ArticleLimitsResponse, ArticleResponse, BatchSentenceResponseItem,
    SentenceData, TopicResponse,
};

// ── Constants ─────────────────────────────────────────────

const FREE_MAX_TOTAL: i32 = 10;
const FREE_MAX_PUBLISHED: i32 = 3;

// ── Row types ─────────────────────────────────────────────

struct ArticleRow {
    id: Uuid,
    title: String,
    slug: String,
    description: Option<String>,
    markdown: String,
    html: String,
    status: String,
    author_display_name: String,
    published_at: Option<time::OffsetDateTime>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

struct ArticleSummaryRow {
    id: Uuid,
    title: String,
    slug: String,
    description: Option<String>,
    status: String,
    author_display_name: String,
    published_at: Option<time::OffsetDateTime>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

struct TopicRow {
    id: Uuid,
    name: String,
    slug: String,
}

struct ArticleTopicRow {
    article_id: Uuid,
    topic_id: Uuid,
    topic_name: String,
    topic_slug: String,
}

struct CountRow {
    total: Option<i64>,
    published: Option<i64>,
}

// ── Helpers ───────────────────────────────────────────────

fn fmt_time(t: time::OffsetDateTime) -> String {
    t.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

fn generate_slug(title: &str) -> String {
    slug::slugify(title)
}

fn article_response(r: ArticleSummaryRow, topics: Vec<TopicResponse>) -> ArticleResponse {
    ArticleResponse {
        id: r.id.to_string(),
        title: r.title,
        slug: r.slug,
        description: r.description,
        status: r.status,
        author_display_name: r.author_display_name,
        topics,
        published_at: r.published_at.map(fmt_time),
        created_at: fmt_time(r.created_at),
        updated_at: fmt_time(r.updated_at),
    }
}

fn article_detail_response(r: ArticleRow, topics: Vec<TopicResponse>) -> ArticleDetailResponse {
    ArticleDetailResponse {
        id: r.id.to_string(),
        title: r.title,
        slug: r.slug,
        description: r.description,
        markdown: r.markdown,
        html: r.html,
        status: r.status,
        author_display_name: r.author_display_name,
        topics,
        published_at: r.published_at.map(fmt_time),
        created_at: fmt_time(r.created_at),
        updated_at: fmt_time(r.updated_at),
    }
}

/// Render article markdown to HTML, converting quotation directives to placeholder divs.
pub fn render_article_markdown(markdown: &str) -> String {
    // Pre-process: extract ::quotation{...} directives and replace with placeholders
    let directive_re = Regex::new(r#"::quotation\{([^}]+)\}"#).expect("Invalid directive regex");

    let mut placeholder_map: Vec<String> = Vec::new();
    let processed = directive_re.replace_all(markdown, |caps: &regex::Captures| {
        let attrs_str = &caps[1];
        let idx = placeholder_map.len();

        // Parse key="value" pairs
        let attr_re = Regex::new(r#"(\w+)="([^"]*)""#).expect("Invalid attr regex");
        let mut data_attrs = String::new();
        for attr_cap in attr_re.captures_iter(attrs_str) {
            let key = &attr_cap[1];
            let val = &attr_cap[2];
            data_attrs.push_str(&format!(r#" data-quotation-{key}="{val}""#));
        }

        // Also parse key=number (no quotes)
        let num_re = Regex::new(r#"(\w+)=(\d+)"#).expect("Invalid num regex");
        for num_cap in num_re.captures_iter(attrs_str) {
            let key = &num_cap[1];
            let val = &num_cap[2];
            // Skip if already captured as quoted string
            if !data_attrs.contains(&format!("data-quotation-{key}=")) {
                data_attrs.push_str(&format!(r#" data-quotation-{key}="{val}""#));
            }
        }

        placeholder_map.push(data_attrs);
        format!("\n<!--QUOTATION_PLACEHOLDER_{idx}-->\n")
    });

    // Run pulldown-cmark on the cleaned markdown
    let parser = pulldown_cmark::Parser::new(&processed);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);

    // Post-process: replace placeholder comments with actual divs
    for (idx, data_attrs) in placeholder_map.iter().enumerate() {
        let placeholder = format!("<!--QUOTATION_PLACEHOLDER_{idx}-->");
        let replacement = format!(r#"<div class="quotation-embed"{data_attrs}></div>"#);
        html_output = html_output.replace(&placeholder, &replacement);
    }

    html_output
}

// ── Article queries ───────────────────────────────────────

pub async fn create_article(
    pool: &PgPool,
    user_id: Uuid,
    title: &str,
) -> Result<ArticleDetailResponse, AppError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::BadRequest("Title cannot be empty".into()));
    }

    let base_slug = generate_slug(title);

    // Try the base slug first, then with random suffix on collision
    let mut slug = base_slug.clone();
    let mut attempts = 0;
    let row = loop {
        let result = sqlx::query_as!(
            ArticleRow,
            r#"INSERT INTO articles (user_id, title, slug)
               VALUES ($1, $2, $3)
               RETURNING
                   id, title, slug, description, markdown, html,
                   status::TEXT AS "status!",
                   (SELECT display_name FROM users WHERE id = $1) AS "author_display_name!",
                   published_at, created_at, updated_at"#,
            user_id,
            title,
            slug,
        )
        .fetch_one(pool)
        .await;

        match result {
            Ok(row) => break row,
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() && attempts < 5 => {
                attempts += 1;
                let suffix: u32 = rand::random::<u32>() % 999999;
                slug = format!("{base_slug}-{suffix:06}");
            }
            Err(e) => return Err(e.into()),
        }
    };

    Ok(article_detail_response(row, vec![]))
}

pub async fn get_user_article_by_slug(
    pool: &PgPool,
    slug: &str,
    user_id: Uuid,
) -> Result<ArticleDetailResponse, AppError> {
    let row = sqlx::query_as!(
        ArticleRow,
        r#"SELECT a.id, a.title, a.slug, a.description, a.markdown, a.html,
                  a.status::TEXT AS "status!",
                  u.display_name AS "author_display_name!",
                  a.published_at, a.created_at, a.updated_at
           FROM articles a
           JOIN users u ON u.id = a.user_id
           WHERE a.slug = $1 AND a.user_id = $2"#,
        slug,
        user_id,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Article not found".into()))?;

    let topics = load_article_topics(pool, row.id).await?;
    Ok(article_detail_response(row, topics))
}

pub async fn get_published_article_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<ArticleDetailResponse, AppError> {
    let row = sqlx::query_as!(
        ArticleRow,
        r#"SELECT a.id, a.title, a.slug, a.description, a.markdown, a.html,
                  a.status::TEXT AS "status!",
                  u.display_name AS "author_display_name!",
                  a.published_at, a.created_at, a.updated_at
           FROM articles a
           JOIN users u ON u.id = a.user_id
           WHERE a.slug = $1 AND a.status = 'published'"#,
        slug,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Article not found".into()))?;

    let topics = load_article_topics(pool, row.id).await?;
    Ok(article_detail_response(row, topics))
}

pub async fn list_user_articles(
    pool: &PgPool,
    user_id: Uuid,
    status_filter: Option<&str>,
) -> Result<Vec<ArticleResponse>, AppError> {
    let rows = sqlx::query_as!(
        ArticleSummaryRow,
        r#"SELECT a.id, a.title, a.slug, a.description,
                  a.status::TEXT AS "status!",
                  u.display_name AS "author_display_name!",
                  a.published_at, a.created_at, a.updated_at
           FROM articles a
           JOIN users u ON u.id = a.user_id
           WHERE a.user_id = $1
             AND ($2::TEXT IS NULL OR a.status::TEXT = $2)
           ORDER BY a.updated_at DESC"#,
        user_id,
        status_filter,
    )
    .fetch_all(pool)
    .await?;

    let article_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
    let topics_map = load_articles_topics(pool, &article_ids).await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let id = r.id;
            article_response(r, topics_map.get(&id).cloned().unwrap_or_default())
        })
        .collect())
}

pub async fn list_published_articles(
    pool: &PgPool,
    topic_slug: Option<&str>,
    page: i32,
    per_page: i32,
) -> Result<(Vec<ArticleResponse>, i64), AppError> {
    let offset = (page - 1).max(0) as i64 * per_page as i64;
    let limit = per_page as i64;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM articles a
           WHERE a.status = 'published'
             AND ($1::TEXT IS NULL OR EXISTS (
                 SELECT 1 FROM article_topics at2
                 JOIN topics t ON t.id = at2.topic_id
                 WHERE at2.article_id = a.id AND t.slug = $1
             ))"#,
        topic_slug,
    )
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query_as!(
        ArticleSummaryRow,
        r#"SELECT a.id, a.title, a.slug, a.description,
                  a.status::TEXT AS "status!",
                  u.display_name AS "author_display_name!",
                  a.published_at, a.created_at, a.updated_at
           FROM articles a
           JOIN users u ON u.id = a.user_id
           WHERE a.status = 'published'
             AND ($1::TEXT IS NULL OR EXISTS (
                 SELECT 1 FROM article_topics at2
                 JOIN topics t ON t.id = at2.topic_id
                 WHERE at2.article_id = a.id AND t.slug = $1
             ))
           ORDER BY a.published_at DESC NULLS LAST
           LIMIT $2 OFFSET $3"#,
        topic_slug,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let article_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
    let topics_map = load_articles_topics(pool, &article_ids).await?;

    let articles = rows
        .into_iter()
        .map(|r| {
            let id = r.id;
            article_response(r, topics_map.get(&id).cloned().unwrap_or_default())
        })
        .collect();

    Ok((articles, total))
}

pub async fn update_article(
    pool: &PgPool,
    slug: &str,
    user_id: Uuid,
    title: Option<&str>,
    markdown: Option<&str>,
    description: Option<&str>,
    topic_ids: Option<&[String]>,
) -> Result<ArticleDetailResponse, AppError> {
    // Fetch article and verify ownership
    let article_id = sqlx::query_scalar!(
        r#"SELECT id FROM articles WHERE slug = $1 AND user_id = $2"#,
        slug,
        user_id,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Article not found".into()))?;

    // Update title and regenerate slug if title changed
    if let Some(title) = title {
        let title = title.trim();
        if title.is_empty() {
            return Err(AppError::BadRequest("Title cannot be empty".into()));
        }
        let new_slug = generate_slug(title);

        // Try base slug, then with suffix on collision
        let mut final_slug = new_slug.clone();
        let mut attempts = 0;
        loop {
            let result = sqlx::query!(
                r#"UPDATE articles SET title = $2, slug = $3, updated_at = now()
                   WHERE id = $1"#,
                article_id,
                title,
                final_slug,
            )
            .execute(pool)
            .await;

            match result {
                Ok(_) => break,
                Err(sqlx::Error::Database(e)) if e.is_unique_violation() && attempts < 5 => {
                    attempts += 1;
                    let suffix: u32 = rand::random::<u32>() % 999999;
                    final_slug = format!("{new_slug}-{suffix:06}");
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    // Update markdown and re-render HTML
    if let Some(md) = markdown {
        let html = render_article_markdown(md);
        sqlx::query!(
            r#"UPDATE articles SET markdown = $2, html = $3, updated_at = now()
               WHERE id = $1"#,
            article_id,
            md,
            html,
        )
        .execute(pool)
        .await?;
    }

    // Update description
    if let Some(desc) = description {
        if desc.len() > 250 {
            return Err(AppError::BadRequest(
                "Description must be 250 characters or fewer".into(),
            ));
        }
        sqlx::query!(
            r#"UPDATE articles SET description = $2, updated_at = now()
               WHERE id = $1"#,
            article_id,
            desc,
        )
        .execute(pool)
        .await?;
    }

    // Update topics
    if let Some(ids) = topic_ids {
        set_article_topics(pool, article_id, ids).await?;
    }

    // Return updated article
    let new_slug = sqlx::query_scalar!(r#"SELECT slug FROM articles WHERE id = $1"#, article_id,)
        .fetch_one(pool)
        .await?;

    get_user_article_by_slug(pool, &new_slug, user_id).await
}

pub async fn publish_article(pool: &PgPool, slug: &str, user_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"UPDATE articles
           SET status = 'published',
               published_at = COALESCE(published_at, now()),
               updated_at = now()
           WHERE slug = $1 AND user_id = $2 AND status != 'archived'"#,
        slug,
        user_id,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Article not found".into()));
    }
    Ok(())
}

pub async fn unpublish_article(pool: &PgPool, slug: &str, user_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"UPDATE articles SET status = 'draft', updated_at = now()
           WHERE slug = $1 AND user_id = $2 AND status = 'published'"#,
        slug,
        user_id,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Article not found".into()));
    }
    Ok(())
}

pub async fn archive_article(pool: &PgPool, slug: &str, user_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"UPDATE articles SET status = 'archived', updated_at = now()
           WHERE slug = $1 AND user_id = $2"#,
        slug,
        user_id,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Article not found".into()));
    }
    Ok(())
}

pub async fn get_user_article_counts(pool: &PgPool, user_id: Uuid) -> Result<(i64, i64), AppError> {
    let row = sqlx::query_as!(
        CountRow,
        r#"SELECT
               COUNT(*) FILTER (WHERE status != 'archived') AS "total?",
               COUNT(*) FILTER (WHERE status = 'published') AS "published?"
           FROM articles
           WHERE user_id = $1"#,
        user_id,
    )
    .fetch_one(pool)
    .await?;

    Ok((row.total.unwrap_or(0), row.published.unwrap_or(0)))
}

pub fn get_article_limits(_roles: &[String]) -> (i32, i32) {
    // For now, all users get free tier limits.
    // When subscriber role is added, check roles and return higher limits.
    (FREE_MAX_TOTAL, FREE_MAX_PUBLISHED)
}

pub async fn get_article_limits_response(
    pool: &PgPool,
    user_id: Uuid,
    roles: &[String],
) -> Result<ArticleLimitsResponse, AppError> {
    let (current_total, current_published) = get_user_article_counts(pool, user_id).await?;
    let (max_total, max_published) = get_article_limits(roles);
    Ok(ArticleLimitsResponse {
        max_total,
        max_published,
        current_total,
        current_published,
    })
}

// ── Topic queries ─────────────────────────────────────────

pub async fn list_topics(pool: &PgPool) -> Result<Vec<TopicResponse>, AppError> {
    let rows = sqlx::query_as!(
        TopicRow,
        r#"SELECT id, name, slug FROM topics ORDER BY name"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| TopicResponse {
            id: r.id.to_string(),
            name: r.name,
            slug: r.slug,
        })
        .collect())
}

async fn load_article_topics(
    pool: &PgPool,
    article_id: Uuid,
) -> Result<Vec<TopicResponse>, AppError> {
    let rows = sqlx::query_as!(
        TopicRow,
        r#"SELECT t.id, t.name, t.slug
           FROM topics t
           JOIN article_topics at2 ON at2.topic_id = t.id
           WHERE at2.article_id = $1
           ORDER BY t.name"#,
        article_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| TopicResponse {
            id: r.id.to_string(),
            name: r.name,
            slug: r.slug,
        })
        .collect())
}

async fn load_articles_topics(
    pool: &PgPool,
    article_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<TopicResponse>>, AppError> {
    if article_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as!(
        ArticleTopicRow,
        r#"SELECT at2.article_id, t.id AS topic_id, t.name AS topic_name, t.slug AS topic_slug
           FROM article_topics at2
           JOIN topics t ON t.id = at2.topic_id
           WHERE at2.article_id = ANY($1)
           ORDER BY t.name"#,
        article_ids,
    )
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<Uuid, Vec<TopicResponse>> = HashMap::new();
    for r in rows {
        map.entry(r.article_id).or_default().push(TopicResponse {
            id: r.topic_id.to_string(),
            name: r.topic_name,
            slug: r.topic_slug,
        });
    }

    Ok(map)
}

async fn set_article_topics(
    pool: &PgPool,
    article_id: Uuid,
    topic_ids: &[String],
) -> Result<(), AppError> {
    if topic_ids.len() > 5 {
        return Err(AppError::BadRequest("Maximum 5 topics per article".into()));
    }

    // Clear existing
    sqlx::query!(
        r#"DELETE FROM article_topics WHERE article_id = $1"#,
        article_id,
    )
    .execute(pool)
    .await?;

    // Insert new
    for id_str in topic_ids {
        let topic_id = Uuid::parse_str(id_str)
            .map_err(|_| AppError::BadRequest(format!("Invalid topic ID: {id_str}")))?;

        sqlx::query!(
            r#"INSERT INTO article_topics (article_id, topic_id) VALUES ($1, $2)
               ON CONFLICT DO NOTHING"#,
            article_id,
            topic_id,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

// ── Batch sentence queries for quotation card hydration ───

struct SentenceRow {
    sentence_number: Option<i32>,
    html: String,
    original_html: Option<String>,
}

pub async fn batch_get_sentences(
    pool: &PgPool,
    book_slug: &str,
    node_slug: &str,
    start_number: i32,
    end_number: Option<i32>,
    kind: &str,
) -> Result<BatchSentenceResponseItem, AppError> {
    let end = end_number.unwrap_or(start_number);
    let is_body = kind == "body";

    struct BookNodeRow {
        book_title: String,
        node_label: String,
    }

    let context = sqlx::query_as!(
        BookNodeRow,
        r#"SELECT b.title AS "book_title!", n.label AS "node_label!"
           FROM books b
           JOIN toc_nodes n ON n.book_id = b.id AND n.slug = $2
           WHERE b.slug = $1"#,
        book_slug,
        node_slug,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Book or node not found".into()))?;

    let rows = if is_body {
        sqlx::query_as!(
            SentenceRow,
            r#"SELECT s.sentence_number AS "sentence_number?", s.html AS "html!", s.original_html
               FROM sentences s
               JOIN books b ON b.id = s.book_id
               WHERE b.slug = $1
                 AND s.sentence_number >= $2
                 AND s.sentence_number <= $3
                 AND s.block_id IS NOT NULL
               ORDER BY s.sentence_number"#,
            book_slug,
            start_number,
            end,
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            SentenceRow,
            r#"SELECT s.sentence_number AS "sentence_number?", s.html AS "html!", s.original_html
               FROM sentences s
               JOIN books b ON b.id = s.book_id
               WHERE b.slug = $1
                 AND s.sentence_number >= $2
                 AND s.sentence_number <= $3
                 AND s.footnote_id IS NOT NULL
               ORDER BY s.sentence_number"#,
            book_slug,
            start_number,
            end,
        )
        .fetch_all(pool)
        .await?
    };

    Ok(BatchSentenceResponseItem {
        book_slug: book_slug.to_string(),
        book_title: context.book_title,
        node_slug: node_slug.to_string(),
        node_label: context.node_label,
        sentences: rows
            .into_iter()
            .map(|r| SentenceData {
                sentence_number: r.sentence_number.unwrap_or(0),
                html: r.html,
                original_html: r.original_html,
            })
            .collect(),
    })
}
