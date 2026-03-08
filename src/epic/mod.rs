mod api;
pub mod auth;
mod filters;
mod formatting;
mod graphql;
mod local;
mod online;
mod types;

// Re-export the public API (unchanged from the original epic.rs)
pub use filters::is_epic_junk_title;
pub use local::full_import_local;
pub use online::full_import_online;
