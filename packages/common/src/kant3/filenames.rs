use super::toc;

/// Original 1-based TOC positions removed as spurious after review. kant3 starts
/// clean; the mechanism is kept so a future removal leaves a permanent numbering
/// gap rather than renumbering already-imported files (whose `source_ref` is the
/// zero-padded position and is the reconcile identity anchor).
const REMOVED_POSITIONS: &[u16] = &[];

/// Map a 0-based index into the flat TOC array to its 1-based document position
/// number, skipping the removed positions so numbering keeps a permanent gap.
pub fn position_number(flat_index: usize) -> usize {
    let mut pos = flat_index + 1;
    for &removed in REMOVED_POSITIONS {
        if pos >= removed as usize {
            pos += 1;
        }
    }
    pos
}

/// Slugify a label: lowercase, transliterate German characters,
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

/// Transliterate common German/Latin characters to ASCII equivalents.
pub fn transliterate(ch: char) -> Option<&'static str> {
    match ch {
        'ä' | 'Ä' => Some("ae"),
        'ö' | 'Ö' => Some("oe"),
        'ü' | 'Ü' => Some("ue"),
        'ß' => Some("ss"),
        _ => None,
    }
}

/// Generate filename: `001_vorrede.md`
pub fn filename(flat_index: usize, label: &str) -> String {
    format!("{:03}_{}.md", position_number(flat_index), slugify(label))
}

/// Return the expected filename for each TOC entry (excluding 000_toc.md).
/// Vec of (flat_index, filename) pairs.
pub fn all_filenames() -> Vec<(usize, String)> {
    toc::flat_toc_entries()
        .iter()
        .map(|&(idx, _, _, label, slug_override)| {
            let name = match slug_override {
                Some(s) => format!("{:03}_{}.md", position_number(idx), s),
                None => filename(idx, label),
            };
            (idx, name)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Vorrede"), "vorrede");
        assert_eq!(slugify("§ 55."), "55");
        assert_eq!(
            slugify("§ 1. Das Geschmacksurtheil ist ästhetisch"),
            "1_das_geschmacksurtheil_ist_aesthetisch"
        );
        assert_eq!(
            slugify("Eintheilung des ganzen Werks"),
            "eintheilung_des_ganzen_werks"
        );
        assert_eq!(
            slugify("A. Vom Mathematisch-Erhabenen"),
            "a_vom_mathematisch_erhabenen"
        );
    }

    #[test]
    fn test_filename() {
        assert_eq!(filename(0, "Vorrede"), "001_vorrede.md");
    }

    #[test]
    fn test_all_filenames_match_toc() {
        let fnames = all_filenames();
        assert_eq!(fnames.len(), toc::toc_len());
        assert_eq!(fnames[0], (0, "001_vorrede.md".to_string()));
    }

    #[test]
    fn test_all_filenames_unique() {
        let fnames = all_filenames();
        let unique: HashSet<&String> = fnames.iter().map(|(_, f)| f).collect();
        assert_eq!(
            unique.len(),
            fnames.len(),
            "duplicate filenames in kant3 TOC"
        );
    }
}
