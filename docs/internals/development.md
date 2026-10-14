# Development

## Prerequisites

- **Rust 1.90** or newer (`rust-version` in `Cargo.toml`; CI builds and tests on
  exactly 1.90 too).
- Nothing for SQLite: `rusqlite` compiles it in.
- On Linux, the windowing and keyring development packages. With Debian or
  Ubuntu names, as CI installs them:

  ```bash
  sudo apt-get install pkg-config libxkbcommon-dev libxkbcommon-x11-dev \
    libxcb1-dev libxcb-xkb-dev libwayland-dev libdbus-1-dev libsecret-1-dev
  ```

## Build and run

```bash
cargo build --locked            # debug build
cargo run --locked              # opens the window
cargo run --locked -- --version # prints "spotter <version>" and exits
cargo build --locked --release  # optimised and stripped: target/release/spotter
```

The first start creates Spotter's folders and an empty database; the folders
are listed in [data-model.md](data-model.md#where-the-data-lives). A debug
build uses the same folders as an installed Spotter, so it opens your real
library: back it up first, or work under a test user.

## Test

```bash
cargo test --locked
```

The tests run in parallel. Each database test opens its own SQLite file in a
fresh temporary directory through `db::open_at`, and the keyring tests use
`keyring-core`'s mock store, so nothing touches your library or your keyring.

| Where | What it covers |
|---|---|
| `tests/unit_tests.rs` | models and enums, preferences parsing and versioning, the database (CRUD, batch save, achievements, playtime history, profile, export and import), CSV and JSON export, errors |
| `tests/integration_tests.rs` | each importer's parser against recorded Steam, PlayStation, Xbox, GOG and Epic responses, image validation, the rate limiter, the import log, URL encoding |
| `#[cfg(test)]` modules in `src/` | Steam HTML scraping, multi-byte title and description cuts, keyring moves, the write refusal after a failed load, the folders and their migration, `preferences.json` and the import of 0.3 settings, the en/fr tables, the Preferences toggle and `Esc`, imports without effects, the window icon |

Not covered by automated tests: live store requests (the importers are only
tested against recorded responses), the login flows end to end, the import
merge in `handlers/data.rs`, and rendering.
A change to a view, the Preferences page or the French layout needs a look on a
real desktop at both ends of the text scale (0.7225x and 1.68x).

## Before a pull request

CI runs these on Linux, Windows and macOS with the latest stable Rust, then
builds and tests again on exactly Rust 1.90:

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked
cargo test --locked
cargo audit
```

On Windows it also checks that the executable's version resource says
`ProductName` Spotter and the crate's version, which code signing requires.
Pull request titles are conventional commits (`feat:`, `fix:`, `docs:` ...),
because they become the squash commit and the changelog entry. Releases are cut
by release-please; see the
[code signing policy](../../README.md#code-signing-policy) for what a release
carries.

## Conventions

**State changes.** Everything goes through a `Message` and `Spotter::update`;
views only read `&Spotter`. Add a variant to the area's message enum in
`messages/` and handle it in the matching file in `handlers/`.

**Background work.** Wrap blocking work in `app::spawn_task`, and anything that
writes saved data in `Spotter::spawn_write`, which refuses the write after a
failed startup load.

**Database.** Open a connection per operation with `db::open()`; WAL mode lets
reads run alongside a write. A new column goes into both `init_tables` and the
`migrate` list in `db.rs`, so existing databases get it.

**Secrets.** A new credential is a `UserProfile` field listed in
`keyring::secrets`, so it goes to the keyring and never into the database.

**Look.** Take every colour from the active palette
(`colony_ui::theme::Palette::*()`), in view code and in style closures alike,
and reuse the style helpers in `theme.rs`; a colour literal would ignore the
user's theme and high contrast. Write text with `theme::ui_text(content, base)`
rather than `text(..).size(N)`, and give a text input
`.size(sz(N)).font(ui_font())`, so both text sizes and the dyslexia font reach
it. Gate animations on `colony_ui::effects_enabled()`.

**Text.** Views hold no text: they call `i18n::t(key)`, `i18n::tf(key, vars)`
to fill `{name}` placeholders, or `i18n::tn(key, count, vars)` for a string that
depends on a count (`key_one` and `key_other`; French also uses the singular
for zero). Numbers with decimals and percentages go through `i18n::decimal` and
`i18n::percent`, which follow French typography (`12,5 %`). English is
canonical: name a new key after its English meaning and write the English first,
then the French. `cargo test` fails when `en.json` and `fr.json` hold different
keys, when a string is empty or the placeholders differ, when a key is missing
or unused, or when one repeats a shared colony-ui label. French uses the vous
form and a non-breaking space before `:`, `;`, `!` and `?`. Store, program and
theme names stay untranslated. Game statuses are stored by their English name
(`GameStatus`'s `Display`) and shown through `GameStatus::label`.

**Files.** Spotter's own paths come from `paths::layout()` in `src/paths.rs`,
built on `colony_ui::paths` (`config_dir`, `data_dir`, `cache_dir`); never
build them from `dirs` directly. The layout is resolved, and its folders
created, once at startup.

**Errors.** Most functions still return `Result<T, String>`; the database layer
uses `SpotterError` from `error.rs`, and new code should prefer it.
