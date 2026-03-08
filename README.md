# Spotter

A cross-platform game library tracker and statistics dashboard built with [Rust](https://www.rust-lang.org/) and [Iced](https://iced.rs).

## Features

- **Multi-platform imports** — Steam, GOG, Epic Games, Xbox (via OpenXBL), and PlayStation Network
- **Achievement tracking** — view and track achievements with icons, descriptions, and unlock dates
- **Statistics dashboard** — playtime charts, status/platform distribution, most played games
- **Customizable UI** — dark/darker/midnight themes, accent colors, UI scaling, sidebar width, compact mode
- **Accessibility** — high contrast mode, large click targets, status label toggles
- **Settings** — date format, notification preferences, toast duration, start screen, default status/platform
- **Local-first** — all data stored in a local SQLite database, no cloud dependency

## Building

Requires Rust 1.75+ (2021 edition).

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
| Steam | API Key + Steam ID | [steamcommunity.com/dev/apikey](https://steamcommunity.com/dev/apikey) — or use the "Login with Steam" button |
| GOG | OAuth token | GOG Galaxy client settings |
| Epic | — | Auto-scans local Epic Games Launcher |
| Xbox | OpenXBL API key | [xbl.io](https://xbl.io) |
| PlayStation | NPSSO token | Browser cookies at [store.playstation.com](https://store.playstation.com) |
| Nintendo | — | Manual entry (no import API) |

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

## License

MIT
