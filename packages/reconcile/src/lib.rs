//! Shared reconciling-re-import toolkit, used by every importer that updates an
//! already-imported book in place (`kant1_struct_to_db`, `bible_to_db`).
//!
//! Two book-agnostic pieces live here:
//! - [`align`]: the text-based aligner that classifies a unit's before/after
//!   sentence lists into update / split / merge / insert / delete (or aborts on
//!   ambiguity). It has no notion of paragraphs, verses, or footnotes — callers
//!   feed it one "unit" (a paragraph, a footnote, a verse) at a time.
//! - [`deps`]: the dependent-migration helpers that keep user/editor data
//!   (quotations, resources, cross-references) and footnote/translation links
//!   attached to the right sentence when one is merged away or split. This logic
//!   is data-integrity-critical, so it lives in exactly one place.

pub mod align;
pub mod deps;

pub use align::{BlockPlan, Existing, plan_block};
pub use deps::{extend_anchors_to, migrate_dependents, sentence_has_dependents};
