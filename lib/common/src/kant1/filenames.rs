use super::toc;

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

/// Generate filename: `001_motto.md`
pub fn filename(flat_index: usize, label: &str) -> String {
    format!("{:03}_{}.md", flat_index + 1, slugify(label))
}

/// Return the expected filename for each TOC entry (excluding 000_toc.md).
/// Vec of (flat_index, filename) pairs.
pub fn all_filenames() -> Vec<(usize, String)> {
    toc::flat_toc_entries()
        .iter()
        .map(|&(idx, _, _, label, _)| (idx, filename(idx, label)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Vorrede zur zweiten Auflage"), "vorrede_zur_zweiten_auflage");
        assert_eq!(slugify("Motto"), "motto");
        assert_eq!(slugify("§1"), "1");
        assert_eq!(slugify("I. Transscendentale Elementarlehre"), "i_transscendentale_elementarlehre");
        assert_eq!(slugify("Die transscendentale Ästhetik"), "die_transscendentale_aesthetik");
        assert_eq!(slugify("1. Hauptstück. Von dem Schematismus"), "1_hauptstueck_von_dem_schematismus");
        assert_eq!(slugify("Grundsätze"), "grundsaetze");
    }

    #[test]
    fn test_filename() {
        assert_eq!(filename(0, "Motto"), "001_motto.md");
        assert_eq!(filename(2, "Vorrede zur zweiten Auflage"), "003_vorrede_zur_zweiten_auflage.md");
    }

    #[test]
    fn test_all_filenames() {
        let fnames = all_filenames();
        assert_eq!(fnames.len(), 114);
        assert_eq!(fnames[0], (0, "001_motto.md".to_string()));
        assert_eq!(fnames[2], (2, "003_vorrede_zur_zweiten_auflage.md".to_string()));
    }
}
