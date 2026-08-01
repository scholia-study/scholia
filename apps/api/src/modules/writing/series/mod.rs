//! Editor-curated, ordered collections of published articles (the
//! Scholia Blog is one). Membership is prepend-on-add and displayed
//! position-ascending everywhere; see `db/migrations/0020_series.sql`.

pub mod db;
pub mod handlers;
pub mod models;
