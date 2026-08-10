//! Filename ⇄ node mapping for the curated hegel1 layers: one flat directory
//! of `NNN_slug.md`, one file per TOC node, `NNN` being the node's document
//! position. Slugs are not truncated — kant3 has equally long ones.

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

/// Transliterate common German/Latin characters to ASCII equivalents. The
/// long-s `ſ` is a glyph variant of `s`, not a letter of its own, so the
/// diplomatic layer's `Gewiſsheit` and a modernized `Gewissheit` slugify alike.
pub fn transliterate(ch: char) -> Option<&'static str> {
    match ch {
        'ä' | 'Ä' => Some("ae"),
        'ö' | 'Ö' => Some("oe"),
        'ü' | 'Ü' => Some("ue"),
        'ß' => Some("ss"),
        'ſ' => Some("s"),
        _ => None,
    }
}

/// Map a 0-based index into the flat TOC array to its 1-based document
/// position. The position is carried by the entry itself, so a node retired
/// after import can leave a permanent numbering gap (its `source_ref` is the
/// zero-padded position and is the reconcile identity anchor) without
/// renumbering its neighbours.
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
        assert_eq!(slugify("Vorrede"), "vorrede");
        assert_eq!(slugify("VI. Der Geist"), "vi_der_geist");
        assert_eq!(
            slugify("I. Die sinnliche Gewiſsheit; oder das Diese und das Meynen"),
            "i_die_sinnliche_gewissheit_oder_das_diese_und_das_meynen"
        );
        assert_eq!(
            slugify("b. Das Gesetz des Herzens, und der Wahn- sinn des Eigendünkels"),
            "b_das_gesetz_des_herzens_und_der_wahn_sinn_des_eigenduenkels"
        );
        assert_eq!(
            slugify("a. Das geistige Thierreich und der Betrug, oder die Sache selbs’"),
            "a_das_geistige_thierreich_und_der_betrug_oder_die_sache_selbs"
        );
    }

    #[test]
    fn test_filename() {
        assert_eq!(filename(0), "001_vorrede.md");
        assert_eq!(filename(49), "050_viii_das_absolute_wissen.md");
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
            "duplicate filenames in hegel1 TOC"
        );
    }

    #[test]
    fn test_position_number_is_contiguous() {
        for i in 0..toc::toc_len() {
            assert_eq!(position_number(i), i + 1);
        }
    }
}
