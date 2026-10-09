# Spotter

A cross-platform game library tracker and statistics dashboard built with [Rust](https://www.rust-lang.org/) and [Iced](https://iced.rs).

## Features

- **Multi-platform imports** - Steam, GOG, Epic Games, Xbox (via OpenXBL), and PlayStation Network
- **Achievement tracking** - view and track achievements with icons, descriptions, and unlock dates
- **Statistics dashboard** - playtime charts, status/platform distribution, most played games
- **Customizable UI** - dark/darker/midnight themes, accent colors, UI scaling, sidebar width, compact mode
- **Accessibility** - high contrast mode, large click targets, status label toggles
- **Settings** - date format, notification preferences, toast duration, start screen, default status/platform
- **Local-first** - all data stored in a local SQLite database, no cloud dependency

## Running the binaries

The easiest way to install Spotter is through [Colony](https://github.com/Project-Colony/Colony), which downloads the right binary and checks its signature for you.

To run a release by hand, download the asset for your system from the [latest release](https://github.com/Project-Colony/Spotter/releases/latest):

| Asset | System |
|---|---|
| `spotter-linux` | Linux x86_64 |
| `spotter-windows.exe` | Windows x86_64 |
| `spotter-macos-arm` | macOS on Apple silicon |
| `spotter-macos-x86` | macOS on Intel |

Each asset has a matching `.sig` file, an ed25519 signature made with the Project-Colony release key, and a signed `.meta` file binding it to its version.

- **Linux**: `chmod +x spotter-linux && ./spotter-linux`. It needs a Wayland or X11 session and libxkbcommon, which every desktop distribution ships.
- **Windows**: run `spotter-windows.exe`. Until it carries an Authenticode signature (see [Code signing policy](#code-signing-policy)), SmartScreen may ask you to confirm with **More info** then **Run anyway**.
- **macOS**: `chmod +x spotter-macos-arm && xattr -d com.apple.quarantine spotter-macos-arm && ./spotter-macos-arm` (use `spotter-macos-x86` on Intel). The binary is not notarized, so Gatekeeper blocks it until the quarantine attribute is removed.

`spotter --version` prints the version and exits without opening a window.

## Building

Requires Rust 1.90+ (2021 edition).

```bash
cargo build --release
```

The binary will be at `target/release/spotter`.

## Running

```bash
cargo run
```

On first launch, Spotter creates a sample library. To import your own games, go to **Import** and configure your platform credentials in **Profile**.

### Platform Setup

| Platform | Credential needed | Where to get it |
|---|---|---|
| Steam | API Key + Steam ID | [steamcommunity.com/dev/apikey](https://steamcommunity.com/dev/apikey) - or use the "Login with Steam" button |
| GOG | OAuth token | GOG Galaxy client settings |
| Epic | - | Auto-scans local Epic Games Launcher |
| Xbox | OpenXBL API key | [xbl.io](https://xbl.io) |
| PlayStation | NPSSO token | Browser cookies at [store.playstation.com](https://store.playstation.com) |
| Nintendo | - | Manual entry (no import API) |

## Testing

```bash
cargo test -- --test-threads=1
```

Single-threaded execution is required because database tests use shared environment state (`XDG_DATA_HOME`).

## Project Structure

```
src/
  main.rs          Entry point
  lib.rs           Public module exports for tests
  app.rs           Application state, messages, update loop
  db.rs            SQLite database operations
  keyring.rs       Platform tokens in the OS keyring
  models.rs        Data structures (Game, Platform, Settings, etc.)
  theme.rs         Color constants and ViewTheme
  steam.rs         Steam import + HTML scraping
  steam_auth.rs    Steam browser login flow
  gog.rs           GOG Galaxy import
  epic.rs          Epic Games Launcher scanner
  xbox.rs          Xbox Live import via OpenXBL
  playstation.rs   PSN import via trophy API
  images.rs        Cover art + achievement icon caching
  views/           UI views (library, detail, stats, settings, etc.)
tests/
  unit_tests.rs    Unit + integration tests
```

## Data Storage

All data is stored locally:
- **Database**: `~/.local/share/spotter/spotter.db` (Linux) or equivalent `dirs::data_dir()`
- **Covers**: `~/.local/share/spotter/covers/`
- **Achievement icons**: `~/.local/share/spotter/achievement_icons/`
- **Settings**: Stored as JSON inside the SQLite database
- **Platform credentials** (API keys, login tokens): in the operating system's keyring (Windows Credential Manager, macOS Keychain, the Secret Service on Linux). Without one, they stay unencrypted in the database and the Profile page says so.

## Code signing policy

Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

Windows builds are signed through SignPath once the SignPath Foundation has accepted the project; until then they ship without Authenticode. Every release asset, on every platform, is always signed with the Project-Colony organisation's ed25519 key, which Colony verifies before installing it.

Team roles and members:

- Committers and reviewers: [MotherSphere](https://github.com/MotherSphere)
- Approvers: [MotherSphere](https://github.com/MotherSphere)

### Privacy policy

Spotter has no telemetry, no analytics and no update check, and sends nothing to its developers. Your library stays in the local database. Your platform credentials (the API keys you enter and the login tokens Spotter receives) are kept in your operating system's keyring (Windows Credential Manager, macOS Keychain, or the Secret Service on Linux, such as GNOME Keyring or KWallet); when no keyring is available, Spotter keeps them unencrypted in the local database instead, and says so on its Profile page. Spotter only contacts the services below, for the features that need them:

- **Imports, when you start one** from the Import screen. Spotter contacts the platform you picked and sends it the credentials you entered for it, plus the ids of your games:
  - Steam: `api.steampowered.com` (your Web API key and Steam ID, to list owned games), `steamcommunity.com` (your Steam ID, to read achievements from your profile) and `store.steampowered.com` (game details).
  - GOG: `auth.gog.com` (the login code you paste, exchanged for a token), then `embed.gog.com` and `api.gog.com` with that token.
  - Epic Games: `account-public-service-prod.ol.epicgames.com` (the login code you paste, exchanged for a token), then Epic's library, catalog, entitlement and launcher services and `store.epicgames.com` with that token and your account id. The Epic Games Launcher scan reads your installed games from disk, then asks Epic's store for their details.
  - Xbox: `xbl.io` (your OpenXBL API key).
  - PlayStation: `ca.account.sony.com` (your NPSSO token, exchanged for an access token), then `m.np.playstation.com` with that token.
- **Steam login**, when you use it: Spotter opens Steam's sign-in page in your browser, receives the answer on a local port of your own computer (127.0.0.1) and asks `steamcommunity.com` to confirm it.
- **Cover art and achievement icons**, downloaded from the image addresses stored with each game, which come from those platforms (or from a JSON backup you import): covers after an import, unless you turn off automatic cover downloads in Settings; achievement icons when you open a Steam game's page.
- **At startup**, if you have connected GOG or Epic Games, Spotter renews that login automatically by sending the saved refresh token to `auth.gog.com` or `account-public-service-prod.ol.epicgames.com`.

Links such as a game's store page open in your browser; Spotter itself sends nothing for them.

## License

GPL-3.0-or-later - see [LICENSE](LICENSE).
