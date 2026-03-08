mod api_client;
mod app;
mod db;
mod epic;
mod error;
mod gog;
mod handlers;
mod images;
mod keyring;
mod messages;
mod models;
mod playstation;
mod steam;
mod theme;
mod views;
mod xbox;

fn main() -> iced::Result {
    app::run()
}
