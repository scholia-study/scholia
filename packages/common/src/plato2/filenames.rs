use super::toc;

/// Slugify an English TOC label: lowercase, apostrophes dropped so
/// `Callicles' Speech` reads `callicles_speech`, everything else
/// non-alphanumeric collapsed to a single underscore.
pub fn slugify(label: &str) -> String {
    let mut slug = String::with_capacity(label.len());
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if ch == '\'' || ch == '\u{2019}' {
            continue;
        } else if !slug.ends_with('_') {
            slug.push('_');
        }
    }
    slug.trim_matches('_').to_string()
}

/// 1-based document position. plato2 is complete at ingest and grows only by
/// correction, so positions are the flat index with no reserved headroom.
pub fn position_number(flat_index: usize) -> usize {
    flat_index + 1
}

/// A conversation's filename token: `The Conversation with Polus` → `polus`.
/// Every division file carries it, so the interlocutor's bare name is the most
/// that earns its place at the head of a filename.
pub fn conversation_slug(label: &str) -> String {
    slugify(label)
        .strip_prefix("the_conversation_with_")
        .map(str::to_string)
        .unwrap_or_else(|| slugify(label))
}

/// A division's filename carries its conversation, so the three stay visually
/// grouped in a flat directory: `002_gorgias_the_question…`. A conversation
/// node needs no prefix — its own name already is one.
pub fn filename(flat_index: usize, label: &str, conversation_label: Option<&str>) -> String {
    let pos = position_number(flat_index);
    match conversation_label {
        Some(conv) => format!(
            "{:03}_{}_{}.md",
            pos,
            conversation_slug(conv),
            slugify(label)
        ),
        None => format!("{:03}_{}.md", pos, conversation_slug(label)),
    }
}

pub fn all_filenames() -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut current: Option<&'static str> = None;
    for (idx, entry) in toc::TOC.iter().enumerate() {
        if entry.depth == 0 {
            current = Some(entry.label);
            out.push((idx, filename(idx, entry.label, None)));
        } else {
            out.push((idx, filename(idx, entry.label, current)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_drops_apostrophes_and_collapses_punctuation() {
        assert_eq!(
            slugify("The Conversation with Polus"),
            "the_conversation_with_polus"
        );
        assert_eq!(
            slugify("Callicles' Speech: Nature against Convention"),
            "callicles_speech_nature_against_convention"
        );
        assert_eq!(slugify("Is Rhetoric a Craft?"), "is_rhetoric_a_craft");
    }

    #[test]
    fn conversation_slug_keeps_only_the_interlocutor() {
        assert_eq!(
            conversation_slug("The Conversation with Gorgias"),
            "gorgias"
        );
        assert_eq!(conversation_slug("The Conversation with Polus"), "polus");
        assert_eq!(
            conversation_slug("The Conversation with Callicles"),
            "callicles"
        );
    }

    #[test]
    fn divisions_carry_their_conversation_prefix() {
        let all = all_filenames();
        assert_eq!(
            all[0].1, "001_gorgias.md",
            "conversation node needs no prefix"
        );
        let mut conv = String::new();
        for (idx, name) in &all {
            let e = &toc::TOC[*idx];
            if e.depth == 0 {
                conv = conversation_slug(e.label);
            } else {
                assert!(
                    name.contains(&format!("_{conv}_")),
                    "{name} does not carry its conversation {conv}"
                );
            }
        }
    }

    #[test]
    fn filenames_are_unique_and_ordered() {
        let all = all_filenames();
        assert_eq!(all.len(), toc::TOC.len());
        let mut names: Vec<&str> = all.iter().map(|(_, n)| n.as_str()).collect();
        names.sort_unstable();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate filename");
        assert!(all[0].1.starts_with("001_"));
    }

    /// The position prefix must match the flat index, or curated files and
    /// `source_ref`s drift apart on the next import.
    #[test]
    fn filename_prefix_matches_position() {
        for (idx, name) in all_filenames() {
            let prefix: usize = name[..3].parse().unwrap();
            assert_eq!(prefix, position_number(idx));
        }
    }
}
