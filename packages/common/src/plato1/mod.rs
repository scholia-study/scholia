//! Canonical data for plato1 — Plato's *Republic* (Πολιτεία) in John Burnet's
//! critical text, with an English translation edition locked 1:1 to it.
//!
//! The ten books are the work's own divisions. The depth-1 divisions beneath
//! them are **Scholia's**: Burnet prints no chapter divisions and the Perseus
//! TEI carries none, so the boundaries and their English titles are editorial
//! apparatus and are disclosed as such in the about text (see ADR/spec DEC-5).

pub mod filenames;
pub mod meta;
pub mod toc;
