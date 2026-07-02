pub mod corpus;
pub mod parse;

// The struct schema and md→html helpers are shared, genre-agnostic infra; the
// verse-specific parsing (`corpus`, `parse`) lives here. Re-exported so internal
// `crate::model` / `crate::html` paths keep resolving.
pub use text_struct::{html, model};
