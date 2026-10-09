// Windows version resource: SignPath only signs an .exe whose ProductName and
// ProductVersion match the project, and Explorer shows them under Details.
// The icon resource gives the .exe its icon in Explorer and on shortcuts.
// Decided on the target, not cfg!(windows): build scripts run on the host.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let version = env!("CARGO_PKG_VERSION");
    winresource::WindowsResource::new()
        .set_icon("assets/icons/icon.ico")
        .set("ProductName", "Spotter")
        .set("FileDescription", "Spotter")
        .set("ProductVersion", version)
        .set("FileVersion", version)
        .compile()
        .expect("failed to embed the Windows icon and version resource");
}
