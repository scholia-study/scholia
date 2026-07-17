use crate::system::error::AppError;

// All user-writable field caps live here. Each limit is applied at the
// handler (or db helper) entry point. Changing a value here changes the
// enforced ceiling everywhere — do not duplicate limits elsewhere.

pub const MAX_ARTICLE_TITLE: usize = 255;
pub const MAX_ARTICLE_DESCRIPTION: usize = 250;
pub const MAX_ARTICLE_MARKDOWN: usize = 200_000;
pub const MAX_ARTICLE_TOPICS: usize = 5;

pub const MAX_NOTE_BODY: usize = 2_000;
pub const MAX_NOTE_TAG_LEN: usize = 50;
pub const MAX_NOTE_TAGS: usize = 10;

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
pub const MIN_PUBLICATION_YEAR: i16 = -3000;
pub const MAX_PUBLICATION_YEAR: i16 = 2100;
pub const MIN_SOURCE_PAGE: i32 = 0;
pub const MAX_SOURCE_PAGE: i32 = 999_999;

pub const MAX_PERSON_NAME: usize = 255;
pub const MAX_PERSON_SORT_NAME: usize = 255;

pub const MAX_RESOURCE_QUOTED_TEXT: usize = 50_000;
pub const MAX_RESOURCE_EDITOR_NOTE: usize = 5_000;
pub const MAX_RESOURCE_ADMIN_NOTES: usize = 5_000;
pub const MAX_RESOURCE_SOURCE_LOCATION: usize = 500;
pub const MIN_RESOURCE_SOURCE_PAGE: i32 = 0;
pub const MAX_RESOURCE_SOURCE_PAGE: i32 = 9_999;

pub const MAX_ARTICLE_QUOTATION_TEXT: usize = 50_000;
pub const MAX_ARTICLE_QUOTATION_HTML: usize = 100_000;

pub const MAX_EMAIL: usize = 254;
pub const MAX_DISPLAY_NAME: usize = 100;
pub const MIN_PASSWORD: usize = 8;
pub const MAX_PASSWORD: usize = 128;
pub const MAX_PASSWORD_CHANGE_PER_HOUR: i64 = 5;

pub const MAX_PROFILE_BIO: usize = 500;
pub const MAX_PROFILE_TITLE: usize = 100;
pub const MAX_PROFILE_LOCATION: usize = 100;
pub const MAX_PROFILE_WEBSITE_URL: usize = 250;

pub const MIN_FEEDBACK_BODY: usize = 5;
pub const MAX_FEEDBACK_BODY: usize = 5_000;
pub const MAX_FEEDBACK_URL: usize = 2_048;
pub const MAX_FEEDBACK_USER_AGENT: usize = 500;
pub const MAX_FEEDBACK_ADMIN_NOTES: usize = 5_000;
pub const MAX_FEEDBACK_PER_DAY: i64 = 20;

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

/// Extract a URL scheme (the token before the first `:`, per RFC 3986) if one
/// is present. Tab/newline/CR are stripped first because browsers ignore them
/// inside URLs, so `java\nscript:` would still execute as `javascript:`.
fn url_scheme(value: &str) -> Option<String> {
    let cleaned: String = value
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .collect();
    let mut chars = cleaned.trim_start().chars();
    let mut scheme = String::new();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => scheme.push(c.to_ascii_lowercase()),
        _ => return None,
    }
    for c in chars {
        match c {
            ':' => return Some(scheme),
            c if c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.') => {
                scheme.push(c.to_ascii_lowercase())
            }
            // Any other char before a `:` means there is no scheme.
            _ => return None,
        }
    }
    None
}

/// Reject a URL/link value whose scheme is outside the safe web allowlist
/// (`http`/`https`). A value with no scheme (a bare domain, a DOI like
/// `10.x/y`, a relative path) is allowed. Blocks `javascript:`, `data:`, etc.
/// from being stored and later rendered as a clickable `href`.
pub fn check_url_scheme(field: &str, value: &str) -> Result<(), AppError> {
    if let Some(scheme) = url_scheme(value)
        && !matches!(scheme.as_str(), "http" | "https")
    {
        return Err(AppError::BadRequest(format!(
            "{field} must be an http or https URL"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check_url_scheme;

    fn ok(v: &str) -> bool {
        check_url_scheme("URL", v).is_ok()
    }

    #[test]
    fn allows_http_https_and_schemeless() {
        assert!(ok("http://example.com"));
        assert!(ok("https://example.com/path?q=1"));
        assert!(ok("HTTPS://EXAMPLE.COM")); // scheme is case-insensitive
        assert!(ok("example.com")); // bare domain, no scheme
        assert!(ok("10.1234/joss.00123")); // DOI, no scheme
        assert!(ok("/relative/path"));
        assert!(ok("")); // empty, no scheme
    }

    #[test]
    fn rejects_dangerous_schemes() {
        assert!(!ok("javascript:alert(1)"));
        assert!(!ok("JavaScript:alert(1)"));
        assert!(!ok("  javascript:alert(1)")); // leading whitespace
        assert!(!ok("data:text/html,<script>alert(1)</script>"));
        assert!(!ok("vbscript:msgbox(1)"));
        assert!(!ok("file:///etc/passwd"));
        assert!(!ok("mailto:a@b.com")); // not wanted for these web-URL fields
    }

    #[test]
    fn rejects_whitespace_smuggled_scheme() {
        // Browsers strip tab/newline/CR inside URLs; the check must too.
        assert!(!ok("java\nscript:alert(1)"));
        assert!(!ok("java\tscript:alert(1)"));
        assert!(!ok("jav\rascript:alert(1)"));
    }
}
