/// Derive a Chicago-style "Last, First" bibliography sort key from a free-form
/// display name. The result is what we seed `users.sort_name` with at signup;
/// users can override it on their profile.
///
/// Rules:
///   - Single token: returned as-is. ("filip" → "filip")
///   - Multi-token:  "{last}, {rest joined by single space}".
///     ("Filip Niklas" → "Niklas, Filip";
///     "Friedrich Wilhelm Joseph Schelling" → "Schelling, Friedrich Wilhelm Joseph")
///   - Whitespace is collapsed; no special handling of particles ("von",
///     "van der", etc.) — users with those names override manually.
pub fn derive_sort_name(display_name: &str) -> String {
    let tokens: Vec<&str> = display_name.split_whitespace().collect();
    match tokens.as_slice() {
        [] => String::new(),
        [single] => (*single).to_string(),
        [rest @ .., last] => format!("{last}, {}", rest.join(" ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_token() {
        assert_eq!(derive_sort_name("filip"), "filip");
    }

    #[test]
    fn two_tokens() {
        assert_eq!(derive_sort_name("Filip Niklas"), "Niklas, Filip");
    }

    #[test]
    fn many_tokens() {
        assert_eq!(
            derive_sort_name("Friedrich Wilhelm Joseph Schelling"),
            "Schelling, Friedrich Wilhelm Joseph"
        );
    }

    #[test]
    fn collapses_whitespace() {
        assert_eq!(derive_sort_name("  Filip   Niklas  "), "Niklas, Filip");
    }

    #[test]
    fn empty() {
        assert_eq!(derive_sort_name(""), "");
        assert_eq!(derive_sort_name("   "), "");
    }
}
