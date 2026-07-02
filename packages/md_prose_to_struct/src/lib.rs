//! Shared annotated-prose parser (the Kant-family genre): curated markdown
//! with footnotes, figures, separators, indented runs, and dual page-marker
//! systems → struct JSON. `corpus` selects the text (kant1 | kant3, source or
//! translation edition); the grammar (`parse`, `html`, `figure`, `separator`,
//! `roman`) and the tree builder (`structure`) are shared. A new prose corpus
//! = a new `common::<corpus>` module + a corpus builder arm — never a new
//! parser.

pub mod corpus;
pub mod figure;
pub mod html;
pub mod parse;
pub mod roman;
pub mod separator;
pub mod structure;

// The struct schema is the shared, genre-agnostic waist of the pipeline.
pub use text_struct::model;
