use std::sync::OnceLock;

use regex::Regex;
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::writing::article_quotations::models::ArticleQuotationResponse;
use crate::system::error::AppError;

/// Normalize text for lenient quote-containment matching: drop HTML tags and
/// entity references, keep only alphanumerics, lowercase. A faithful
/// quotation is extracted client-side from the article's rendered text, so
/// its words are verbatim from the source `html`; this normalization lets it
/// match regardless of whitespace, paragraph boundaries, or entity encoding,
/// while fabricated text (different words) fails to match.
fn normalize_for_containment(s: &str, strip_html: bool) -> String {
    static TAG_RE: OnceLock<Regex> = OnceLock::new();
    static ENTITY_RE: OnceLock<Regex> = OnceLock::new();
    let cleaned = if strip_html {
        let tag_re = TAG_RE.get_or_init(|| Regex::new(r"<[^>]*>").unwrap());
        let entity_re = ENTITY_RE.get_or_init(|| Regex::new(r"&[a-zA-Z0-9#]+;").unwrap());
        let no_tags = tag_re.replace_all(s, " ");
        entity_re.replace_all(&no_tags, " ").into_owned()
    } else {
        s.to_string()
    };
    cleaned
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Whether `quote_text` faithfully occurs in the cited article's rendered
/// `html`. Guards against attributing fabricated text to another author.
fn quote_text_occurs_in_html(article_html: &str, quote_text: &str) -> bool {
    let haystack = normalize_for_containment(article_html, true);
    let needle = normalize_for_containment(quote_text, false);
    haystack.contains(&needle)
}

struct ArticleQuotationRow {
    id: Uuid,
    article_id: Option<Uuid>,
    article_title: String,
    author_display_name: String,
    text: String,
    html: String,
    note_count: Option<i64>,
    created_at: time::OffsetDateTime,
}

fn fmt_time(t: time::OffsetDateTime) -> String {
    t.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

fn article_quotation_from_row(r: ArticleQuotationRow) -> ArticleQuotationResponse {
    ArticleQuotationResponse {
        id: r.id.to_string(),
        article_id: r.article_id.map(|id| id.to_string()),
        article_title: r.article_title,
        author_display_name: r.author_display_name,
        text: r.text,
        html: r.html,
        note_count: r.note_count.unwrap_or(0),
        created_at: fmt_time(r.created_at),
    }
}

pub async fn create_article_quotation(
    pool: &PgPool,
    user_id: Uuid,
    article_id: Uuid,
    text: &str,
    html: &str,
) -> Result<(ArticleQuotationResponse, bool), AppError> {
    // App-level dedup: check if same user already saved same text from same article
    let existing = sqlx::query_scalar!(
        r#"SELECT id FROM article_quotations
           WHERE user_id = $1 AND article_id = $2 AND text = $3"#,
        user_id,
        article_id,
        text,
    )
    .fetch_optional(pool)
    .await?;

    if let Some(existing_id) = existing {
        let row = fetch_article_quotation_row(pool, existing_id).await?;
        return Ok((article_quotation_from_row(row), false));
    }

    // Fetch article metadata for snapshot. We freeze title, author display
    // name, author sort name, and the article's published_at — these are
    // the fields the bibliography renderer needs and that may drift on the
    // source side after the quotation is saved.
    struct ArticleMeta {
        title: String,
        author_display_name: String,
        author_sort_name: Option<String>,
        published_at: Option<time::OffsetDateTime>,
        html: String,
    }
    let meta = sqlx::query_as!(
        ArticleMeta,
        r#"SELECT a.title,
                  u.display_name AS "author_display_name!",
                  u.sort_name    AS "author_sort_name?",
                  a.published_at,
                  a.html AS "html!"
           FROM articles a
           JOIN users u ON u.id = a.user_id
           WHERE a.id = $1 AND a.status IN ('published', 'archived')"#,
        article_id,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Article not found or not published".into()))?;

    // Reject fabricated quotes: the text must actually occur in the cited
    // article. Without this a user could attribute arbitrary text to another
    // author (the snapshot freezes that author's name) and, once embedded in
    // a published article, serve it publicly.
    if !quote_text_occurs_in_html(&meta.html, text) {
        return Err(AppError::BadRequest(
            "Quotation text was not found in the cited article.".into(),
        ));
    }

    let new_id = sqlx::query_scalar!(
        r#"INSERT INTO article_quotations (
               user_id, article_id, article_title,
               author_display_name, author_sort_name,
               source_published_at, text, html
           )
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           RETURNING id"#,
        user_id,
        article_id,
        meta.title,
        meta.author_display_name,
        meta.author_sort_name,
        meta.published_at,
        text,
        html,
    )
    .fetch_one(pool)
    .await?;

    let row = fetch_article_quotation_row(pool, new_id).await?;
    Ok((article_quotation_from_row(row), true))
}

pub async fn list_article_quotations(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ArticleQuotationResponse>, AppError> {
    let rows = sqlx::query_as!(
        ArticleQuotationRow,
        r#"SELECT aq.id, aq.article_id, aq.article_title,
                  aq.author_display_name, aq.text, aq.html,
                  COUNT(qn.id) AS "note_count?",
                  aq.created_at
           FROM article_quotations aq
           LEFT JOIN quotation_notes qn ON qn.article_quotation_id = aq.id
           WHERE aq.user_id = $1
           GROUP BY aq.id
           ORDER BY aq.created_at DESC"#,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(article_quotation_from_row).collect())
}

pub async fn get_article_quotation(
    pool: &PgPool,
    id: Uuid,
) -> Result<ArticleQuotationResponse, AppError> {
    let row = fetch_article_quotation_row(pool, id).await?;
    Ok(article_quotation_from_row(row))
}

pub async fn delete_article_quotation(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"DELETE FROM article_quotations WHERE id = $1 AND user_id = $2"#,
        id,
        user_id,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Article quotation not found".into()));
    }
    Ok(())
}

pub struct UnifiedArticleQuotationRow {
    pub id: Uuid,
    pub article_id: Option<Uuid>,
    pub article_title: String,
    pub author_display_name: String,
    pub text: String,
    pub note_count: Option<i64>,
    pub created_at: time::OffsetDateTime,
}

pub async fn list_article_quotations_for_unified(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<UnifiedArticleQuotationRow>, AppError> {
    let rows = sqlx::query_as!(
        UnifiedArticleQuotationRow,
        r#"SELECT aq.id, aq.article_id, aq.article_title,
                  aq.author_display_name, aq.text,
                  COUNT(qn.id) AS "note_count?",
                  aq.created_at
           FROM article_quotations aq
           LEFT JOIN quotation_notes qn ON qn.article_quotation_id = aq.id
           WHERE aq.user_id = $1
           GROUP BY aq.id
           ORDER BY aq.created_at DESC"#,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

async fn fetch_article_quotation_row(
    pool: &PgPool,
    id: Uuid,
) -> Result<ArticleQuotationRow, AppError> {
    sqlx::query_as!(
        ArticleQuotationRow,
        r#"SELECT aq.id, aq.article_id, aq.article_title,
                  aq.author_display_name, aq.text, aq.html,
                  COUNT(qn.id) AS "note_count?",
                  aq.created_at
           FROM article_quotations aq
           LEFT JOIN quotation_notes qn ON qn.article_quotation_id = aq.id
           WHERE aq.id = $1
           GROUP BY aq.id"#,
        id,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::NotFound("Article quotation not found".into()))
}

#[cfg(test)]
mod tests {
    use super::quote_text_occurs_in_html;

    const ARTICLE: &str = "<p>Kant argues that space &amp; time are <em>a priori</em> forms \
         of intuition.</p><p>Reason has its limits.</p>";

    #[test]
    fn accepts_verbatim_quote() {
        assert!(quote_text_occurs_in_html(
            ARTICLE,
            "space & time are a priori"
        ));
    }

    #[test]
    fn accepts_quote_spanning_paragraphs() {
        // Selection joins across the </p><p> boundary; whitespace/tags are
        // normalized away.
        assert!(quote_text_occurs_in_html(
            ARTICLE,
            "forms of intuition. Reason has its limits."
        ));
    }

    #[test]
    fn accepts_quote_with_decoded_entity() {
        // Source stores `&amp;`; the client selection has a literal `&`.
        assert!(quote_text_occurs_in_html(ARTICLE, "space & time"));
    }

    #[test]
    fn accepts_quote_crossing_inline_tags() {
        // `<em>a priori</em>` — the quote has no tags, source has them.
        assert!(quote_text_occurs_in_html(
            ARTICLE,
            "time are a priori forms"
        ));
    }

    #[test]
    fn accepts_case_insensitively() {
        assert!(quote_text_occurs_in_html(
            ARTICLE,
            "SPACE & TIME ARE A PRIORI"
        ));
    }

    #[test]
    fn accepts_despite_punctuation_and_whitespace_differences() {
        // Extra punctuation and collapsed/expanded whitespace don't matter —
        // only alphanumeric run order does.
        assert!(quote_text_occurs_in_html(
            ARTICLE,
            "  a-priori   forms, of  intuition!!  "
        ));
    }

    #[test]
    fn accepts_numeric_and_curly_entity_apostrophe() {
        // Source uses a numeric entity for a curly apostrophe; client text has
        // a literal (curly or straight) apostrophe. Both normalize away.
        let src = "<p>It&#8217;s Kant&rsquo;s critique.</p>";
        assert!(quote_text_occurs_in_html(
            src,
            "it\u{2019}s kant's critique"
        ));
    }

    #[test]
    fn accepts_non_ascii_words() {
        let src = "<p>Die Kritik der reinen Vernunft war Kants Würde.</p>";
        assert!(quote_text_occurs_in_html(
            src,
            "reinen Vernunft war Kants Würde"
        ));
    }

    #[test]
    fn accepts_empty_quote_text() {
        // Degenerate but harmless: an empty needle is a substring of anything.
        assert!(quote_text_occurs_in_html(ARTICLE, ""));
        assert!(quote_text_occurs_in_html(ARTICLE, "   "));
    }

    #[test]
    fn rejects_fabricated_quote() {
        assert!(!quote_text_occurs_in_html(
            ARTICLE,
            "Kant admits he plagiarized the whole book"
        ));
    }

    #[test]
    fn rejects_quote_with_extra_injected_word() {
        assert!(!quote_text_occurs_in_html(
            ARTICLE,
            "space & time are false forms of intuition"
        ));
    }

    #[test]
    fn rejects_reordered_words() {
        // Same words, different order — not a contiguous substring.
        assert!(!quote_text_occurs_in_html(
            ARTICLE,
            "time & space are a priori"
        ));
    }

    #[test]
    fn rejects_word_absent_from_source() {
        assert!(!quote_text_occurs_in_html(ARTICLE, "reason has no limits"));
    }

    #[test]
    fn rejects_text_from_a_different_article() {
        assert!(!quote_text_occurs_in_html(
            ARTICLE,
            "Hume was Kant's great antagonist"
        ));
    }

    #[test]
    fn does_not_let_tag_or_entity_names_leak_into_the_haystack() {
        // The literal words "em", "amp", "priori" from tags/entities must not
        // become matchable tokens that a fabricated quote could exploit.
        assert!(!quote_text_occurs_in_html(ARTICLE, "em amp"));
    }
}
