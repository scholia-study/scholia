use super::toc;

/// Slugify an English TOC label: lowercase, apostrophes dropped so
/// `Thrasymachus' Speech` reads `thrasymachus_speech`, everything else
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

/// 1-based document position. plato1 is complete at ingest and grows only by
/// correction, so positions are the flat index with no reserved headroom.
pub fn position_number(flat_index: usize) -> usize {
    flat_index + 1
}

/// A book's filename token, abbreviated: `Book I` → `bk_i`. Every one of the
/// 118 files carries it, so the full title would cost 118 × 3 characters of
/// line length for no added meaning.
pub fn book_slug(book_label: &str) -> String {
    slugify(book_label).replace("book_", "bk_")
}

/// A division's filename carries its book, so the ten books stay visually
/// grouped in a flat directory of 118 files: `002_bk_i_the_descent…`. A book
/// node needs no prefix — its own name already is one.
pub fn filename(flat_index: usize, label: &str, book_label: Option<&str>) -> String {
    let pos = position_number(flat_index);
    match book_label {
        Some(book) => format!("{:03}_{}_{}.md", pos, book_slug(book), slugify(label)),
        None => format!("{:03}_{}.md", pos, book_slug(label)),
    }
}

pub fn all_filenames() -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut current_book: Option<&'static str> = None;
    for (idx, entry) in toc::TOC.iter().enumerate() {
        if entry.depth == 0 {
            current_book = Some(entry.label);
            out.push((idx, filename(idx, entry.label, None)));
        } else {
            out.push((idx, filename(idx, entry.label, current_book)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_drops_apostrophes_and_collapses_punctuation() {
        assert_eq!(slugify("Book I"), "book_i");
        assert_eq!(book_slug("Book I"), "bk_i");
        assert_eq!(book_slug("Book VIII"), "bk_viii");
        assert_eq!(
            slugify("Thrasymachus' Speech: The Shepherd and the Flock"),
            "thrasymachus_speech_the_shepherd_and_the_flock"
        );
        assert_eq!(
            slugify("Is Injustice Virtue and Wisdom?"),
            "is_injustice_virtue_and_wisdom"
        );
        assert_eq!(
            slugify("The Guardians' Happiness"),
            "the_guardians_happiness"
        );
    }

    #[test]
    fn divisions_carry_their_book_prefix() {
        let all = all_filenames();
        assert_eq!(all[0].1, "001_bk_i.md", "book node needs no prefix");
        assert_eq!(all[1].1, "002_bk_i_the_descent_to_the_piraeus.md");
        // Every depth-1 file names the book it belongs to.
        let mut book = String::new();
        for (idx, name) in &all {
            let e = &toc::TOC[*idx];
            if e.depth == 0 {
                book = book_slug(e.label);
            } else {
                assert!(
                    name.contains(&format!("_{book}_")),
                    "{name} does not carry its book {book}"
                );
            }
        }
    }

    #[test]
    fn filenames_are_unique_and_ordered() {
        let all = all_filenames();
        assert_eq!(all.len(), toc::toc_len());
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
