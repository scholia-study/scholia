//! Text normalization + similarity used by the reconciling re-import to
//! decide whether a re-parsed sentence is unchanged, lightly edited, or the
//! product of a split/merge. Kept here (not in the importer) so both the
//! German and English passes share one definition of "the same text".

/// Collapse a sentence to a canonical form for comparison: trim, lowercase,
/// and squash every run of whitespace to a single space. Punctuation and
/// letters are preserved — a wording or punctuation edit is a real change —
/// but cosmetic whitespace/markdown re-wrapping is not.
pub fn normalize_for_match(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.trim().chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.extend(ch.to_lowercase());
            prev_space = false;
        }
    }
    out
}

/// True when two sentences are identical after normalization.
pub fn normalized_eq(a: &str, b: &str) -> bool {
    normalize_for_match(a) == normalize_for_match(b)
}

/// Similarity in `[0.0, 1.0]` between two sentences, computed as a
/// Levenshtein ratio over their normalized forms (`1.0` = identical,
/// `0.0` = nothing in common). Two empty strings are treated as identical.
pub fn similarity(a: &str, b: &str) -> f64 {
    let na = normalize_for_match(a);
    let nb = normalize_for_match(b);
    let ca: Vec<char> = na.chars().collect();
    let cb: Vec<char> = nb.chars().collect();
    let max_len = ca.len().max(cb.len());
    if max_len == 0 {
        return 1.0;
    }
    let dist = levenshtein(&ca, &cb);
    1.0 - (dist as f64) / (max_len as f64)
}

/// Similarity between a single sentence and the concatenation of several
/// (joined with a single space). Used to test split/merge hypotheses:
/// `concat_similarity(old, &[new1, new2])` answers "did `old` split into
/// these two?".
pub fn concat_similarity(whole: &str, parts: &[&str]) -> f64 {
    let joined = parts
        .iter()
        .map(|p| normalize_for_match(p))
        .collect::<Vec<_>>()
        .join(" ");
    similarity(whole, &joined)
}

/// Standard two-row Levenshtein edit distance over char slices. Inputs are
/// single sentences, so the O(n·m) table is small.
fn levenshtein(a: &[char], b: &[char]) -> usize {
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_and_case_are_ignored() {
        assert!(normalized_eq("The  Cat\n sat.", "the cat sat."));
        assert!((similarity("The  Cat", "the cat") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn wording_change_is_detected() {
        assert!(!normalized_eq("the cat sat", "the dog sat"));
        let s = similarity("the cat sat on the mat", "the cat sat on the rug");
        assert!(s > 0.7 && s < 1.0, "got {s}");
    }

    #[test]
    fn split_concatenation_matches() {
        // One sentence split into two pieces reconstructs the original.
        let s = concat_similarity(
            "All bodies are extended, and they are heavy.",
            &["All bodies are extended,", "and they are heavy."],
        );
        assert!(s > 0.95, "got {s}");
    }

    #[test]
    fn unrelated_sentences_score_low() {
        assert!(similarity("the cat sat", "quantum entanglement") < 0.4);
    }
}
