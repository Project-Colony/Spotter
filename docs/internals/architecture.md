# Architecture

Spotter is one Rust binary: an [iced](https://iced.rs) 0.14 application on the
Elm pattern (state, messages, `update`, `view`), a SQLite database through
`rusqlite`, and one blocking `ureq` client per store. The look, the
Preferences page building blocks, the fonts, the en/fr shared labels and the
folder paths come from
[colony-ui](https://github.com/Project-Colony/Project-Colony-Resources/tree/main/crates/colony-ui),
the Project Colony design-system crate.

## Module map

```
src/
├── main.rs            entry point: --version, then spotter::run()
├── lib.rs             the whole module tree; the pub modules are what tests/ uses
├── app.rs             Spotter (the state), Message, update(), view(), run()
├── handlers/          update() split by area: data and persistence, imports,
│                      store logins, preferences
├── messages/          the message enums those areas add to Message
├── views/             one file per screen (library, game detail, statistics,
│                      import, profile, add game, Preferences) plus the sidebar
├── models.rs          Game, Platform, GameStatus, Achievement, UserProfile and
│                      Spotter's own preferences
├── preferences.rs     preferences.json: lenient load, save, import of the 0.3
│                      settings
├── paths.rs           where every file lives (on colony_ui::paths), and the
│                      copy of the 0.3 files
├── i18n/              en.json, fr.json and the t / tf / tn lookup
├── db.rs              SQLite schema, migrations, queries, export and import
├── keyring.rs         the seven secrets, in the OS keyring or the fallback file
├── api_client.rs      HTTP agents, retry and rate limiting, the import log,
│                      URL encoding, opening the browser
├── images.rs          cover and achievement icon download and cache
├── theme.rs           icons, ui_text and the widget styles over the colony-ui
│                      palette
├── error.rs           SpotterError
├── steam/             mod.rs: owned games, store and achievement scraping
│                      auth.rs: OpenID login on 127.0.0.1
├── gog/               mod.rs: library and store metadata; auth.rs: OAuth code exchange
├── epic/              online.rs: account import; local.rs: installed-games scan;
│                      auth.rs, api.rs, graphql.rs and helpers
├── xbox.rs            OpenXBL import
└── playstation.rs     NPSSO exchange, trophy titles, game list
```

`main.rs` declares no module of its own: it calls `spotter::run`, and
everything else hangs off `lib.rs`. Which store endpoints each importer calls is
in [api-integrations.md](api-integrations.md).

## Data flow

```
user input / keyboard
        │
        ▼
     Message ──────► Spotter::update() ──► state change ──► Spotter::view()
        ▲                   │
        │                   ▼
        │        Task<Message> (spawn_task / spawn_write)
        │                   │
        │                   ▼
        └──── result ◄── worker thread: store requests, SQLite, files
```

Every change goes through a `Message`. Views only read `&Spotter` and return
widgets; they never mutate it. Blocking work runs in `app::spawn_task`, which
moves the closure to a new thread and maps its `Result<T, String>` back into a
message:

```rust
fn spawn_task<T, F, M>(f: F, msg: impl FnOnce(Result<T, String>) -> M + Send + 'static) -> Task<M>
```

Writes of saved data go through `Spotter::spawn_write` instead. If the startup
load failed, the app is holding defaults rather than the user's data, so
`spawn_write` refuses every write (preferences, profile, games, exports,
auto-backup) and reports it, and the files on disk stay as they were.

## Screens

```
┌───────────────┐   ┌──────────────────────────────────────────┐
│ [Spotter ⚙]───┼──►│ Preferences (replaces the content area;   │
│               │   │   same button or Close goes back)         │
│ Library ──────┼──►│ Library ──► Game detail                   │
│ Add game ─────┼──►│ Add game                                  │
│ Statistics ───┼──►│ Statistics                                │
│ Import ───────┼──►│ Import (one card per store)               │
│ Profile ──────┼──►│ Profile (name, keyring state, data paths, │
│               │   │   export and import)                      │
│ quick stats   │   └──────────────────────────────────────────┘
└───────────────┘
```

The sidebar stays on every screen. Keyboard: `Esc` cancels a pending delete,
or else closes Preferences, or else goes back to the library; `Ctrl+F` focuses
the search, and `Ctrl+A` selects every game the library
currently shows (`Cmd` instead of `Ctrl` on macOS).

## Import merge

An import returns a `Vec<Game>`, and `merge_imported_games`
(`handlers/data.rs`) folds it into the library. Each incoming game is matched,
in this order, on:

1. its store id (`steam_appid`, `gog_id`, `epic_id`, `xbox_id`, `psn_id`);
2. its title and platform;
3. its title alone.

A matched game gets the incoming genre, description, release date, tags, cover
address and review score when the import has them, the achievement counts when
the import has any, and the larger of the two playtimes; store ids it lacked
are filled in. Status, rating, notes and last-played date are never touched.
Unmatched games are added, and arrive in the library one by one (the import
cascade) unless reduced motion is on.

After an import Spotter saves the library, writes `spotter_auto_backup.json`,
and, unless turned off, downloads the missing covers.

## Look and preferences

Colours come from the active colony-ui theme, never from literals: the
`colony_ui::theme::Palette` roles (`Palette::TEXT_MUTED()`, `Palette::BG_CARD()`
...), read wherever a widget is drawn, including inside style closures, and
`contrast_on` for text on a filled button. Platforms and game statuses keep
one colour on every theme: `Platform::accent_key` and `GameStatus::accent_key`
name a shared accent swatch, which `theme::shared_accent` turns into a colour.
The style helpers (cards, chips, buttons, pills) are in `theme.rs`.

Text goes through `theme::ui_text(content, base)`, which sizes it with
`colony_ui::sz(base)`, so the Appearance and Accessibility text sizes both
apply, sets it in the interface font, so the dyslexia font reaches it, and
keeps the Nerd Font glyphs inside it in the glyph font. Animations, the import
cascade included, check `colony_ui::effects_enabled()`, which is false when
**Animations** is off or reduced motion is on.

The preferences file is Spotter's own struct with colony-ui's
`StandardPreferences` flattened into it. At startup Spotter loads it, calls
`StandardPreferences::apply()` and sets the locale; a change on the Preferences
page goes through `StandardPreferences::update()` and is saved at once. Text is
looked up by key (`i18n::t`, `tf`, `tn`) in Spotter's own en/fr tables
(`src/i18n/en.json`, `src/i18n/fr.json`), then in colony-ui's shared labels;
tests keep the two locales' keys and placeholders identical.

## Credentials

`keyring.rs` sits between the profile and the database. On load it reads the
seven secret fields from the keyring; on save it writes them there and keeps
their database columns empty. The first keyring failure in a session switches
it to the fallback file `<config>/credentials/tokens.json` for the rest of the
session, and the Profile page shows a warning. If that file cannot be written
either, the secrets stay in their database columns, unencrypted, and the next
start moves them out again. On load, secrets still in the database (Spotter
0.3 and older) move to the keyring, or to the file without one, and secrets in
the file move to the keyring once it works again. It only deletes keyring entries it has
read or written this session, because a locked store can answer "no entry"
for a secret it does hold.

## Dependencies

| Crate | Role |
|---|---|
| `iced` (with `image`) | GUI |
| `colony-ui` | themes, typography, fonts, Preferences widgets, shared labels, folder paths |
| `rusqlite` (bundled) | SQLite, compiled in; no system library needed |
| `dirs` | the home folder, for the Epic Games local scan |
| `ureq` (with `json`) | blocking HTTP |
| `scraper` | Steam store and achievement pages |
| `keyring`, `keyring-core` | OS keyring, and its mock store in tests |
| `serde`, `serde_json`, `csv` | JSON and CSV |
| `chrono` | dates |
| `thiserror` | `SpotterError` |
| `winresource` (build) | the Windows icon and version resource |

Every dependency names its full version in `Cargo.toml`, and `Cargo.lock` is
committed; CI builds with `--locked`.

The release profile uses thin LTO, one codegen unit, `opt-level = 3`, and strips
symbols.
