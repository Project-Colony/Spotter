# Architecture

Spotter is a cross-platform game library tracker built with **Rust** and the **Iced** GUI framework. It uses **SQLite** for local persistence and supports importing games from 5 platforms.

## Module Map

```
main.rs                 Entry point → app::run()
app.rs (1300+ lines)    State machine: Spotter struct, Message enum, update(), view()
├── views/
│   ├── library.rs      Game list with search, filters, sort
│   ├── detail.rs       Single game: metadata, achievements, rating, notes
│   ├── stats.rs        Statistics dashboard with charts
│   ├── settings.rs     Three-tab preferences (General / Appearances / Accessibility)
│   ├── profile.rs      User profile, credentials, data export/import
│   ├── import.rs       Platform import cards with step-by-step flows
│   ├── sidebar.rs      Navigation + quick stats
│   └── add_game.rs     Manual game entry form
├── models.rs           Data structures: Game, Settings, Platform, Achievement, etc.
├── db.rs               SQLite: schema, CRUD, playtime history, export/import
├── theme.rs            Color constants, ViewTheme, style helpers
├── error.rs            SpotterError enum (migration in progress)
├── api_client.rs       HTTP agents, retry logic, URL encoding, browser open
├── images.rs           Cover art + achievement icon download/caching
├── steam.rs            Steam import (Web API + HTML scraping)
├── steam_auth.rs       Steam OpenID login (localhost:29876)
├── gog.rs              GOG import (OAuth API)
├── gog_auth.rs         GOG OAuth 2.0 token exchange
├── epic.rs             Epic Games local manifest scanner
├── xbox.rs             Xbox Live import (OpenXBL API)
└── playstation.rs      PSN import (OAuth + trophy API)
```

## Data Flow

```
User Interaction
       │
       ▼
   Message enum (100+ variants)
       │
       ▼
   app::update()  ──→  Task<Message> (async operations)
       │                      │
       ▼                      ▼
   State mutation        Background thread
   (Spotter fields)      (API calls, DB writes)
       │                      │
       ▼                      ▼
   app::view()           Result callback
   (renders UI)          (re-enters Message loop)
```

All state mutations pass through the `Message` enum in `app.rs`. Async operations (API calls, DB writes) are spawned as `Task<Message>` and return results via callback messages.

## Screen Navigation

```
┌──────────────┐    ┌────────────────────────────────────┐
│   Sidebar    │    │         Main Content Area           │
│              │    │                                     │
│  Library ────┼───►│  Library (search, filter, game list)│
│  + Add Game ─┼───►│  AddGame (form)                    │
│  Statistics ─┼───►│  Statistics (charts, distributions) │
│  Import ─────┼───►│  Import (platform cards)            │
│  Profile ────┼───►│  Profile (credentials, export)      │
│              │    │                                     │
│  ── Quick ── │    │  GameDetail (from library click)    │
│  Playing: X  │    │  Settings (from sidebar logo)       │
│  Completed: Y│    │                                     │
│  Unplayed: Z │    └────────────────────────────────────┘
│              │
│  [Username]  │
└──────────────┘
```

## Key Architectural Patterns

### Message-Driven Updates

Every user interaction produces a `Message` variant. The `update()` function pattern-matches on it, mutates state, and optionally returns an async `Task<Message>`.

```rust
// Example flow: rating a game
Message::SetGameRating(game_id, Some(8))
  → find game by ID, set rating
  → spawn Task: save to DB
  → on completion: Message::DataSaved(Result)
  → show toast notification
```

### Async via Thread Spawning

Iced uses a `Task` abstraction. Spotter wraps blocking operations in `std::thread::spawn`:

```rust
fn spawn_task<T, F, M>(f: F, msg: fn(Result<T, String>) -> M) -> Task<M>
```

This runs `f` on a new thread and maps the result through `msg`.

### Import Merge Logic

When importing games from a platform, deduplication uses a multi-key index:

1. **Platform ID match** (e.g., `steam_appid`) — most reliable
2. **Title + Platform match** — for games without IDs
3. **Title-only match** — last resort

Existing user data (ratings, notes, status) is always preserved. Only empty metadata fields are backfilled from imports.

### Theme System

Two layers:
- **Static constants** (`theme.rs`): Used in style closures where `ViewTheme` isn't available
- **ViewTheme struct**: Computed per-render from `Settings`, provides theme-mode-aware colors

```rust
let vt = ViewTheme::from_settings(&app.settings);
// vt.bg_card, vt.text_muted, vt.border, etc.
```

## Dependencies

| Crate | Version | Role |
|-------|---------|------|
| iced | 0.13 | GUI framework (with `image` feature) |
| rusqlite | 0.32 | SQLite (bundled, zero system deps) |
| ureq | 2 | Blocking HTTP client |
| serde / serde_json | 1 | JSON serialization |
| chrono | 0.4 | Date/time formatting |
| csv | 1 | CSV export |
| thiserror | 1 | Error derive macros |
| dirs | 5 | Platform-specific directories |

## Build Profiles

```toml
[profile.release]
lto = "thin"       # Link-time optimization
strip = true       # Strip debug symbols
opt-level = 3      # Maximum optimization
```
