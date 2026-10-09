// The whole module tree lives here; main.rs only calls `run`.
// The public modules are what the integration tests use.
pub mod api_client;
pub mod db;
pub mod error;
pub mod keyring;
pub mod models;
pub mod theme;

mod app;
mod epic;
mod gog;
mod handlers;
mod images;
mod messages;
mod playstation;
mod steam;
mod views;
mod xbox;

pub use app::run;
