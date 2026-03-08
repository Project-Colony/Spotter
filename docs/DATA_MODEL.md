# Data Model

## Core Structures

### Game

The primary entity. 21 fields tracking a game across platforms.

```rust
Game {
    id: Option<i64>,              // SQLite AUTOINCREMENT primary key
    title: String,                // Display name
    platform: Platform,           // Steam | Gog | Epic | PlayStation | Xbox | Nintendo
    playtime_minutes: u32,        // Total play time
    achievements_unlocked: u32,   // Player's unlocked count
    achievements_total: u32,      // Available achievements
    status: GameStatus,           // Playing | Completed | Unplayed | Dropped | Wishlist
    rating: Option<u8>,           // 0-10 user rating (None = unrated)
    genre: String,                // e.g. "Action RPG"
    last_played: String,          // "YYYY-MM-DD" or "—" if never
    cover_url: String,            // Remote URL for cover art
    steam_appid: Option<u32>,     // Steam App ID (e.g. 1245620 for Elden Ring)
    gog_id: Option<String>,       // GOG product ID
    epic_id: Option<String>,      // Epic catalog item ID
    xbox_id: Option<String>,      // Xbox title ID
    psn_id: Option<String>,       // PSN game ID
    notes: String,                // User notes (max 2000 chars)
    description: String,          // Store description
    release_date: String,         // "YYYY-MM-DD"
    review_percent: Option<u32>,  // Store review score (0-100)
    tags: String,                 // Comma-separated tags
}
```

**Builder pattern:**

```rust
GameBuilder::new("Elden Ring", Platform::Steam)
    .playtime(12000)
    .achievements(32, 42)
    .status(GameStatus::Playing)
    .rating(9)
    .genre("Action RPG")
    .steam_appid(1245620)
    .build()
```

### Platform

```
Steam       → color: rgb(0.4, 0.6, 0.9)   (blue)
Gog         → color: rgb(0.7, 0.5, 0.9)   (purple)
Epic        → color: rgb(0.9, 0.9, 0.3)   (yellow)
PlayStation → color: rgb(0.2, 0.4, 0.9)   (navy)
Xbox        → color: rgb(0.2, 0.8, 0.2)   (green)
Nintendo    → color: rgb(0.9, 0.2, 0.2)   (red)
```

### GameStatus

```
Playing   → color: rgb(0.2, 0.8, 0.4)   (green)
Completed → color: rgb(0.3, 0.6, 1.0)   (blue)
Unplayed  → color: rgb(0.9, 0.7, 0.2)   (gold)
Dropped   → color: rgb(0.8, 0.3, 0.3)   (red)
Wishlist  → color: rgb(0.7, 0.5, 0.9)   (purple)
```

### Achievement

Per-game achievement data. Only available for Steam games currently.

```rust
Achievement {
    api_name: String,        // Internal ID (e.g. "ACHIEVEMENT_001")
    display_name: String,    // "First Steps"
    description: String,     // "Complete the tutorial"
    icon_url: String,        // Colored icon URL (unlocked)
    icon_gray_url: String,   // Grayscale icon URL (locked)
    unlocked: bool,          // Player's unlock status
    unlock_time: u64,        // Unix timestamp (0 if locked)
}
```

### UserProfile

Stores user identity and platform credentials.

```rust
UserProfile {
    username: String,           // Display name (default: "Player")
    avatar_path: String,        // Reserved for future use
    steam_api_key: String,      // From steamcommunity.com/dev/apikey
    steam_id: String,           // 64-bit Steam ID (from login)
    gog_token: String,          // OAuth access token
    gog_refresh_token: String,  // For automatic token renewal
    xbox_api_key: String,       // From xbl.io
    xbox_gamertag: String,      // Xbox username
    psn_npsso: String,          // PSN browser cookie token
    member_since: String,       // "YYYY-MM-DD" (auto-set on creation)
}
```

### Settings

25 user-configurable preferences. See [CONFIGURATION.md](CONFIGURATION.md) for details.

## Database Schema

SQLite with WAL mode and foreign keys enabled.

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
    tags                  TEXT DEFAULT ''
);
```

**Indexes:** `platform`, `status`, `steam_appid`, `title`, `rating`, `last_played`

### achievements

```sql
CREATE TABLE achievements (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    steam_appid    INTEGER NOT NULL,
    api_name       TEXT NOT NULL,
    display_name   TEXT DEFAULT '',
    description    TEXT DEFAULT '',
    icon_url       TEXT DEFAULT '',
    icon_gray_url  TEXT DEFAULT '',
    unlocked       INTEGER DEFAULT 0,
    unlock_time    INTEGER DEFAULT 0,
    UNIQUE(steam_appid, api_name)
);
```

### playtime_history

```sql
CREATE TABLE playtime_history (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id        INTEGER NOT NULL REFERENCES games(id),
    date           TEXT NOT NULL,         -- "YYYY-MM-DD"
    minutes_played INTEGER DEFAULT 0
);
```

### user_profile

Single-row table (id = 1) storing `UserProfile` fields as individual columns.

### settings

Single-row table (id = 1) storing JSON-serialized `Settings` struct.

```sql
CREATE TABLE settings (
    id   INTEGER PRIMARY KEY,
    data TEXT NOT NULL              -- JSON blob
);
```

## Storage Locations

| OS | Path |
|----|------|
| Linux | `~/.config/Colony/Spotter/` |
| macOS | `~/Library/Application Support/Colony/Spotter/` |
| Windows | `%LOCALAPPDATA%\Colony\Spotter\` |

**Subdirectories:**

```
Colony/Spotter/
├── data/
│   └── spotter.db              # SQLite database
├── cache/
│   ├── covers/                 # Game cover art (JPEG)
│   └── achievement_icons/      # Achievement icons (colored + grayscale)
└── exports/
    └── spotter_export.json     # JSON backup file
```
