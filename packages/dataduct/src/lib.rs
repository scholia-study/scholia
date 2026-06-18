//! Shared ingest/serve plumbing: Postgres connect-option resolution, cache
//! PURGE helpers, and seed-import constants. Extracted so the ingest binaries
//! and the API share one copy instead of each maintaining its own.

pub mod cache;
pub mod db;
pub mod seed;
