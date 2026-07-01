//! Curated two-layer drama markdown → struct JSON. The drama markup
//! (`@ speaker`, `@stage`, `| verse`, `*(…)*`, `{{{ N }}}`) is tokenised into
//! the shared `text_struct` schema (`block_type` is a free string, so the new
//! `speaker`/`stage` block types need no struct change), then imported by the
//! reused `struct_to_db`. See `PLAN_DRAMA.md` and ADR 0005.

pub mod corpus;
pub mod markers;
pub mod parse;
