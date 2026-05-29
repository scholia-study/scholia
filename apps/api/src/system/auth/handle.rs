use std::sync::OnceLock;

use regex::Regex;

use crate::system::error::AppError;

/// Maximum length of a handle in characters.
pub const MAX_HANDLE_LEN: usize = 40;

/// Days a user must wait between handle renames. The first rename
/// (from the auto-derived seed) is free — see `users.handle_changed_at`
/// being NULL on signup.
pub const HANDLE_RENAME_COOLDOWN_DAYS: i64 = 30;

/// Top-level paths and reserved words a handle must not collide with.
/// Keep alphabetised so this list stays scannable as it grows.
const RESERVED_HANDLES: &[&str] = &[
    "about",
    "admin",
    "api",
    "articles",
    "auth",
    "books",
    "by-id",
    "contribute",
    "feedback",
    "forgot-password",
    "index",
    "licence",
    "login",
    "privacy",
    "register",
    "reset-password",
    "static",
    "system",
    "terms",
    "user",
    "users",
    "verify-email",
];

/// `[a-z0-9]([a-z0-9-]{0,38}[a-z0-9])?` — lowercase, digits, hyphens,
/// 1–40 chars, no leading/trailing hyphen.
fn handle_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-z0-9]([a-z0-9-]{0,38}[a-z0-9])?$").expect("handle regex"))
}

/// Validate user-supplied or auto-derived handles. Caller is
/// responsible for the uniqueness check against the database.
pub fn validate_handle(handle: &str) -> Result<(), AppError> {
    if handle.is_empty() {
        return Err(AppError::BadRequest("Handle is required".into()));
    }
    if handle.chars().count() > MAX_HANDLE_LEN {
        return Err(AppError::BadRequest(format!(
            "Handle must be {MAX_HANDLE_LEN} characters or fewer"
        )));
    }
    if !handle_re().is_match(handle) {
        return Err(AppError::BadRequest(
            "Handle may only contain lowercase letters, digits, and hyphens (no leading or trailing hyphen)".into(),
        ));
    }
    if RESERVED_HANDLES.contains(&handle) {
        return Err(AppError::BadRequest(format!(
            "Handle '{handle}' is reserved"
        )));
    }
    Ok(())
}

/// Derive an initial handle from a free-form display name. Lowercases,
/// strips non-alphanumerics, collapses runs of `-`. Caller MUST resolve
/// uniqueness (typically by appending `-2`, `-3`, …) before insert.
///
/// If derivation produces nothing usable (e.g. all-emoji display name),
/// the caller should fall back to a random suffix like `user-abc123`.
pub fn derive_handle(display_name: &str) -> String {
    let lowered = display_name.to_lowercase();
    let mut out = String::with_capacity(lowered.len());
    let mut prev_dash = false;
    for c in lowered.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.chars().count() > MAX_HANDLE_LEN {
        out = out.chars().take(MAX_HANDLE_LEN).collect();
        while out.ends_with('-') {
            out.pop();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_simple() {
        assert_eq!(derive_handle("Filip Niklas"), "filip-niklas");
        assert_eq!(derive_handle("filip"), "filip");
    }

    #[test]
    fn collapses_punctuation() {
        assert_eq!(derive_handle("Filip!! Niklas??"), "filip-niklas");
        assert_eq!(derive_handle("--filip---niklas--"), "filip-niklas");
    }

    #[test]
    fn handles_unicode() {
        // Non-ASCII letters act as separators, same treatment as spaces
        // and punctuation. Users with diacritics in their display name
        // get a slightly noisy seed and can rename to clean it up.
        assert_eq!(derive_handle("Émile Zóla"), "mile-z-la");
    }

    #[test]
    fn validates_charset() {
        assert!(validate_handle("filip").is_ok());
        assert!(validate_handle("filip-niklas").is_ok());
        assert!(validate_handle("a").is_ok());
        assert!(validate_handle("Filip").is_err()); // uppercase
        assert!(validate_handle("-filip").is_err()); // leading hyphen
        assert!(validate_handle("filip-").is_err()); // trailing hyphen
        assert!(validate_handle("filip_niklas").is_err()); // underscore
        assert!(validate_handle("filip niklas").is_err()); // space
        assert!(validate_handle("").is_err());
    }

    #[test]
    fn rejects_reserved() {
        assert!(validate_handle("admin").is_err());
        assert!(validate_handle("api").is_err());
        assert!(validate_handle("system").is_err());
    }

    #[test]
    fn enforces_max_length() {
        let long = "a".repeat(MAX_HANDLE_LEN + 1);
        assert!(validate_handle(&long).is_err());
        let ok = "a".repeat(MAX_HANDLE_LEN);
        assert!(validate_handle(&ok).is_ok());
    }
}
