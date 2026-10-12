# Development Guide

## Prerequisites

- **Rust** 1.90+ (2021 edition)
- SQLite is bundled via `rusqlite`, nothing to install for it
- On Linux, the windowing and keyring development packages (Debian/Ubuntu names):
  `pkg-config libxkbcommon-dev libxkbcommon-x11-dev libxcb1-dev libxcb-xkb-dev libwayland-dev libdbus-1-dev libsecret-1-dev`

## Build

```bash
# Debug build
cargo build

# Release build (optimized, stripped)
cargo build --release
```

The release binary is at `target/release/spotter`.

## Run

```bash
cargo run
```

On first launch, Spotter creates its data directories and an empty SQLite database.

## Test

```bash
# Run all tests
cargo test
```

Tests are in `tests/unit_tests.rs` and `tests/integration_tests.rs`. Each DB test creates an isolated temporary directory with its own database via `db::open_at()`, so tests can run in parallel.

### Test Categories

| Category | Count | What's tested |
|----------|-------|---------------|
| Playtime display | 3 | Format hours/minutes strings |
| Models | 20+ | Enum roundtrips, Display impls, distinct platform and status colours, settings serialization |
| Database | 8 | CRUD, batch save, achievements, playtime history, export/import, profile, settings |
| API client | 3 | URL encode/decode, roundtrip |
| Error | 2 | Display format, String conversion |

### Coverage Gaps

The following areas lack test coverage:
- HTML scraping (Steam store pages)
- Import merge/deduplication logic
- OAuth flows (Steam, GOG, PSN)
- View rendering (Iced widget testing is limited)
- API client retry behavior

## Project Structure

```
src/
├── main.rs              # Entry point
├── lib.rs               # Module tree; public API for integration tests
├── app.rs               # Core state machine (~1300 lines)
├── models.rs            # All data structures (~800 lines)
├── i18n/                # en.json, fr.json and the t/tf/tn lookup
├── db.rs                # SQLite operations
├── paths.rs             # File locations, copy from Spotter 0.3
├── preferences.rs       # preferences.json load/save/import
├── theme.rs             # Icons and style helpers on the Colony palette
├── error.rs             # SpotterError enum
├── api_client.rs        # HTTP utilities
├── images.rs            # Cover/icon download + caching
├── steam.rs             # Steam import
├── steam_auth.rs        # Steam OpenID
├── gog.rs               # GOG import
├── gog_auth.rs          # GOG OAuth
├── epic.rs              # Epic local scanner
├── xbox.rs              # Xbox OpenXBL import
├── playstation.rs       # PSN import
└── views/
    ├── mod.rs           # Module declarations
    ├── library.rs       # Game list view
    ├── detail.rs        # Game detail view
    ├── stats.rs         # Statistics dashboard
    ├── preferences.rs   # Preferences page
    ├── profile.rs       # Profile & data management
    ├── import.rs        # Import flows
    ├── sidebar.rs       # Navigation sidebar
    └── add_game.rs      # Manual game entry

tests/
└── unit_tests.rs        # All tests (40 tests)

docs/
├── ARCHITECTURE.md      # Architecture overview
├── DATA_MODEL.md        # Data model & DB schema
├── API_INTEGRATIONS.md  # External API documentation
├── CONFIGURATION.md     # Preferences, languages and theming
└── DEVELOPMENT.md       # This file
```

## Key Conventions

### Error Handling

The codebase currently uses `Result<T, String>` for most operations. A typed `SpotterError` enum exists in `error.rs` for gradual migration. New code should prefer `SpotterError` where possible.

### State Mutations

All state changes go through the `Message` enum in `app.rs`. Never mutate `Spotter` fields directly from views. Views are pure rendering functions.

### Async Operations

Use `spawn_task()` for background work:

```rust
fn spawn_task<T, F, M>(f: F, msg: fn(Result<T, String>) -> M) -> Task<M>
```

This spawns a thread and maps the result through a `Message` variant.

### Database Access

Always open a fresh connection per operation via `db::open()`. The connection uses WAL mode for concurrent reads.

### Styling

Take every colour from the active palette (`colony_ui::theme::Palette::*()`), in view code and in style closures alike, and the style helpers in `theme.rs`. Never write a colour literal: it would ignore the user's theme and high-contrast setting.

Write text with `theme::ui_text(content, base)` rather than `text(..).size(N)`: it sizes the text with `colony_ui::sz(base)`, so both text-size preferences scale it, sets it in the interface font, so the dyslexia font reaches it, and keeps the Nerd Font glyphs inside it in the glyph font. A text input takes `.size(sz(N)).font(ui_font())`.

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `XDG_CONFIG_HOME` | Override config directory (Linux). |

## Common Warnings

The following `dead_code` warnings are expected and can be ignored:

- `save_single_game`: utility function reserved for future single-game save
- `export_games_json` / `export_games_csv`: called via message dispatch, not detected by static analysis
- `error::Result`: alias for future `SpotterError` migration
- Various `GameBuilder` methods: available for test convenience
