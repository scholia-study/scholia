use super::filenames::{position_number, slugify};
use super::toc_en;

/// Generate filename for an English TOC entry: `001_motto.md`
pub fn filename_en(flat_index: usize, label: &str) -> String {
    format!("{:03}_{}.md", position_number(flat_index), slugify(label))
}

/// Return the expected English filename for each TOC entry (excluding 000_toc.md).
/// Vec of (flat_index, filename) pairs.
pub fn all_filenames_en() -> Vec<(usize, String)> {
    toc_en::flat_toc_entries_en()
        .iter()
        .map(|&(idx, _, _, label, _)| (idx, filename_en(idx, label)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_en() {
        assert_eq!(filename_en(0, "Motto"), "001_motto.md");
        assert_eq!(
            filename_en(2, "Preface to the Second Edition"),
            "003_preface_to_the_second_edition.md"
        );
    }

    #[test]
    fn test_all_filenames_en() {
        let fnames = all_filenames_en();
        assert_eq!(fnames.len(), 113);
        assert_eq!(fnames[0], (0, "001_motto.md".to_string()));
        assert_eq!(
            fnames[2],
            (2, "003_preface_to_the_second_edition.md".to_string())
        );
    }
}
