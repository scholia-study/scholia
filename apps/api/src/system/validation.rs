use crate::system::error::AppError;

// ── Centralized input limits ─────────────────────────────
//
// All user-writable field caps live here. Each limit is applied at the
// handler (or db helper) entry point. Changing a value here changes the
// enforced ceiling everywhere — do not duplicate limits elsewhere.

// Articles
pub const MAX_ARTICLE_TITLE: usize = 255;
pub const MAX_ARTICLE_DESCRIPTION: usize = 250;
pub const MAX_ARTICLE_MARKDOWN: usize = 200_000;
pub const MAX_ARTICLE_TOPICS: usize = 5;

// Notes
pub const MAX_NOTE_BODY: usize = 2_000;
pub const MAX_NOTE_TAG_LEN: usize = 50;
pub const MAX_NOTE_TAGS: usize = 10;

// Sources
pub const MAX_SOURCE_TITLE: usize = 500;
pub const MAX_SOURCE_TITLE_DISPLAY: usize = 500;
pub const MAX_SOURCE_PUBLISHER: usize = 255;
pub const MAX_SOURCE_JOURNAL_NAME: usize = 255;
pub const MAX_SOURCE_DOI: usize = 255;
pub const MAX_SOURCE_EDITION: usize = 100;
pub const MAX_SOURCE_VOLUME: usize = 50;
pub const MAX_SOURCE_URL: usize = 2_048;
pub const MAX_SOURCE_ISBN_LEN: usize = 17;
pub const MAX_SOURCE_ISBNS: usize = 5;
// Allow ancient composition dates (BCE as negatives) through near-future.
pub const MIN_PUBLICATION_YEAR: i16 = -3000;
pub const MAX_PUBLICATION_YEAR: i16 = 2100;
pub const MIN_SOURCE_PAGE: i32 = 0;
pub const MAX_SOURCE_PAGE: i32 = 999_999;

// Persons
pub const MAX_PERSON_NAME: usize = 255;
pub const MAX_PERSON_SORT_NAME: usize = 255;

// Resources (book quotations)
pub const MAX_RESOURCE_QUOTED_TEXT: usize = 50_000;
pub const MAX_RESOURCE_EDITOR_NOTE: usize = 5_000;
pub const MAX_RESOURCE_ADMIN_NOTES: usize = 5_000;
pub const MAX_RESOURCE_SOURCE_LOCATION: usize = 500;
pub const MIN_RESOURCE_SOURCE_PAGE: i32 = 0;
pub const MAX_RESOURCE_SOURCE_PAGE: i32 = 9_999;

// Article quotations
pub const MAX_ARTICLE_QUOTATION_TEXT: usize = 50_000;
pub const MAX_ARTICLE_QUOTATION_HTML: usize = 100_000;

// Auth
pub const MAX_EMAIL: usize = 254;
pub const MAX_DISPLAY_NAME: usize = 100;
pub const MIN_PASSWORD: usize = 8;
pub const MAX_PASSWORD: usize = 128;

// Profile
pub const MAX_PROFILE_BIO: usize = 500;
pub const MAX_PROFILE_TITLE: usize = 100;
pub const MAX_PROFILE_LOCATION: usize = 100;
pub const MAX_PROFILE_WEBSITE_URL: usize = 250;

// Feedback
pub const MIN_FEEDBACK_BODY: usize = 5;
pub const MAX_FEEDBACK_BODY: usize = 5_000;
pub const MAX_FEEDBACK_URL: usize = 2_048;
pub const MAX_FEEDBACK_USER_AGENT: usize = 500;
pub const MAX_FEEDBACK_ADMIN_NOTES: usize = 5_000;
/// Per-user submissions allowed in any rolling 24h window.
pub const MAX_FEEDBACK_PER_DAY: i64 = 20;

// ── Helpers ──────────────────────────────────────────────

/// Reject a string longer than `max` Unicode scalar values.
pub fn check_max_len(field: &str, value: &str, max: usize) -> Result<(), AppError> {
    let len = value.chars().count();
    if len > max {
        return Err(AppError::BadRequest(format!(
            "{field} must be {max} characters or fewer (got {len})"
        )));
    }
    Ok(())
}

/// Reject an integer outside the inclusive range [min, max].
pub fn check_int_range<T>(field: &str, value: T, min: T, max: T) -> Result<(), AppError>
where
    T: PartialOrd + std::fmt::Display + Copy,
{
    if value < min || value > max {
        return Err(AppError::BadRequest(format!(
            "{field} must be between {min} and {max}"
        )));
    }
    Ok(())
}

/// Reject a collection larger than `max` items.
pub fn check_count<T>(field: &str, items: &[T], max: usize) -> Result<(), AppError> {
    if items.len() > max {
        return Err(AppError::BadRequest(format!(
            "{field} must contain {max} items or fewer (got {})",
            items.len()
        )));
    }
    Ok(())
}
