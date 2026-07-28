//! The writing domain: user-authored articles, quotations (with notes and
//! tags), the article↔quotation embeds linking them, and the derived
//! passage-reference index (which articles quote which passages). Features
//! are flat siblings; `quotations` and `article_quotations` are mutually
//! coupled. Other domains reach writing only through the re-exports below.

mod article_passage_references;
mod article_quotations;
mod article_reviews;
mod articles;
mod quotations;

use utoipa_axum::router::OpenApiRouter;

use crate::system::state::AppState;

// Cross-domain facade — what the identity domain needs to render an
// author's public profile (their published articles).
pub use articles::db::list_published_articles_by_author;
pub use articles::models::{ArticleResponse, PublicArticleListQuery};
// Orphan schemas (no referencing route) re-exported so `crate::ApiDoc`
// can keep registering them to hold the OpenAPI surface byte-identical.
// See the note in `crate::ApiDoc`.
pub use quotations::models::{QuotationWithContextListResponse, QuotationWithContextResponse};

/// Authenticated user endpoints: managing one's quotations, notes, tags,
/// articles, and article-quotation embeds.
pub fn user_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        // Quotations, notes, tags.
        .routes(utoipa_axum::routes!(
            quotations::handlers::list_quotations,
            quotations::handlers::create_quotation
        ))
        .routes(utoipa_axum::routes!(quotations::handlers::delete_quotation))
        .routes(utoipa_axum::routes!(
            quotations::handlers::list_notes,
            quotations::handlers::create_note
        ))
        .routes(utoipa_axum::routes!(
            quotations::handlers::update_note,
            quotations::handlers::delete_note
        ))
        .routes(utoipa_axum::routes!(quotations::handlers::list_tags))
        .routes(utoipa_axum::routes!(
            quotations::handlers::list_all_quotations
        ))
        .routes(utoipa_axum::routes!(quotations::handlers::list_all_notes))
        // Articles.
        .routes(utoipa_axum::routes!(articles::handlers::create_article))
        .routes(utoipa_axum::routes!(articles::handlers::list_user_articles))
        .routes(utoipa_axum::routes!(
            articles::handlers::get_user_article,
            articles::handlers::update_article
        ))
        .routes(utoipa_axum::routes!(articles::handlers::publish_article))
        .routes(utoipa_axum::routes!(articles::handlers::archive_article))
        // Article-quotation embeds.
        .routes(utoipa_axum::routes!(
            article_quotations::handlers::create_article_quotation,
            article_quotations::handlers::list_article_quotations
        ))
        .routes(utoipa_axum::routes!(
            article_quotations::handlers::delete_article_quotation
        ))
        // Article reviews: author submission plus the shared review
        // surface (author or `ArticlesReview` holder — checked in-handler).
        .routes(utoipa_axum::routes!(
            article_reviews::handlers::create_review_request
        ))
        .routes(utoipa_axum::routes!(
            article_reviews::handlers::withdraw_review_request
        ))
        .routes(utoipa_axum::routes!(
            article_reviews::handlers::get_review_request
        ))
        .routes(utoipa_axum::routes!(
            article_reviews::handlers::list_review_comments,
            article_reviews::handlers::create_review_comment
        ))
        .routes(utoipa_axum::routes!(
            article_reviews::handlers::create_review_reply
        ))
        .routes(utoipa_axum::routes!(
            article_reviews::handlers::update_review_comment
        ))
        .routes(utoipa_axum::routes!(
            article_reviews::handlers::list_review_messages,
            article_reviews::handlers::create_review_message
        ))
        .routes(utoipa_axum::routes!(
            article_reviews::handlers::get_review_activity
        ))
}

/// Public endpoints: published articles, topics, editorial labels, the
/// sentence-batch fetch, reading a single article-quotation embed, and
/// the per-passage article-reference lookup.
pub fn public_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(
            articles::handlers::list_published_articles
        ))
        .routes(utoipa_axum::routes!(
            articles::handlers::get_published_article
        ))
        .routes(utoipa_axum::routes!(articles::handlers::get_article_by_id))
        .routes(utoipa_axum::routes!(articles::handlers::list_topics))
        .routes(utoipa_axum::routes!(
            articles::handlers::list_editorial_labels
        ))
        .routes(utoipa_axum::routes!(articles::handlers::batch_sentences))
        .routes(utoipa_axum::routes!(
            article_quotations::handlers::get_article_quotation
        ))
        .routes(utoipa_axum::routes!(
            article_passage_references::handlers::list_article_references
        ))
}

/// Admin/editor endpoints: applying and removing editorial labels on
/// articles (gated by `Permission::ArticleLabelsManage`) and the article
/// review queue (gated by `Permission::ArticlesReview` — editors qualify).
pub fn admin_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(
            articles::handlers::apply_article_label
        ))
        .routes(utoipa_axum::routes!(
            articles::handlers::remove_article_label
        ))
        .routes(utoipa_axum::routes!(
            article_reviews::handlers::list_article_review_queue
        ))
        .routes(utoipa_axum::routes!(
            article_reviews::handlers::decide_article_review
        ))
}
