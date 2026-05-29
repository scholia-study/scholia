//! Business domains. Each domain owns its handlers, db queries, and DTOs,
//! and exposes tier-scoped router constructors (`public_router`,
//! `user_router`, `editor_router`, `admin_router`, `rate_limited_router`)
//! consumed by `crate::api_router`. Cross-domain access goes only through a
//! domain's public facade (its `mod.rs` re-exports), never its internals.

pub mod billing;
pub mod corpus;
pub mod feedback;
pub mod identity;
pub mod writing;
