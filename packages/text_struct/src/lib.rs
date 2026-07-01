//! Shared struct-JSON schema (`model`) and curated-markdown → HTML helpers
//! (`html`) for the structured-text ingest pipeline. Genre-agnostic: the poetry
//! parser (`poetry_md_to_struct`), the drama parser (`drama_md_to_struct`), and
//! the importer (`struct_to_db`) all build on this so their JSON is
//! byte-compatible end to end.

pub mod html;
pub mod model;
