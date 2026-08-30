//! Shared reconciling-re-import toolkit, used by every importer that updates an
//! already-imported book in place (`struct_to_db`, `bible_to_db`).
//!
//! Six book-agnostic pieces live here:
//! - [`align`]: the text-based aligner that classifies a unit's before/after
//!   sentence lists into update / split / merge / insert / delete (or aborts on
//!   ambiguity). It has no notion of paragraphs, verses, or footnotes — callers
//!   feed it one "unit" (a paragraph, a footnote, a verse) at a time.
//! - [`deps`]: the dependent-migration helpers that keep user/editor data
//!   (quotations, resources, cross-references) and footnote/translation links
//!   attached to the right sentence when one is merged away or split. This logic
//!   is data-integrity-critical, so it lives in exactly one place.
//! - [`hash`]: the per-node + root content hashing that drives the incremental
//!   reconcile (callers build the hash-input from their model).
//! - [`insertion`]: the opt-in mid-book insertion pre-alignment for books that
//!   grow (`--allow-insertion`) — renumbers existing book-global sequences to
//!   the desired values so the strictly-additive pre-flight then passes.
//! - [`keys`]: the sentence `natural_key` formats, so fresh-insert and reconcile
//!   build the same per-sentence identity strings.
//! - [`orchestrate`]: the full in-place reconcile, reading from the owned
//!   [`orchestrate::ReconcileInput`] IR. Importers map their model → the IR,
//!   compute their hashes themselves, then call [`orchestrate::reconcile_book`].

pub mod align;
pub mod deps;
pub mod hash;
pub mod insertion;
pub mod keys;
pub mod orchestrate;

pub use align::{BlockPlan, Existing, plan_block};
pub use deps::{extend_anchors_to, migrate_dependents, sentence_has_dependents};
pub use hash::{
    BlockContent, FootnoteContent, MarginNoteContent, MarkerContent, NodeContent, SentenceContent,
    node_hash, root_hash,
};
pub use keys::{footnote_natural_key, margin_note_natural_key, natural_key};
pub use orchestrate::{
    BlockInput, FootnoteInput, MarginNoteInput, MarkerInput, NodeAnchor, NodeInput, ReconcileInput,
    ReconcileReport, SentenceInput, reconcile_book,
};
