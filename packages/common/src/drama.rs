//! The node shape every drama corpus declares. `md_drama_to_struct` maps these
//! to its own `NodeSpec`; this side is pure data, so a new drama corpus is a
//! `common::<corpus>` module and nothing else.

/// One curated file. `label` is deliberately absent — it comes from the file's
/// front matter at parse time, which keeps the markdown the single source of
/// truth for what a node is called.
pub struct Node {
    pub source_ref: String,
    pub slug: String,
    pub path: String,
    pub depth: i16,
    pub parent_source_ref: Option<String>,
    pub filename: String,
    pub position: u32,
}
