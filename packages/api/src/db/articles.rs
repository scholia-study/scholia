use regex::Regex;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::auth::permissions::{resolve_permissions, Permission};
use crate::error::AppError;
use crate::models::article::{
    ArticleDetailResponse, ArticleLimitsResponse, ArticleResponse, BatchSentenceResponseItem,
    SentenceData, SourceContext, TopicResponse,
};

// ── Tier limits ──────────────────────────────────────────
// Free tier defaults (applied when user lacks elevated permissions).
const FREE_ARTICLES_ACTIVE: i32 = 5;
const FREE_ARTICLES_ARCHIVE: i32 = 10;
// Paid / staff tier (granted by ArticlesLimit1000 / ArticlesArchiveLimit1000).
const PAID_ARTICLES_ACTIVE: i32 = 1000;
const PAID_ARTICLES_ARCHIVE: i32 = 1000;

// ── Validation helpers ────────────────────────────────────

/// Reject any input containing emoji or other extended pictographs.
/// Articles are meant for serious study; emoji are out of scope.
fn reject_emoji(field: &str, value: &str) -> Result<(), AppError> {
    static EMOJI_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = EMOJI_RE
        .get_or_init(|| Regex::new(r"\p{Extended_Pictographic}").expect("emoji regex"));
    if re.is_match(value) {
        return Err(AppError::BadRequest(format!(
            "Emoji and pictographic characters are not allowed in the article {field}."
        )));
    }
    Ok(())
}

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
    active: Option<i64>,
    archived: Option<i64>,
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
pub async fn render_article_markdown(pool: &PgPool, markdown: &str) -> String {
    // Pre-process: extract ::quotation{...} directives and replace with placeholders
    let directive_re = Regex::new(r#"::quotation\{([^}]+)\}"#).expect("Invalid directive regex");

    let mut placeholder_map: Vec<String> = Vec::new();
    let mut quotation_book_slugs: Vec<String> = Vec::new();
    let processed = directive_re.replace_all(markdown, |caps: &regex::Captures| {
        let attrs_str = &caps[1];
        let idx = placeholder_map.len();

        // Extract book slug for bibliography
        let book_re = Regex::new(r#"book="([^"]*)""#).expect("Invalid book regex");
        if let Some(book_cap) = book_re.captures(attrs_str) {
            let slug = book_cap[1].to_string();
            if !quotation_book_slugs.contains(&slug) {
                quotation_book_slugs.push(slug);
            }
        }

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

    // Pre-process: extract ::article-quotation{...} directives.
    // mdast-util-directive serializes `id="xxx"` as the shorthand `#xxx`,
    // so accept both forms.
    let article_q_re =
        Regex::new(r#"::article-quotation\{([^}]+)\}"#).expect("Invalid article-quotation regex");
    let id_shorthand_re =
        Regex::new(r#"#([^\s}]+)"#).expect("Invalid id shorthand regex");
    let mut article_q_placeholder_map: Vec<String> = Vec::new();
    let processed = article_q_re.replace_all(&processed, |caps: &regex::Captures| {
        let attrs_str = &caps[1];
        let idx = article_q_placeholder_map.len();

        let attr_re = Regex::new(r#"(\w+)="([^"]*)""#).expect("Invalid attr regex");
        let mut data_attrs = String::new();
        for attr_cap in attr_re.captures_iter(attrs_str) {
            let key = &attr_cap[1];
            let val = &attr_cap[2];
            data_attrs.push_str(&format!(r#" data-article-quotation-{key}="{val}""#));
        }
        if !data_attrs.contains("data-article-quotation-id=") {
            if let Some(id_cap) = id_shorthand_re.captures(attrs_str) {
                let val = &id_cap[1];
                data_attrs
                    .push_str(&format!(r#" data-article-quotation-id="{val}""#));
            }
        }

        article_q_placeholder_map.push(data_attrs);
        format!("\n<!--ARTICLE_QUOTATION_PLACEHOLDER_{idx}-->\n")
    });

    // Pre-process: extract :cite{sources="..."} directives
    let cite_re = Regex::new(r#":cite\{[^}]*?sources="([^"]+)"[^}]*?\}"#).expect("Invalid cite regex");

    // Collect all citation entries: Vec<(placeholder_index, Vec<(source_id, pages)>)>
    let mut citation_map: Vec<Vec<(String, String)>> = Vec::new();
    let mut all_source_ids: Vec<Uuid> = Vec::new();

    let processed = cite_re.replace_all(&processed, |caps: &regex::Captures| {
        let sources_str = &caps[1];
        let idx = citation_map.len();
        let mut entries = Vec::new();

        for entry in sources_str.split(',') {
            let parts: Vec<&str> = entry.splitn(2, ':').collect();
            let source_id = parts[0].trim().to_string();
            let pages = parts.get(1).unwrap_or(&"").trim().to_string();
            if let Ok(uuid) = Uuid::parse_str(&source_id) {
                all_source_ids.push(uuid);
            }
            entries.push((source_id, pages));
        }

        citation_map.push(entries);
        format!("<!--CITATION_PLACEHOLDER_{idx}-->")
    });

    // Look up source IDs for quotation book slugs
    if !quotation_book_slugs.is_empty() {
        // Get source IDs for quoted books, plus their originals (for translations)
        let rows: Vec<(Uuid, Option<Uuid>)> = sqlx::query_as(
            "SELECT s.id, s.translation_of_id
             FROM books b
             JOIN sources s ON s.id = b.source_id
             WHERE b.slug = ANY($1)",
        )
        .bind(&quotation_book_slugs)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (source_id, translation_of_id) in rows {
            if !all_source_ids.contains(&source_id) {
                all_source_ids.push(source_id);
            }
            if let Some(orig_id) = translation_of_id {
                if !all_source_ids.contains(&orig_id) {
                    all_source_ids.push(orig_id);
                }
            }
        }
    }
    // Batch fetch source + person data for all citations and quotations
    let source_data = if !all_source_ids.is_empty() {
        fetch_citation_data(pool, &all_source_ids).await
    } else {
        HashMap::new()
    };

    // Run pulldown-cmark on the cleaned markdown
    let parser = pulldown_cmark::Parser::new(&processed);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);

    // Post-process: replace quotation placeholder comments with actual divs
    for (idx, data_attrs) in placeholder_map.iter().enumerate() {
        let placeholder = format!("<!--QUOTATION_PLACEHOLDER_{idx}-->");
        let replacement = format!(r#"<div class="quotation-embed"{data_attrs}></div>"#);
        html_output = html_output.replace(&placeholder, &replacement);
    }

    // Post-process: replace article quotation placeholder comments with actual divs
    for (idx, data_attrs) in article_q_placeholder_map.iter().enumerate() {
        let placeholder = format!("<!--ARTICLE_QUOTATION_PLACEHOLDER_{idx}-->");
        let replacement =
            format!(r#"<div class="article-quotation-embed"{data_attrs}></div>"#);
        html_output = html_output.replace(&placeholder, &replacement);
    }

    // Post-process: replace citation placeholders with inline spans
    // Seed bibliography with all collected source IDs (quotations + citations)
    let mut bibliography_sources: Vec<Uuid> = all_source_ids
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    for (idx, entries) in citation_map.iter().enumerate() {
        let placeholder = format!("<!--CITATION_PLACEHOLDER_{idx}-->");
        let inline_text = format_inline_citation(entries, &source_data);
        let replacement = format!(r#"<span class="citation">{inline_text}</span>"#);
        html_output = html_output.replace(&placeholder, &replacement);

        // Track unique sources for bibliography
        for (id_str, _) in entries {
            if let Ok(uuid) = Uuid::parse_str(id_str) {
                if !bibliography_sources.contains(&uuid) {
                    bibliography_sources.push(uuid);
                }
            }
        }
    }

    // Append bibliography if any citations exist
    if !bibliography_sources.is_empty() {
        html_output.push_str("\n<section class=\"bibliography\">\n<h2>Bibliography</h2>\n<ul style=\"list-style:none;padding:0;margin:0\">\n");
        // Sort bibliography by author last name
        let mut bib_entries: Vec<String> = bibliography_sources
            .iter()
            .filter_map(|id| {
                source_data.get(id).map(|data| format_bibliography_entry(data))
            })
            .collect();
        bib_entries.sort();
        for entry in bib_entries {
            html_output.push_str(&format!("<li style=\"margin:0.25em 0\">{entry}</li>\n"));
        }
        html_output.push_str("</ul>\n</section>\n");
    }

    html_output
}

// ── Citation helpers ─────────────────────────────────────

struct CitationSourceData {
    title: String,
    publication_year: Option<i16>,
    publisher: Option<String>,
    authors: Vec<String>,         // sorted by position
    author_sort_names: Vec<String>,
}

async fn fetch_citation_data(
    pool: &PgPool,
    source_ids: &[Uuid],
) -> HashMap<Uuid, CitationSourceData> {
    let mut map = HashMap::new();

    struct SourceRow {
        id: Uuid,
        title: String,
        publication_year: Option<i16>,
        publisher: Option<String>,
    }

    let sources: Vec<SourceRow> = sqlx::query_as!(
        SourceRow,
        r#"SELECT id, title, publication_year, publisher
           FROM sources WHERE id = ANY($1)"#,
        source_ids,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for s in &sources {
        map.insert(s.id, CitationSourceData {
            title: s.title.clone(),
            publication_year: s.publication_year,
            publisher: s.publisher.clone(),
            authors: Vec::new(),
            author_sort_names: Vec::new(),
        });
    }

    struct PersonRow {
        source_id: Uuid,
        name: String,
        sort_name: Option<String>,
    }

    let persons: Vec<PersonRow> = sqlx::query_as!(
        PersonRow,
        r#"SELECT sp.source_id, p.name, p.sort_name
           FROM source_persons sp
           JOIN persons p ON p.id = sp.person_id
           WHERE sp.source_id = ANY($1) AND sp.role = 'author'
           ORDER BY sp.position"#,
        source_ids,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for p in persons {
        if let Some(data) = map.get_mut(&p.source_id) {
            data.authors.push(p.name.clone());
            data.author_sort_names.push(
                p.sort_name.unwrap_or_else(|| {
                    // Derive last name from full name
                    p.name.split_whitespace().last().unwrap_or(&p.name).to_string()
                }),
            );
        }
    }

    map
}

/// Format inline citation: (LastName Year, Pages; ...)
fn format_inline_citation(
    entries: &[(String, String)],
    source_data: &HashMap<Uuid, CitationSourceData>,
) -> String {
    let parts: Vec<String> = entries
        .iter()
        .map(|(id_str, pages)| {
            let uuid = Uuid::parse_str(id_str).ok();
            let data = uuid.and_then(|id| source_data.get(&id));

            let author_part = match data {
                Some(d) if d.authors.is_empty() => "Unknown".to_string(),
                Some(d) if d.authors.len() == 1 => last_name(&d.authors[0]),
                Some(d) if d.authors.len() == 2 => {
                    format!("{} and {}", last_name(&d.authors[0]), last_name(&d.authors[1]))
                }
                Some(d) => format!("{} et al.", last_name(&d.authors[0])),
                None => "Unknown".to_string(),
            };

            let year = data
                .and_then(|d| d.publication_year)
                .map(|y| y.to_string())
                .unwrap_or_else(|| "n.d.".to_string());

            if pages.is_empty() {
                format!("{author_part} {year}")
            } else {
                format!("{author_part} {year}, {pages}")
            }
        })
        .collect();

    format!("({})", parts.join("; "))
}

/// Format a bibliography entry in Chicago author-date style
fn format_bibliography_entry(data: &CitationSourceData) -> String {
    // Author(s)
    let author_part = if data.author_sort_names.is_empty() {
        "Unknown".to_string()
    } else if data.author_sort_names.len() == 1 {
        // "Last, First"
        data.author_sort_names[0].clone()
    } else {
        // "Last, First, and First Last"
        let first = &data.author_sort_names[0];
        let rest: Vec<&str> = data.authors[1..].iter().map(|s| s.as_str()).collect();
        if rest.len() == 1 {
            format!("{first}, and {}", rest[0])
        } else {
            format!("{first}, {}, and {}", rest[..rest.len() - 1].join(", "), rest.last().unwrap())
        }
    };

    // Year
    let year = data
        .publication_year
        .map(|y| y.to_string())
        .unwrap_or_else(|| "n.d.".to_string());

    // Title (italicized)
    let title = &data.title;

    // Publisher
    let publisher = data.publisher.as_deref().unwrap_or("");

    if publisher.is_empty() {
        format!("{author_part}. {year}. <em>{title}</em>.")
    } else {
        format!("{author_part}. {year}. <em>{title}</em>. {publisher}.")
    }
}

/// Extract last name from a full name
fn last_name(name: &str) -> String {
    name.split_whitespace().last().unwrap_or(name).to_string()
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
    reject_emoji("title", title)?;

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
           WHERE a.slug = $1 AND a.status IN ('published', 'archived')"#,
        slug,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Article not found".into()))?;

    let topics = load_article_topics(pool, row.id).await?;
    Ok(article_detail_response(row, topics))
}

pub async fn get_article_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<ArticleDetailResponse, AppError> {
    let row = sqlx::query_as!(
        ArticleRow,
        r#"SELECT a.id, a.title, a.slug, a.description, a.markdown, a.html,
                  a.status::TEXT AS "status!",
                  u.display_name AS "author_display_name!",
                  a.published_at, a.created_at, a.updated_at
           FROM articles a
           JOIN users u ON u.id = a.user_id
           WHERE a.id = $1 AND a.status IN ('published', 'archived')"#,
        id,
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
    roles: &[String],
    title: Option<&str>,
    markdown: Option<&str>,
    description: Option<&str>,
    topic_ids: Option<&[String]>,
) -> Result<ArticleDetailResponse, AppError> {
    // Fetch article and verify ownership
    let row = sqlx::query!(
        r#"SELECT id, status AS "status: String" FROM articles
           WHERE slug = $1 AND user_id = $2"#,
        slug,
        user_id,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Article not found".into()))?;

    if row.status == "archived" {
        return Err(AppError::BadRequest(
            "Archived articles cannot be edited".into(),
        ));
    }
    let article_id = row.id;

    if !user_can_edit_article(pool, user_id, article_id, roles).await? {
        return Err(AppError::Forbidden(
            "Upgrade your account to edit more articles".into(),
        ));
    }

    // Update title and regenerate slug if title changed
    if let Some(title) = title {
        let title = title.trim();
        if title.is_empty() {
            return Err(AppError::BadRequest("Title cannot be empty".into()));
        }
        reject_emoji("title", title)?;
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
        reject_emoji("article body", md)?;
        let html = render_article_markdown(pool, md).await;
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
        reject_emoji("description", desc)?;
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

pub async fn publish_article(
    pool: &PgPool,
    slug: &str,
    user_id: Uuid,
    roles: &[String],
) -> Result<(), AppError> {
    let row = sqlx::query!(
        r#"SELECT id, status AS "status: String" FROM articles
           WHERE slug = $1 AND user_id = $2"#,
        slug,
        user_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Article not found".into()))?;

    if row.status != "draft" {
        return Err(AppError::BadRequest(
            "Article is not in draft status".into(),
        ));
    }

    if !user_can_edit_article(pool, user_id, row.id, roles).await? {
        return Err(AppError::Forbidden(
            "Upgrade your account to edit more articles".into(),
        ));
    }

    sqlx::query!(
        r#"UPDATE articles
           SET status = 'published',
               published_at = COALESCE(published_at, now()),
               updated_at = now()
           WHERE id = $1"#,
        row.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn archive_article(pool: &PgPool, slug: &str, user_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"UPDATE articles SET status = 'archived', updated_at = now()
           WHERE slug = $1 AND user_id = $2 AND status = 'published'"#,
        slug,
        user_id,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Article not found or not in published status".into(),
        ));
    }
    Ok(())
}

/// Returns the IDs of the N oldest non-archived articles for a user,
/// ordered by `created_at ASC`. Used to determine which articles remain
/// editable for users without elevated article-limit permissions.
pub async fn list_oldest_active_article_ids(
    pool: &PgPool,
    user_id: Uuid,
    limit: i32,
) -> Result<Vec<Uuid>, AppError> {
    let rows = sqlx::query_scalar!(
        r#"SELECT id FROM articles
           WHERE user_id = $1 AND status != 'archived'
           ORDER BY created_at ASC
           LIMIT $2"#,
        user_id,
        limit as i64,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Returns true if the user may edit the given article given their roles.
/// Users with ArticlesLimit1000 may edit any non-archived article they own;
/// otherwise only their oldest FREE_ARTICLES_ACTIVE active articles are editable.
pub async fn user_can_edit_article(
    pool: &PgPool,
    user_id: Uuid,
    article_id: Uuid,
    roles: &[String],
) -> Result<bool, AppError> {
    if resolve_permissions(roles).contains(&Permission::ArticlesLimit1000) {
        return Ok(true);
    }
    let editable = list_oldest_active_article_ids(pool, user_id, FREE_ARTICLES_ACTIVE).await?;
    Ok(editable.contains(&article_id))
}

/// Returns (current_active, current_archived) counts.
pub async fn get_user_article_counts(pool: &PgPool, user_id: Uuid) -> Result<(i64, i64), AppError> {
    let row = sqlx::query_as!(
        CountRow,
        r#"SELECT
               COUNT(*) FILTER (WHERE status != 'archived') AS "active?",
               COUNT(*) FILTER (WHERE status = 'archived')  AS "archived?"
           FROM articles
           WHERE user_id = $1"#,
        user_id,
    )
    .fetch_one(pool)
    .await?;

    Ok((row.active.unwrap_or(0), row.archived.unwrap_or(0)))
}

/// Derive article limits from the user's resolved permissions.
pub fn get_article_limits(roles: &[String]) -> (i32, i32) {
    let perms = resolve_permissions(roles);
    let max_active = if perms.contains(&Permission::ArticlesLimit1000) {
        PAID_ARTICLES_ACTIVE
    } else {
        FREE_ARTICLES_ACTIVE
    };
    let max_archive = if perms.contains(&Permission::ArticlesArchiveLimit1000) {
        PAID_ARTICLES_ARCHIVE
    } else {
        FREE_ARTICLES_ARCHIVE
    };
    (max_active, max_archive)
}

pub async fn get_article_limits_response(
    pool: &PgPool,
    user_id: Uuid,
    roles: &[String],
) -> Result<ArticleLimitsResponse, AppError> {
    let (current_active, current_archive) = get_user_article_counts(pool, user_id).await?;
    let (max_active, max_archive) = get_article_limits(roles);
    Ok(ArticleLimitsResponse {
        max_active,
        current_active,
        max_archive,
        current_archive,
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
        r#"SELECT COALESCE(s.title_display, s.title) AS "book_title!", n.label AS "node_label!"
           FROM books b
           JOIN sources s ON s.id = b.source_id
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
            r#"SELECT s.sentence_number AS "sentence_number?",
                      s.html AS "html!",
                      COALESCE(s.original_html, src.html) AS original_html
               FROM sentences s
               JOIN books b ON b.id = s.book_id
               LEFT JOIN sentences src ON src.id = s.source_sentence_start_id
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
            r#"SELECT s.sentence_number AS "sentence_number?",
                      s.html AS "html!",
                      COALESCE(s.original_html, src.html) AS original_html
               FROM sentences s
               JOIN books b ON b.id = s.book_id
               LEFT JOIN sentences src ON src.id = s.source_sentence_start_id
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

    // Fetch source book/node context if sentences link to a source
    struct SourceRow {
        book_slug: String,
        book_title: String,
        node_slug: String,
        node_label: String,
    }
    let source_context = sqlx::query_as!(
        SourceRow,
        r#"SELECT b.slug AS "book_slug!", COALESCE(bs.title_display, bs.title) AS "book_title!",
                  n.slug AS "node_slug!", n.label AS "node_label!"
           FROM sentences s
           JOIN books cur ON cur.id = s.book_id
           JOIN sentences src ON src.id = s.source_sentence_start_id
           JOIN books b ON b.id = src.book_id
           JOIN sources bs ON bs.id = b.source_id
           JOIN toc_nodes n ON n.id = src.node_id
           WHERE cur.slug = $1
             AND s.sentence_number >= $2
             AND s.sentence_number <= $3
           LIMIT 1"#,
        book_slug,
        start_number,
        end,
    )
    .fetch_optional(pool)
    .await?
    .map(|r| SourceContext {
        book_slug: r.book_slug,
        book_title: r.book_title,
        node_slug: r.node_slug,
        node_label: r.node_label,
    });

    Ok(BatchSentenceResponseItem {
        book_slug: book_slug.to_string(),
        book_title: context.book_title,
        node_slug: node_slug.to_string(),
        node_label: context.node_label,
        source: source_context,
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
