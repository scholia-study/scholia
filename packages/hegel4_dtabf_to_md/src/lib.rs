//! Digitale-Hegel-Edition DTA-Basisformat TEI (`GW20.xml`) → the curated
//! hegel4 `md_reviewed` layer: Hegel's *Enzyklopädie* (1830), GW 20.
//!
//! The XML is the primary source. The edition's JSON (`work-gw-20.js`) is
//! consulted only as an oracle for line-end hyphens (its paragraph text is
//! already dehyphenated); it is not an independent witness.
//!
//! Output is one markdown file per `common::hegel4::toc` node. Every § is a
//! `##` heading line carrying its `{{ N }}` marker, each Remark run is a
//! `::: remark` fence opened by `{{ N A }}`, GW page starts are `{{{ 20.N }}}`.
//! The converter refuses to emit on anything it was not written to handle:
//! unknown elements, unanchored or ambiguous TOC entries, unsettled hyphen
//! joins, non-sequential §§.

pub mod assemble;
pub mod hyphen;
pub mod indent_runs;
pub mod modernize;
pub mod page_joins;
pub mod render;
pub mod source;

pub type Result<T> = std::result::Result<T, String>;
