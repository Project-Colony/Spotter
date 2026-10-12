# Data model

## Where the data lives

Spotter uses the three standard Colony folders, through
`colony_ui::paths::{config_dir, data_dir, cache_dir}("Spotter")`, and keeps
them apart by sub-directory, because config and data are the same folder on
Windows and macOS:

| | Linux | Windows | macOS |
|---|---|---|---|
| config | `~/.config/Colony/Spotter/` | `%LOCALAPPDATA%\Colony\Spotter\` | `~/Library/Application Support/Colony/Spotter/` |
| data | `~/.local/share/Colony/Spotter/` | `%LOCALAPPDATA%\Colony\Spotter\` | `~/Library/Application Support/Colony/Spotter/` |
| cache | `~/.cache/Colony/Spotter/` | `%LOCALAPPDATA%\Colony\Spotter\cache\` | `~/Library/Caches/Colony/Spotter/` |

```
<config>/
├── preferences/
│   └── preferences.json          every preference, see guide/configuration.md
└── credentials/
    └── tokens.json               only when no OS keyring is usable (0600 on Unix)

<data>/
├── library/
│   └── spotter.db                the SQLite database below
├── exports/
│   ├── spotter_export.json       written by Export JSON, read by Import JSON
│   ├── spotter_export.csv        written by Export CSV
│   └── spotter_auto_backup.json  rewritten after every import
└── logs/
    ├── import.log                one line per import step
    └── import.log.1              the previous log, rotated at 5 MB

<cache>/
├── covers/                       one .jpg per game, named after its title
└── achievement_icons/            <appid>_<name>.jpg and its grey variant
```

`paths::layout()` in `src/paths.rs` resolves these paths and creates the
folders once, at startup; the Profile page shows the paths it resolved.
Deleting the cache folder costs nothing but the downloads.

Spotter 0.3 and older kept everything under `~/.config/Colony/Spotter/`
(`Application Support` on macOS, `AppData\Local` on Windows), with the database
in `data/` and the settings in the database. A newer version copies the
database, exports and logs from there on its first start, imports the old
settings row into `preferences.json`, writes a marker so the check runs once,
and never deletes the old folder. If the copy fails, it keeps using the old
files where they are (`<config>/data/spotter.db`, `<config>/exports/`,
`<config>/logs/`) rather than start on an empty library, and tries the copy
again on the next start. On a system without a usable keyring, the old
database left in `<config>/data/` can still hold the platform tokens
unencrypted.

## Records

### Game

The library entry, one row of the `games` table.

```rust
Game {
    id: Option<i64>,              // SQLite primary key, None before the first save
    title: String,
    platform: Platform,           // Steam | Gog | Epic | PlayStation | Xbox | Nintendo
    playtime_minutes: u32,
    achievements_unlocked: u32,
    achievements_total: u32,
    status: GameStatus,           // Playing | Completed | Unplayed | Dropped | Wishlist
    rating: Option<u8>,           // 0-10, None = unrated
    genre: String,
    last_played: String,          // "YYYY-MM-DD", empty if never
    cover_url: String,            // remote address; the file is in <cache>/covers
    steam_appid: Option<u32>,
    gog_id: Option<String>,
    epic_id: Option<String>,
    xbox_id: Option<String>,
    psn_id: Option<String>,
    notes: String,                // the user's own notes
    description: String,          // store description
    release_date: String,
    review_percent: Option<u32>,  // store review score, 0-100
    tags: String,                 // comma-separated
}
```

The store ids are what an import matches on before it falls back to the title;
see [architecture.md](architecture.md#import-merge).

### Achievement

```rust
Achievement {
    api_name: String,        // the store's internal id
    display_name: String,
    description: String,
    icon_url: String,        // unlocked icon
    icon_gray_url: String,   // locked icon
    unlocked: bool,
    unlock_time: u64,        // Unix timestamp, 0 while locked
}
```

Steam and Xbox imports store one row per achievement. PlayStation only fills
the game's unlocked and total counts.

### UserProfile

```rust
UserProfile {
    username: String,             // display name, "Player" by default
    avatar_path: String,          // unused
    steam_api_key: String,        // secret
    steam_id: String,             // 64-bit Steam ID, from the Steam login
    gog_token: String,            // secret
    gog_refresh_token: String,    // secret
    xbox_api_key: String,         // secret, the OpenXBL key
    xbox_gamertag: String,
    xbox_xuid: String,            // Xbox user id, read during an import
    psn_npsso: String,            // secret
    epic_token: String,           // secret
    epic_refresh_token: String,   // secret
    epic_account_id: String,
    epic_display_name: String,
    member_since: String,         // "YYYY-MM-DD", set on creation
}
```

`keyring.rs` loads and saves the seven secret fields. They live in the OS
keyring under the service name `spotter`, one entry per field. When no keyring
is usable they go to `<config>/credentials/tokens.json` instead, and the
Profile page says so. The matching database columns stay empty.

### Preferences

`preferences.json` holds Spotter's own preferences and colony-ui's
`StandardPreferences`, flattened into one object. Every field is read on its
own, so one bad value only resets that field. The keys are listed in
[guide/configuration.md](../guide/configuration.md).

## Database schema

`spotter.db` is SQLite in WAL mode with foreign keys on and `secure_delete`
on, so an overwritten value does not linger in the file's free pages.
`db::open` creates the tables if they are missing and then runs the column
migrations in `db.rs`, once per session.

### games

```sql
CREATE TABLE games (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    title                 TEXT NOT NULL,
    platform              TEXT NOT NULL,
    playtime_minutes      INTEGER DEFAULT 0,
    achievements_unlocked INTEGER DEFAULT 0,
    achievements_total    INTEGER DEFAULT 0,
    status                TEXT DEFAULT 'Unplayed',
    rating                INTEGER,
    genre                 TEXT DEFAULT '',
    last_played           TEXT DEFAULT '',
    cover_url             TEXT DEFAULT '',
    steam_appid           INTEGER,
    gog_id                TEXT,
    epic_id               TEXT,
    xbox_id               TEXT,
    psn_id                TEXT,
    notes                 TEXT DEFAULT '',
    description           TEXT DEFAULT '',
    release_date          TEXT DEFAULT '',
    review_percent        INTEGER,
    tags                  TEXT DEFAULT '',
    created_at            TEXT DEFAULT CURRENT_TIMESTAMP
);
```

Indexes: `platform`, `status`, `title`, `rating`, `last_played`, and each store
id (`steam_appid`, `gog_id`, `epic_id`, `xbox_id`, `psn_id`).

### achievements

```sql
CREATE TABLE achievements (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    steam_appid   INTEGER NOT NULL,
    platform_id   TEXT NOT NULL DEFAULT '',   -- '' for Steam, 'xbox:<title id>' for Xbox
    api_name      TEXT NOT NULL,
    display_name  TEXT NOT NULL DEFAULT '',
    description   TEXT NOT NULL DEFAULT '',
    icon_url      TEXT NOT NULL DEFAULT '',
    icon_gray_url TEXT NOT NULL DEFAULT '',
    unlocked      INTEGER NOT NULL DEFAULT 0,
    unlock_time   INTEGER NOT NULL DEFAULT 0,
    UNIQUE(steam_appid, platform_id, api_name)
);
```

### playtime_history

```sql
CREATE TABLE playtime_history (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id        INTEGER NOT NULL,
    date           TEXT NOT NULL,              -- "YYYY-MM-DD"
    minutes_played INTEGER NOT NULL,
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);
```

The Statistics screen charts the last 30 days of it.

### user_profile

A single row (`id = 1`) with one column per `UserProfile` field. The seven
secret columns (`steam_api_key`, `gog_token`, `gog_refresh_token`,
`xbox_api_key`, `psn_npsso`, `epic_token`, `epic_refresh_token`) are empty:
the secrets are in the keyring, or in `<config>/credentials/tokens.json`
without one. A secret stays in its column, unencrypted, only when neither the
keyring nor that file could take it; `keyring.rs` moves it out on the next
start.

### settings

A single row (`id = 1`) with a JSON `data` column. Only Spotter 0.3 and older
wrote it; a newer version reads it once to seed `preferences.json` on upgrade.
