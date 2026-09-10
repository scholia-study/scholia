use unicode_normalization::UnicodeNormalization;

/// Unifies elision-mark lookalikes to U+2019 and NFC-normalizes.
///
/// Perseus TEI elides with U+02BC MODIFIER LETTER APOSTROPHE, which is
/// Unicode category Lm (a *letter*), so a letter-based tokenizer reads
/// `δʼ` as one word. Rewriting it to U+2019 (category Pf) lets such
/// tokenizers split on it like any other punctuation mark.
///
/// Greek text only. A straight `'` is an elision mark in a Greek
/// transcription and is unified with the rest, so running this over English
/// prose would rewrite its apostrophes too.
pub fn normalize_greek(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\u{02BC}' | '\u{2018}' | '\u{0027}' | '\u{1FBD}' => '\u{2019}',
            other => other,
        })
        .nfc()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unifies_modifier_letter_apostrophe() {
        let input = "δ\u{02BC} ὃς";
        let output = normalize_greek(input);
        assert!(output.contains('\u{2019}'));
        assert!(!output.contains('\u{02BC}'));
    }

    #[test]
    fn composes_to_nfc() {
        let input = "\u{03B1}\u{0301}";
        let output = normalize_greek(input);
        assert_eq!(output, "\u{03AC}");
    }

    #[test]
    fn is_idempotent() {
        let sample = "ἀλλὰ περιμενοῦμεν, ἦ δ\u{02BC} ὃς ὁ Γλαύκων.";
        let once = normalize_greek(sample);
        let twice = normalize_greek(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn preserves_accents_and_breathings() {
        let input = "ἀγαθός";
        assert_eq!(normalize_greek(input), input);
    }

    #[test]
    fn preserves_iota_subscript() {
        let input = "ᾳῃῳ";
        assert_eq!(normalize_greek(input), input);

        let psyche_dative = normalize_greek("ψυχῇ");
        let psyche_nominative = normalize_greek("ψυχή");
        assert_ne!(psyche_dative, psyche_nominative);
    }

    #[test]
    fn preserves_final_sigma() {
        let output = normalize_greek("λόγος");
        assert!(output.ends_with('\u{03C2}'));
        assert!(!output.ends_with('\u{03C3}'));
    }

    #[test]
    fn real_source_line_has_no_modifier_apostrophe() {
        let input = "ἀλλὰ περιμενοῦμεν, ἦ δ\u{02BC} ὃς ὁ Γλαύκων.";
        let output = normalize_greek(input);

        assert!(!output.contains('\u{02BC}'));
        assert_eq!(output.matches('\u{2019}').count(), 1);

        let expected = input.replace('\u{02BC}', "\u{2019}");
        assert_eq!(output, expected);
    }

    #[test]
    fn leaves_latin_text_untouched() {
        let input = "The quick brown fox jumps over the lazy dog.";
        assert_eq!(normalize_greek(input), input);
    }

    /// Guards the doc comment's caveat: a straight apostrophe is an elision
    /// mark here, so this must not be pointed at English prose.
    #[test]
    fn straight_apostrophe_is_unified_as_elision() {
        assert_eq!(normalize_greek("δ' ὃς"), "δ\u{2019} ὃς");
        assert_eq!(normalize_greek("don't"), "don\u{2019}t");
    }
}
