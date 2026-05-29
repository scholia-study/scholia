//! Cross-cutting infrastructure shared across business modules:
//! authentication plumbing, configuration, persistence/session state,
//! email, error types, rate limiting, request validation, and the
//! migration runner. Business logic lives under `crate::modules`; this
//! tree is the substrate it sits on.

pub mod auth;
pub mod cache;
pub mod config;
pub mod email;
pub mod error;
pub mod migrate;
pub mod rate_limit;
pub mod state;
pub mod validation;
