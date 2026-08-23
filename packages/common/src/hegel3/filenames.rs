//! Filename ⇄ node mapping for the curated hegel3 layers: one flat directory
//! of `NNN_slug.md`, one file per TOC node, `NNN` the document position.

use super::toc;

/// Slugify a label — the converter's rules: long-s and combining umlauts
/// folded, lowercase, non-alphanumeric → `_`, collapse, trim.
pub fn slugify(label: &str) -> String {
    let label = label
        .replace("a\u{364}", "ä")
        .replace("o\u{364}", "ö")
        .replace("u\u{364}", "ü")
        .replace("A\u{364}", "Ä")
        .replace("O\u{364}", "Ö")
        .replace("U\u{364}", "Ü");
    let mut slug = String::with_capacity(label.len());
    for ch in label.chars().flat_map(char::to_lowercase) {
        match ch {
            'ſ' => slug.push('s'),
            'ä' => slug.push_str("ae"),
            'ö' => slug.push_str("oe"),
            'ü' => slug.push_str("ue"),
            'ß' => slug.push_str("ss"),
            c if c.is_ascii_alphanumeric() => slug.push(c),
            _ => {
                if !slug.ends_with('_') {
                    slug.push('_');
                }
            }
        }
    }
    slug.trim_matches('_').to_string()
}

pub fn position_number(flat_index: usize) -> usize {
    toc::entries()[flat_index].position as usize
}

pub fn filename(flat_index: usize) -> String {
    let entry = &toc::entries()[flat_index];
    format!("{:03}_{}.md", entry.position, entry.slug)
}

/// The expected filename for every TOC entry, as (flat_index, filename).
pub fn all_filenames() -> Vec<(usize, String)> {
    (0..toc::toc_len()).map(|i| (i, filename(i))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn slugs_match_the_toc() {
        // repeated labels carry a counter suffix; the base must match
        for (i, e) in toc::entries().iter().enumerate() {
            let base = slugify(e.label);
            assert!(
                e.slug == base || e.slug.starts_with(&format!("{base}_")),
                "slug drift at index {i}: {} vs {base}",
                e.slug
            );
        }
    }

    #[test]
    fn filenames_are_unique() {
        let fnames = all_filenames();
        let unique: HashSet<&String> = fnames.iter().map(|(_, f)| f).collect();
        assert_eq!(unique.len(), fnames.len());
    }
}
