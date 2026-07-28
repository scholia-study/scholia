//! Article review workflow: authors submit an article (draft or published)
//! to the editorial team for feedback or as a publication hand-off. Each
//! request freezes a sentence-annotated snapshot that editor comments anchor
//! into; a per-article message channel is shared across rounds. Approving a
//! publication request publishes the article (if needed) and applies the
//! `imprimatur` editorial label.

pub mod db;
pub mod handlers;
pub mod models;
pub mod snapshot;
