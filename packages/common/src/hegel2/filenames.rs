//! Filename ⇄ node mapping for the curated hegel2 layers: one flat directory
//! of `NNN_slug.md`, one file per TOC node, `NNN` being the node's document
//! position (274 nodes — three digits still suffice).

use super::toc;

/// Slugify a label: lowercase, transliterate German characters and the Greek
/// section letters (α/β/γ, used in the Endlichkeit subsections),
/// non-alphanumeric → `_`, collapse, trim.
pub fn slugify(label: &str) -> String {
    let mut slug = String::with_capacity(label.len());
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if let Some(replacement) = transliterate(ch) {
            slug.push_str(replacement);
        } else if !slug.ends_with('_') {
            slug.push('_');
        }
    }
    slug.trim_matches('_').to_string()
}

/// Transliterate German/Greek characters to ASCII equivalents.
pub fn transliterate(ch: char) -> Option<&'static str> {
    match ch {
        'ä' | 'Ä' => Some("ae"),
        'ö' | 'Ö' => Some("oe"),
        'ü' | 'Ü' => Some("ue"),
        'ß' => Some("ss"),
        'ſ' => Some("s"),
        'α' => Some("alpha"),
        'β' => Some("beta"),
        'γ' => Some("gamma"),
        _ => None,
    }
}

/// Map a 0-based index into the flat TOC array to its 1-based document
/// position (carried by the entry itself, so a retired node can leave a
/// permanent numbering gap without renumbering its neighbours).
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
    fn test_slugify() {
        assert_eq!(
            slugify("Vorrede zur zweyten Ausgabe"),
            "vorrede_zur_zweyten_ausgabe"
        );
        assert_eq!(
            slugify("Womit muß der Anfang der Wissenschaft gemacht werden?"),
            "womit_muss_der_anfang_der_wissenschaft_gemacht_werden"
        );
        assert_eq!(
            slugify("α. Die Unmittelbarkeit der Endlichkeit"),
            "alpha_die_unmittelbarkeit_der_endlichkeit"
        );
    }

    #[test]
    fn test_filename() {
        assert_eq!(filename(0), "001_erster_theil_die_objective_logik.md");
    }

    #[test]
    fn test_all_filenames_unique() {
        let fnames = all_filenames();
        let unique: HashSet<&String> = fnames.iter().map(|(_, f)| f).collect();
        assert_eq!(unique.len(), fnames.len());
    }

    #[test]
    fn test_position_number_is_contiguous() {
        for i in 0..toc::toc_len() {
            assert_eq!(position_number(i), i + 1);
        }
    }
}
