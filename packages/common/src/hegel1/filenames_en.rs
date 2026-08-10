//! English filenames for the translated layer, kant1-style: `NNN_slug.md`
//! with `NNN` the node's document position and the slug derived from the
//! English label. The position number is the stable identity shared with
//! the German files.

use super::{filenames, toc, toc_en};

pub fn filename_en(flat_index: usize) -> String {
    format!(
        "{:03}_{}.md",
        toc::entries()[flat_index].position,
        filenames::slugify(toc_en::LABELS_EN[flat_index])
    )
}

/// The expected English filename for every TOC entry, as (flat_index, filename).
pub fn all_filenames_en() -> Vec<(usize, String)> {
    (0..toc::toc_len()).map(|i| (i, filename_en(i))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filenames_are_unique_and_positioned() {
        let all = all_filenames_en();
        assert_eq!(all.len(), 50);
        let set: std::collections::HashSet<_> = all.iter().map(|(_, f)| f).collect();
        assert_eq!(set.len(), 50);
        assert!(all[0].1.starts_with("001_"));
    }
}
