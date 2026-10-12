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
| `#[cfg(test)]` modules in `src/` | Steam HTML scraping, multi-byte title and description cuts, keyring moves, the write refusal after a failed load, the folders and their migration, the en/fr key parity, the window icon |

Not covered by automated tests: live store requests (the importers are only
tested against recorded responses), the login flows end to end, the import
merge in `handlers/data.rs`, and rendering.
A change to a view, the Preferences page or the French layout needs a look on a
real desktop at both ends of the text scale (0.72x and 1.68x).

## Before a pull request

CI runs on Linux, Windows and macOS, and again on Rust 1.90:

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

**Look.** Colours come from colony-ui's palette roles and text sizes from
`Typography::current()` and `typo.sz(base)`; no colour literals and no raw
`.size(N)`. Gate animations on `colony_ui::effects_enabled()`.

**Text.** Every user-visible string is a key, present in both the English and
the French table (a test fails otherwise). French uses the vous form and a
space before `:`, `;`, `!` and `?`. Store and product names stay untranslated.

**Files.** Paths come from `colony_ui::paths` (`config_dir`, `data_dir`,
`cache_dir`), never from `dirs` directly; use `paths::locate` to show a path
without creating it.

**Errors.** Most functions still return `Result<T, String>`; the database layer
uses `SpotterError` from `error.rs`, and new code should prefer it.
