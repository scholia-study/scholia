//! Sentence `natural_key` formats — the stable per-sentence identity strings the
//! importers store on fresh insert and the reconciler matches against. Both
//! paths must build them identically, so the format lives in exactly one place.

/// `<source_ref>/b<block_position>/s<sentence_position>` — a block sentence.
pub fn natural_key(source_ref: &str, block_position: i16, sentence_position: i16) -> String {
    format!("{source_ref}/b{block_position}/s{sentence_position}")
}

/// `<source_ref>/fn<footnote_number>/s<sentence_position>` — a footnote sentence.
pub fn footnote_natural_key(
    source_ref: &str,
    footnote_number: i32,
    sentence_position: i16,
) -> String {
    format!("{source_ref}/fn{footnote_number}/s{sentence_position}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_match_legacy() {
        assert_eq!(natural_key("sonnets:1", 1, 0), "sonnets:1/b1/s0");
        assert_eq!(footnote_natural_key("010", 3, 2), "010/fn3/s2");
    }
}
