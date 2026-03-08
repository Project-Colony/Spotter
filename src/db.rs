use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::SpotterError;
use crate::models::*;

/// Track whether migrations have run this session (avoids 37 SQL statements per open).
static MIGRATED: AtomicBool = AtomicBool::new(false);

/// Raw base path without directory creation.
/// Linux:   ~/.config/Colony/Spotter/
/// Windows: AppData/Local/Colony/Spotter/
/// macOS:   ~/Library/Application Support/Colony/Spotter/
fn base_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = dirs::data_local_dir();
    #[cfg(not(target_os = "windows"))]
    let base = dirs::config_dir();

    base.unwrap_or_else(|| PathBuf::from("."))
        .join("Colony")
        .join("Spotter")
}

/// Ensure a directory exists, logging on failure.
fn ensure_dir(dir: &std::path::Path) {
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("[db] Failed to create directory {:?}: {}", dir, e);
    }
}

/// Base directory for all Spotter data.
pub fn base_dir() -> PathBuf {
    let dir = base_path();
    ensure_dir(&dir);
    dir
}

/// Data directory (database).
pub fn data_dir() -> PathBuf {
    let dir = base_path().join("data");
    ensure_dir(&dir);
    dir
}

/// Cache directory (covers, achievement icons).
pub fn cache_dir() -> PathBuf {
    let dir = base_path().join("cache");
    ensure_dir(&dir);
    dir
}

/// Covers cache directory.
pub fn covers_dir() -> PathBuf {
    let dir = base_path().join("cache").join("covers");
    ensure_dir(&dir);
    dir
}

/// Exports directory (JSON/CSV backups).
pub fn exports_dir() -> PathBuf {
    let dir = base_path().join("exports");
    ensure_dir(&dir);
    dir
}

pub fn db_path() -> PathBuf {
    data_dir().join("spotter.db")
}

pub fn open() -> Result<Connection, SpotterError> {
    open_inner(db_path().as_ref(), true)
}

/// Open (or create) a database at an explicit path.
/// Used by tests to avoid global env-var side effects.
#[allow(dead_code)]
pub fn open_at(path: &std::path::Path) -> Result<Connection, SpotterError> {
    open_inner(path, false)
}

fn open_inner(
    path: &std::path::Path,
    use_migration_flag: bool,
) -> Result<Connection, SpotterError> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;",
    )?;
    if use_migration_flag {
        // Check MIGRATED flag but also verify tables exist (handles fresh databases
        // when the flag is stale, e.g. in tests with different data directories).
        let needs_init = !MIGRATED.load(Ordering::Acquire) || !has_games_table(&conn);
        if needs_init {
            init_tables(&conn)?;
            migrate(&conn);
            MIGRATED.store(true, Ordering::Release);
        }
    } else {
        init_tables(&conn)?;
        migrate(&conn);
    }
    Ok(conn)
}

fn has_games_table(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='games'",
        [],
        |row| row.get::<_, i32>(0),
    )
    .unwrap_or(0)
        > 0
}

fn init_tables(conn: &Connection) -> Result<(), SpotterError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            platform TEXT NOT NULL,
            playtime_minutes INTEGER DEFAULT 0,
            achievements_unlocked INTEGER DEFAULT 0,
            achievements_total INTEGER DEFAULT 0,
            status TEXT DEFAULT 'Unplayed',
            rating INTEGER,
            genre TEXT DEFAULT '',
            last_played TEXT DEFAULT '',
            cover_url TEXT DEFAULT '',
            steam_appid INTEGER,
            gog_id TEXT,
            epic_id TEXT,
            xbox_id TEXT,
            psn_id TEXT,
            notes TEXT DEFAULT '',
            description TEXT DEFAULT '',
            release_date TEXT DEFAULT '',
            review_percent INTEGER,
            tags TEXT DEFAULT '',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS playtime_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id INTEGER NOT NULL,
            date TEXT NOT NULL,
            minutes_played INTEGER NOT NULL,
            FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
        );

        -- Credentials are stored as plaintext in the local SQLite database.
        -- This is a deliberate design choice for a desktop application where:
        --   1. The database is only accessible to the local OS user
        --   2. OS-level keychain integration (via the `keyring` crate) would add
        --      complexity and platform-specific failure modes
        --   3. The stored tokens (Steam API key, GOG OAuth tokens, Xbox API key,
        --      PSN NPSSO cookie) are user-provided and scoped to read-only game data
        -- Future improvement: integrate with the OS keyring for sensitive fields.
        CREATE TABLE IF NOT EXISTS user_profile (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            username TEXT DEFAULT 'Player',
            avatar_path TEXT DEFAULT '',
            steam_api_key TEXT DEFAULT '',
            steam_id TEXT DEFAULT '',
            gog_token TEXT DEFAULT '',
            xbox_api_key TEXT DEFAULT '',
            xbox_gamertag TEXT DEFAULT '',
            psn_npsso TEXT DEFAULT '',
            member_since TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS achievements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            steam_appid INTEGER NOT NULL,
            platform_id TEXT NOT NULL DEFAULT '',
            api_name TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            icon_url TEXT NOT NULL DEFAULT '',
            icon_gray_url TEXT NOT NULL DEFAULT '',
            unlocked INTEGER NOT NULL DEFAULT 0,
            unlock_time INTEGER NOT NULL DEFAULT 0,
            UNIQUE(steam_appid, platform_id, api_name)
        );

        CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            data TEXT NOT NULL DEFAULT '{}'
        );

        -- Indexes for common queries
        CREATE INDEX IF NOT EXISTS idx_games_platform ON games(platform);
        CREATE INDEX IF NOT EXISTS idx_games_status ON games(status);
        CREATE INDEX IF NOT EXISTS idx_games_steam_appid ON games(steam_appid);
        CREATE INDEX IF NOT EXISTS idx_playtime_game_id ON playtime_history(game_id);
        CREATE INDEX IF NOT EXISTS idx_playtime_date ON playtime_history(date);
        CREATE INDEX IF NOT EXISTS idx_achievements_appid ON achievements(steam_appid);
        CREATE INDEX IF NOT EXISTS idx_games_title ON games(title);
        CREATE INDEX IF NOT EXISTS idx_games_rating ON games(rating);
        CREATE INDEX IF NOT EXISTS idx_games_last_played ON games(last_played);
        ",
    )?;
    Ok(())
}

/// Run migrations for existing databases (add columns that might be missing).
/// ALTER TABLE ADD COLUMN returns an error if the column already exists — that's expected
/// and we intentionally ignore it. Real errors (disk full, corruption) are logged.
fn migrate(conn: &Connection) {
    let migrations = [
        "ALTER TABLE games ADD COLUMN notes TEXT DEFAULT ''",
        "ALTER TABLE games ADD COLUMN description TEXT DEFAULT ''",
        "ALTER TABLE games ADD COLUMN release_date TEXT DEFAULT ''",
        "ALTER TABLE games ADD COLUMN review_percent INTEGER",
        "ALTER TABLE games ADD COLUMN tags TEXT DEFAULT ''",
        "ALTER TABLE games ADD COLUMN epic_id TEXT",
        "ALTER TABLE games ADD COLUMN xbox_id TEXT",
        "ALTER TABLE games ADD COLUMN psn_id TEXT",
        "ALTER TABLE user_profile ADD COLUMN xbox_api_key TEXT DEFAULT ''",
        "ALTER TABLE user_profile ADD COLUMN xbox_gamertag TEXT DEFAULT ''",
        "ALTER TABLE user_profile ADD COLUMN psn_npsso TEXT DEFAULT ''",
        "ALTER TABLE user_profile ADD COLUMN gog_refresh_token TEXT DEFAULT ''",
        "ALTER TABLE user_profile ADD COLUMN epic_token TEXT DEFAULT ''",
        "ALTER TABLE user_profile ADD COLUMN epic_refresh_token TEXT DEFAULT ''",
        "ALTER TABLE user_profile ADD COLUMN epic_account_id TEXT DEFAULT ''",
        "ALTER TABLE user_profile ADD COLUMN epic_display_name TEXT DEFAULT ''",
        "ALTER TABLE user_profile ADD COLUMN xbox_xuid TEXT DEFAULT ''",
        // platform_id column on achievements is added during table rebuild (see below)
        // or included in CREATE TABLE for fresh databases.
        "ALTER TABLE achievements ADD COLUMN platform_id TEXT DEFAULT ''",
    ];

    for sql in &migrations {
        if let Err(e) = conn.execute(sql, []) {
            let msg = e.to_string();
            // "duplicate column name" is expected for already-applied migrations.
            // Also check for "duplicate column" to handle different SQLite versions.
            if !msg.contains("duplicate column") && !msg.contains("already exists") {
                eprintln!("[db] Migration warning: {} — {}", sql, msg);
            }
        }
    }

    // Data migration: rename old status value
    if let Err(e) = conn.execute(
        "UPDATE games SET status = 'Unplayed' WHERE status = 'Backlog'",
        [],
    ) {
        eprintln!("[db] Status migration failed: {}", e);
    }

    // Platform ID indexes for faster import lookups
    let indexes = [
        "CREATE INDEX IF NOT EXISTS idx_games_gog_id ON games(gog_id)",
        "CREATE INDEX IF NOT EXISTS idx_games_epic_id ON games(epic_id)",
        "CREATE INDEX IF NOT EXISTS idx_games_xbox_id ON games(xbox_id)",
        "CREATE INDEX IF NOT EXISTS idx_games_psn_id ON games(psn_id)",
        "CREATE INDEX IF NOT EXISTS idx_achievements_platform_id ON achievements(platform_id)",
    ];
    for sql in &indexes {
        if let Err(e) = conn.execute(sql, []) {
            eprintln!("[db] Index creation failed: {}", e);
        }
    }

    // Rebuild achievements table if it has the old UNIQUE(steam_appid, api_name)
    // constraint. The new constraint includes platform_id so multiple Xbox games
    // can have achievements with the same numeric ID (e.g., "1", "2").
    let needs_rebuild: bool = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='achievements'",
            [],
            |row| {
                let sql: String = row.get(0)?;
                // Old constraint: UNIQUE(steam_appid, api_name) without platform_id
                Ok(sql.contains("UNIQUE(steam_appid, api_name)")
                    || (sql.contains("UNIQUE") && !sql.contains("platform_id")))
            },
        )
        .unwrap_or(false);

    if needs_rebuild {
        eprintln!("[db] Rebuilding achievements table with platform-aware UNIQUE constraint...");
        if let Err(e) = conn.execute_batch(
            "CREATE TABLE achievements_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                steam_appid INTEGER NOT NULL,
                platform_id TEXT NOT NULL DEFAULT '',
                api_name TEXT NOT NULL,
                display_name TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                icon_url TEXT NOT NULL DEFAULT '',
                icon_gray_url TEXT NOT NULL DEFAULT '',
                unlocked INTEGER NOT NULL DEFAULT 0,
                unlock_time INTEGER NOT NULL DEFAULT 0,
                UNIQUE(steam_appid, platform_id, api_name)
            );
            INSERT OR IGNORE INTO achievements_new
                (id, steam_appid, platform_id, api_name, display_name, description,
                 icon_url, icon_gray_url, unlocked, unlock_time)
            SELECT id, steam_appid, COALESCE(platform_id, ''), api_name, display_name,
                   description, icon_url, icon_gray_url, unlocked, unlock_time
            FROM achievements;
            DROP TABLE achievements;
            ALTER TABLE achievements_new RENAME TO achievements;
            CREATE INDEX IF NOT EXISTS idx_achievements_appid ON achievements(steam_appid);
            CREATE INDEX IF NOT EXISTS idx_achievements_platform_id ON achievements(platform_id);",
        ) {
            eprintln!("[db] Achievements table rebuild failed: {}", e);
        } else {
            eprintln!("[db] Achievements table rebuilt successfully");
        }
    }
}

pub fn load_games(conn: &Connection) -> Result<Vec<Game>, SpotterError> {
    let mut stmt = conn.prepare(
        "SELECT id, title, platform, playtime_minutes, achievements_unlocked,
                    achievements_total, status, rating, genre, last_played,
                    cover_url, steam_appid, gog_id, COALESCE(notes, ''),
                    COALESCE(description, ''), COALESCE(release_date, ''),
                    review_percent, COALESCE(tags, ''), epic_id, xbox_id, psn_id
             FROM games ORDER BY title",
    )?;

    let games = stmt.query_map([], |row| {
        Ok(Game {
            id: Some(row.get::<_, i64>(0)?),
            title: row.get(1)?,
            platform: Platform::from_str_name(&row.get::<_, String>(2)?),
            playtime_minutes: row.get(3)?,
            achievements_unlocked: row.get(4)?,
            achievements_total: row.get(5)?,
            status: GameStatus::from_str_name(&row.get::<_, String>(6)?),
            rating: row.get(7)?,
            genre: row.get(8)?,
            last_played: row.get(9)?,
            cover_url: row.get::<_, String>(10).unwrap_or_default(),
            steam_appid: row.get(11)?,
            gog_id: row.get(12)?,
            notes: row.get::<_, String>(13).unwrap_or_default(),
            description: row.get::<_, String>(14).unwrap_or_default(),
            release_date: row.get::<_, String>(15).unwrap_or_default(),
            review_percent: row.get(16)?,
            tags: row.get::<_, String>(17).unwrap_or_default(),
            epic_id: row.get(18)?,
            xbox_id: row.get(19)?,
            psn_id: row.get(20)?,
        })
    })?;

    let mut result = Vec::with_capacity(256);
    for game in games {
        result.push(game?);
    }
    Ok(result)
}

pub fn save_game(conn: &Connection, game: &Game) -> Result<i64, SpotterError> {
    if let Some(id) = game.id {
        conn.execute(
            "UPDATE games SET title=?1, platform=?2, playtime_minutes=?3,
             achievements_unlocked=?4, achievements_total=?5, status=?6,
             rating=?7, genre=?8, last_played=?9, cover_url=?10,
             steam_appid=?11, gog_id=?12, notes=?13,
             description=?14, release_date=?15, review_percent=?16, tags=?17,
             epic_id=?18, xbox_id=?19, psn_id=?20
             WHERE id=?21",
            params![
                game.title,
                game.platform.to_string(),
                game.playtime_minutes,
                game.achievements_unlocked,
                game.achievements_total,
                game.status.to_string(),
                game.rating,
                game.genre,
                game.last_played,
                game.cover_url,
                game.steam_appid,
                game.gog_id,
                game.notes,
                game.description,
                game.release_date,
                game.review_percent,
                game.tags,
                game.epic_id,
                game.xbox_id,
                game.psn_id,
                id
            ],
        )?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO games (title, platform, playtime_minutes, achievements_unlocked,
             achievements_total, status, rating, genre, last_played, cover_url,
             steam_appid, gog_id, notes, description, release_date, review_percent, tags,
             epic_id, xbox_id, psn_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
            params![
                game.title,
                game.platform.to_string(),
                game.playtime_minutes,
                game.achievements_unlocked,
                game.achievements_total,
                game.status.to_string(),
                game.rating,
                game.genre,
                game.last_played,
                game.cover_url,
                game.steam_appid,
                game.gog_id,
                game.notes,
                game.description,
                game.release_date,
                game.review_percent,
                game.tags,
                game.epic_id,
                game.xbox_id,
                game.psn_id,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

/// Save all games, writing new IDs back into the slice.
pub fn save_all_games(conn: &Connection, games: &mut [Game]) -> Result<(), SpotterError> {
    let tx = conn.unchecked_transaction()?;
    for game in games.iter_mut() {
        let id = save_game(&tx, game)?;
        game.id = Some(id);
    }
    tx.commit()?;
    Ok(())
}

/// Save all games without mutating (IDs are not written back).
/// Use this for background persistence where the caller does not need new IDs.
pub fn save_all_games_ref(conn: &Connection, games: &[Game]) -> Result<(), SpotterError> {
    let tx = conn.unchecked_transaction()?;
    for game in games {
        save_game(&tx, game)?;
    }
    tx.commit()?;
    Ok(())
}

pub fn delete_game(conn: &Connection, game_id: i64) -> Result<(), SpotterError> {
    let tx = conn.unchecked_transaction()?;
    // Delete associated achievements (look up steam_appid first)
    let steam_appid: Option<u32> = tx
        .query_row(
            "SELECT steam_appid FROM games WHERE id = ?1",
            params![game_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();
    if let Some(appid) = steam_appid {
        tx.execute(
            "DELETE FROM achievements WHERE steam_appid = ?1",
            params![appid],
        )?;
    }
    // Delete playtime history
    tx.execute(
        "DELETE FROM playtime_history WHERE game_id = ?1",
        params![game_id],
    )?;
    // Delete the game itself
    tx.execute("DELETE FROM games WHERE id = ?1", params![game_id])?;
    tx.commit()?;
    Ok(())
}

pub fn load_profile(conn: &Connection) -> Result<UserProfile, SpotterError> {
    let mut stmt = conn.prepare(
        "SELECT username, avatar_path, steam_api_key, steam_id, gog_token,
                  COALESCE(xbox_api_key, ''), COALESCE(xbox_gamertag, ''),
                  COALESCE(psn_npsso, ''), member_since,
                  COALESCE(gog_refresh_token, ''),
                  COALESCE(epic_token, ''), COALESCE(epic_refresh_token, ''),
                  COALESCE(epic_account_id, ''), COALESCE(epic_display_name, ''),
                  COALESCE(xbox_xuid, '')
                  FROM user_profile WHERE id = 1",
    )?;

    let profile = stmt.query_row([], |row| {
        Ok(UserProfile {
            username: row.get(0)?,
            avatar_path: row.get(1)?,
            steam_api_key: row.get(2)?,
            steam_id: row.get(3)?,
            gog_token: row.get(4)?,
            xbox_api_key: row.get::<_, String>(5).unwrap_or_default(),
            xbox_gamertag: row.get::<_, String>(6).unwrap_or_default(),
            psn_npsso: row.get::<_, String>(7).unwrap_or_default(),
            member_since: row.get(8)?,
            gog_refresh_token: row.get::<_, String>(9).unwrap_or_default(),
            epic_token: row.get::<_, String>(10).unwrap_or_default(),
            epic_refresh_token: row.get::<_, String>(11).unwrap_or_default(),
            epic_account_id: row.get::<_, String>(12).unwrap_or_default(),
            epic_display_name: row.get::<_, String>(13).unwrap_or_default(),
            xbox_xuid: row.get::<_, String>(14).unwrap_or_default(),
        })
    });

    match profile {
        Ok(p) => Ok(p),
        Err(_) => {
            let p = UserProfile::default();
            save_profile(conn, &p)?;
            Ok(p)
        }
    }
}

pub fn save_profile(conn: &Connection, profile: &UserProfile) -> Result<(), SpotterError> {
    conn.execute(
        "INSERT INTO user_profile (id, username, avatar_path, steam_api_key, steam_id, gog_token,
         xbox_api_key, xbox_gamertag, psn_npsso, member_since, gog_refresh_token,
         epic_token, epic_refresh_token, epic_account_id, epic_display_name, xbox_xuid)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
         ON CONFLICT(id) DO UPDATE SET
         username=?1, avatar_path=?2, steam_api_key=?3, steam_id=?4, gog_token=?5,
         xbox_api_key=?6, xbox_gamertag=?7, psn_npsso=?8, member_since=?9, gog_refresh_token=?10,
         epic_token=?11, epic_refresh_token=?12, epic_account_id=?13, epic_display_name=?14,
         xbox_xuid=?15",
        params![
            profile.username,
            profile.avatar_path,
            profile.steam_api_key,
            profile.steam_id,
            profile.gog_token,
            profile.xbox_api_key,
            profile.xbox_gamertag,
            profile.psn_npsso,
            profile.member_since,
            profile.gog_refresh_token,
            profile.epic_token,
            profile.epic_refresh_token,
            profile.epic_account_id,
            profile.epic_display_name,
            profile.xbox_xuid,
        ],
    )?;
    Ok(())
}

pub fn get_daily_playtime(
    conn: &Connection,
    days: u32,
) -> Result<Vec<(String, u32)>, SpotterError> {
    let since = chrono::Local::now()
        .checked_sub_signed(chrono::Duration::days(days as i64))
        .unwrap_or_else(chrono::Local::now)
        .format("%Y-%m-%d")
        .to_string();

    let mut stmt = conn.prepare(
        "SELECT date, SUM(minutes_played) as total
         FROM playtime_history
         WHERE date >= ?1
         GROUP BY date
         ORDER BY date",
    )?;

    let rows = stmt.query_map(params![since], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
    })?;

    let mut result = Vec::with_capacity(64);
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn export_games_json_from_slice(games: &[Game]) -> Result<String, SpotterError> {
    Ok(serde_json::to_string_pretty(games)?)
}

pub fn export_games_csv_from_slice(games: &[Game]) -> Result<String, SpotterError> {
    let mut wtr = csv::Writer::from_writer(Vec::with_capacity(games.len() * 256));
    for game in games {
        wtr.serialize(game)?;
    }
    let bytes = wtr
        .into_inner()
        .map_err(|e| SpotterError::Other(format!("CSV flush error: {}", e)))?;
    String::from_utf8(bytes).map_err(|e| SpotterError::Parse(format!("CSV UTF-8 error: {}", e)))
}

/// Import games from a JSON backup file, returning the parsed games.
pub fn import_games_json() -> Result<Vec<Game>, SpotterError> {
    let import_path = exports_dir().join("spotter_export.json");
    if !import_path.exists() {
        return Err(SpotterError::Import(format!(
            "No backup file found at {}. Export first, then import.",
            import_path.display()
        )));
    }
    let json = std::fs::read_to_string(&import_path)?;
    let games: Vec<Game> = serde_json::from_str(&json)?;
    Ok(games)
}

/// Replace all achievements for a given Steam appid (atomic transaction).
pub fn save_achievements(
    conn: &Connection,
    steam_appid: u32,
    achievements: &[Achievement],
) -> Result<(), SpotterError> {
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "DELETE FROM achievements WHERE steam_appid = ?1",
        params![steam_appid],
    )?;

    let mut stmt = tx.prepare(
        "INSERT INTO achievements
             (steam_appid, api_name, display_name, description,
              icon_url, icon_gray_url, unlocked, unlock_time)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )?;

    for a in achievements {
        stmt.execute(params![
            steam_appid,
            a.api_name,
            a.display_name,
            a.description,
            a.icon_url,
            a.icon_gray_url,
            a.unlocked as i32,
            a.unlock_time as i64,
        ])?;
    }

    drop(stmt);
    tx.commit()?;
    Ok(())
}

/// Load application settings (returns defaults if not yet saved).
pub fn load_settings(conn: &Connection) -> Result<Settings, SpotterError> {
    let mut stmt = conn.prepare("SELECT data FROM settings WHERE id = 1")?;

    let result = stmt.query_row([], |row| row.get::<_, String>(0));

    match result {
        Ok(json) => serde_json::from_str(&json).map_err(Into::into),
        Err(_) => Ok(Settings::default()),
    }
}

/// Save application settings as JSON.
pub fn save_settings(conn: &Connection, settings: &Settings) -> Result<(), SpotterError> {
    let json = serde_json::to_string(settings)?;
    conn.execute(
        "INSERT INTO settings (id, data) VALUES (1, ?1)
         ON CONFLICT(id) DO UPDATE SET data=?1",
        params![json],
    )?;
    Ok(())
}

/// Load all achievements for a given Steam appid, in insertion order.
pub fn load_achievements(
    conn: &Connection,
    steam_appid: u32,
) -> Result<Vec<Achievement>, SpotterError> {
    let mut stmt = conn.prepare(
        "SELECT api_name, display_name, description,
                    icon_url, icon_gray_url, unlocked, unlock_time
             FROM achievements
             WHERE steam_appid = ?1
             ORDER BY id",
    )?;

    let rows = stmt.query_map(params![steam_appid], |row| {
        Ok(Achievement {
            api_name: row.get(0)?,
            display_name: row.get(1)?,
            description: row.get(2)?,
            icon_url: row.get(3)?,
            icon_gray_url: row.get(4)?,
            unlocked: row.get::<_, i32>(5)? != 0,
            unlock_time: row.get::<_, i64>(6)? as u64,
        })
    })?;

    let mut result = Vec::with_capacity(64);
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

/// Replace all achievements for a given platform-specific ID (e.g. Xbox titleId).
pub fn save_achievements_by_platform(
    conn: &Connection,
    platform_id: &str,
    achievements: &[Achievement],
) -> Result<(), SpotterError> {
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "DELETE FROM achievements WHERE platform_id = ?1",
        params![platform_id],
    )?;

    let mut stmt = tx.prepare(
        "INSERT INTO achievements
         (steam_appid, platform_id, api_name, display_name, description,
          icon_url, icon_gray_url, unlocked, unlock_time)
         VALUES (0, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )?;

    for a in achievements {
        stmt.execute(params![
            platform_id,
            a.api_name,
            a.display_name,
            a.description,
            a.icon_url,
            a.icon_gray_url,
            a.unlocked as i32,
            a.unlock_time as i64,
        ])?;
    }

    drop(stmt);
    tx.commit()?;
    Ok(())
}

/// Load achievements by platform-specific ID (e.g. Xbox titleId).
pub fn load_achievements_by_platform(
    conn: &Connection,
    platform_id: &str,
) -> Result<Vec<Achievement>, SpotterError> {
    let mut stmt = conn.prepare(
        "SELECT api_name, display_name, description,
                icon_url, icon_gray_url, unlocked, unlock_time
         FROM achievements
         WHERE platform_id = ?1
         ORDER BY id",
    )?;

    let rows = stmt.query_map(params![platform_id], |row| {
        Ok(Achievement {
            api_name: row.get(0)?,
            display_name: row.get(1)?,
            description: row.get(2)?,
            icon_url: row.get(3)?,
            icon_gray_url: row.get(4)?,
            unlocked: row.get::<_, i32>(5)? != 0,
            unlock_time: row.get::<_, i64>(6)? as u64,
        })
    })?;

    let mut result = Vec::with_capacity(64);
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}
