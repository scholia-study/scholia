/// One flattened TOC row shared by the prose corpora:
/// `(flat_index, page, depth, label, slug_override)`. The page is the corpus
/// page key's value as a string — kant's numeric AA page, hegel1's
/// Roman-or-Arabic 1807 page — `None` for a node opening before the first
/// numbered page.
pub type FlatTocEntry = (
    usize,
    Option<String>,
    u16,
    &'static str,
    Option<&'static str>,
);

pub mod content;
pub mod epub_reader;
pub mod hegel1;
pub mod hegel2;
pub mod hegel3;
pub mod hobbes1;
pub mod ibsen1;
pub mod kant1;
pub mod kant3;
pub mod milton1;
pub mod model;
pub mod ncx;
pub mod opf;
pub mod sentences;
pub mod shakespeare1;
pub mod textmatch;
