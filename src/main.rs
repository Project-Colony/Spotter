#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() -> iced::Result {
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!("spotter {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    spotter::run()
}
