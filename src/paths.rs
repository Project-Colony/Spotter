//! Where Spotter keeps its files.
//!
//! The three roots come from `colony_ui::paths` (`Colony/Spotter/` under the
//! platform's config, data and cache directories), and each kind of file has
//! its own sub-directory, because config and data are the same directory on
//! Windows and macOS:
//!
//! | File | Where |
//! |---|---|
//! | Preferences | `<config>/preferences/preferences.json` |
//! | Tokens, when no system keyring is usable | `<config>/credentials/tokens.json` |
//! | Library database | `<data>/library/spotter.db` |
//! | Exports and the automatic backup | `<data>/exports/` |
//! | Import log | `<data>/logs/` |
//! | Covers and achievement icons | `<cache>/covers/`, `<cache>/achievement_icons/` |
//!
//! Spotter 0.3 and earlier kept everything under the config root instead
//! (`data/spotter.db`, `exports/`, `logs/`, `cache/`). The first start copies
//! those files over, see [`migrate`].

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const PROGRAM: &str = "Spotter";
const DB_FILE: &str = "spotter.db";
/// Written once the files of Spotter 0.3 and earlier were looked for, so the
/// copy runs at most once even if the new library is deleted later.
const MIGRATED_MARKER: &str = ".migrated-from-0.3";

/// Every path Spotter reads or writes.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub db: PathBuf,
    pub exports: PathBuf,
    pub logs: PathBuf,
    pub covers: PathBuf,
    pub achievement_icons: PathBuf,
    pub preferences: PathBuf,
    pub credentials: PathBuf,
}

static LAYOUT: OnceLock<Result<Layout, String>> = OnceLock::new();

/// Spotter's paths, resolved on first use: the directories are created, and
/// the files of an older Spotter are copied over first.
pub fn layout() -> Result<&'static Layout, String> {
    LAYOUT
        .get_or_init(|| {
            let root = |dir: io::Result<PathBuf>| {
                dir.map_err(|e| format!("cannot use Spotter's folder: {}", e))
            };
            Layout::resolve(
                &root(colony_ui::paths::config_dir(PROGRAM))?,
                &root(colony_ui::paths::data_dir(PROGRAM))?,
                &root(colony_ui::paths::cache_dir(PROGRAM))?,
            )
        })
        .as_ref()
        .map_err(Clone::clone)
}

impl Layout {
    /// The current layout under these roots.
    fn current(config: &Path, data: &Path, cache: &Path) -> Self {
        Self {
            db: data.join("library").join(DB_FILE),
            exports: data.join("exports"),
            logs: data.join("logs"),
            covers: cache.join("covers"),
            achievement_icons: cache.join("achievement_icons"),
            preferences: config.join("preferences").join("preferences.json"),
            credentials: config.join("credentials").join("tokens.json"),
        }
    }

    /// Where Spotter 0.3 and earlier kept the library, its exports and its
    /// log: under the config root, which `colony_ui::paths::config_dir`
    /// resolves to the same directory those versions used on every platform.
    /// Covers stay in the current cache: they are downloaded again.
    fn legacy(config: &Path, data: &Path, cache: &Path) -> Self {
        Self {
            db: config.join("data").join(DB_FILE),
            exports: config.join("exports"),
            logs: config.join("logs"),
            ..Self::current(config, data, cache)
        }
    }

    /// The layout to use under these roots, after copying the files of an
    /// older Spotter. A copy that fails leaves those files where they are and
    /// uses them from there, so a failed migration never shows an empty
    /// library; the next start tries again.
    fn resolve(config: &Path, data: &Path, cache: &Path) -> Result<Self, String> {
        let current = Self::current(config, data, cache);
        let layout = match migrate(&Self::legacy(config, data, cache), &current, data) {
            Ok(()) => current,
            Err(e) => {
                eprintln!(
                    "[paths] Could not copy the files of an older Spotter ({}). Using them where they are.",
                    e
                );
                Self::legacy(config, data, cache)
            }
        };
        layout
            .create_dirs()
            .map_err(|e| format!("cannot create Spotter's folders: {}", e))?;
        Ok(layout)
    }

    fn create_dirs(&self) -> io::Result<()> {
        for dir in [
            &self.exports,
            &self.logs,
            &self.covers,
            &self.achievement_icons,
        ] {
            fs::create_dir_all(dir)?;
        }
        for file in [&self.db, &self.preferences, &self.credentials] {
            if let Some(dir) = file.parent() {
                fs::create_dir_all(dir)?;
            }
        }
        Ok(())
    }
}

/// Copy the library, exports and log of Spotter 0.3 and earlier (`old`) to
/// the current layout (`new`), once. Nothing is moved or deleted: if this
/// release is wrong, the old files are still there. The library is copied
/// last, so the new library only exists once everything else is in place.
fn migrate(old: &Layout, new: &Layout, data: &Path) -> Result<(), String> {
    let marker = data.join(MIGRATED_MARKER);
    if marker.exists() {
        return Ok(());
    }
    if old.db.exists() && !new.db.exists() {
        copy_new_files(&old.exports, &new.exports).map_err(|e| format!("exports: {}", e))?;
        copy_new_files(&old.logs, &new.logs).map_err(|e| format!("logs: {}", e))?;
        copy_db(&old.db, &new.db)?;
    }
    // Only an optimisation once the library is in place: without the marker,
    // the next start finds the new library and copies nothing.
    if let Err(e) = fs::write(&marker, "") {
        eprintln!("[paths] Could not write {:?}: {}", marker, e);
    }
    Ok(())
}

/// Copy the files directly in `from` that `to` does not have yet. Nothing to
/// do when there is no `from`, or when both are the same directory (exports
/// and logs on Windows and macOS).
fn copy_new_files(from: &Path, to: &Path) -> io::Result<()> {
    if from == to || !from.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_file() && !target.exists() {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Copy a database with SQLite itself rather than file by file: the copy is
/// one consistent file that includes what is still in the old one's
/// write-ahead log, and the old database is only read.
fn copy_db(from: &Path, to: &Path) -> Result<(), String> {
    use rusqlite::{Connection, OpenFlags};

    let dir = to.parent().ok_or("no library folder")?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    // A copy an interrupted start left behind: VACUUM INTO needs a new file.
    let part = dir.join(format!("{}.part", DB_FILE));
    let _ = fs::remove_file(&part);
    let part_name = part.to_str().ok_or("the library path is not valid UTF-8")?;
    let conn = Connection::open_with_flags(from, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| e.to_string())?;
    conn.execute("VACUUM INTO ?1", [part_name])
        .map_err(|e| e.to_string())?;
    drop(conn);
    fs::rename(&part, to).map_err(|e| e.to_string())
}

/// Replace `path` with `bytes` through a temporary file, so a crash never
/// leaves half a file. `private` makes it readable by the user only on Unix.
pub fn write_atomic(path: &Path, bytes: &[u8], private: bool) -> io::Result<()> {
    let tmp = path.with_extension("tmp");
    // A leftover would keep its permissions: start from a new file.
    let _ = fs::remove_file(&tmp);
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    if private {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    #[cfg(not(unix))]
    let _ = private;
    let mut file = options.open(&tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// A fresh directory with config, data and cache roots inside.
    struct Roots {
        base: PathBuf,
        config: PathBuf,
        data: PathBuf,
        cache: PathBuf,
    }

    impl Roots {
        fn new(name: &str) -> Self {
            let base =
                std::env::temp_dir().join(format!("spotter_paths_{}_{}", name, std::process::id()));
            let _ = fs::remove_dir_all(&base);
            let roots = Self {
                config: base.join("config"),
                data: base.join("data"),
                cache: base.join("cache"),
                base,
            };
            for dir in [&roots.config, &roots.data, &roots.cache] {
                fs::create_dir_all(dir).unwrap();
            }
            roots
        }

        fn resolve(&self) -> Layout {
            Layout::resolve(&self.config, &self.data, &self.cache).unwrap()
        }

        fn current(&self) -> Layout {
            Layout::current(&self.config, &self.data, &self.cache)
        }

        fn legacy(&self) -> Layout {
            Layout::legacy(&self.config, &self.data, &self.cache)
        }

        /// What Spotter 0.3 left: a library with one game in WAL mode, with
        /// the game still in the write-ahead log, plus an export and a log.
        /// The connection is returned so the log is not checkpointed away.
        fn spotter_03(&self) -> Connection {
            let old = self.legacy();
            fs::create_dir_all(old.db.parent().unwrap()).unwrap();
            let conn = Connection::open(&old.db).unwrap();
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 CREATE TABLE games (title TEXT);
                 INSERT INTO games VALUES ('Outer Wilds');",
            )
            .unwrap();
            fs::create_dir_all(&old.exports).unwrap();
            fs::write(old.exports.join("spotter_auto_backup.json"), "[]").unwrap();
            fs::create_dir_all(&old.logs).unwrap();
            fs::write(old.logs.join("import.log"), "old log").unwrap();
            conn
        }
    }

    impl Drop for Roots {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.base);
        }
    }

    fn titles(db: &Path) -> Vec<String> {
        let conn = Connection::open(db).unwrap();
        let mut stmt = conn.prepare("SELECT title FROM games").unwrap();
        let rows = stmt.query_map([], |r| r.get(0)).unwrap();
        rows.map(Result::unwrap).collect()
    }

    #[test]
    fn a_fresh_install_uses_the_current_layout() {
        let roots = Roots::new("fresh");
        let layout = roots.resolve();
        assert_eq!(layout, roots.current());
        assert!(layout.db.parent().unwrap().is_dir());
        assert!(layout.preferences.parent().unwrap().is_dir());
        assert!(layout.covers.is_dir() && layout.logs.is_dir());
        assert!(roots.data.join(MIGRATED_MARKER).exists());
        assert!(!roots.legacy().db.exists(), "nothing to copy from");
    }

    #[test]
    fn spotter_03_files_are_copied_and_left_in_place() {
        let roots = Roots::new("legacy");
        let _open = roots.spotter_03();
        let old = roots.legacy();

        let layout = roots.resolve();
        assert_eq!(layout, roots.current());
        assert_eq!(titles(&layout.db), ["Outer Wilds"]);
        assert_eq!(
            fs::read_to_string(layout.logs.join("import.log")).unwrap(),
            "old log"
        );
        assert!(layout.exports.join("spotter_auto_backup.json").exists());
        assert!(roots.data.join(MIGRATED_MARKER).exists());

        // Copied, never moved.
        assert_eq!(titles(&old.db), ["Outer Wilds"]);
        assert!(old.logs.join("import.log").exists());
        assert!(old.exports.join("spotter_auto_backup.json").exists());
    }

    #[test]
    fn a_migrated_install_is_not_copied_again() {
        let roots = Roots::new("migrated");
        let _open = roots.spotter_03();
        let layout = roots.resolve();

        // The user empties the new library; the old one must not come back,
        // even without the marker as long as the new library exists.
        Connection::open(&layout.db)
            .unwrap()
            .execute("DELETE FROM games", [])
            .unwrap();
        assert_eq!(roots.resolve(), roots.current());
        assert!(titles(&layout.db).is_empty());

        fs::remove_file(roots.data.join(MIGRATED_MARKER)).unwrap();
        assert_eq!(roots.resolve(), roots.current());
        assert!(titles(&layout.db).is_empty());

        // With the marker, not even a deleted library brings it back.
        fs::remove_file(&layout.db).unwrap();
        assert_eq!(roots.resolve(), roots.current());
        assert!(!layout.db.exists());
    }

    #[test]
    fn a_failed_copy_keeps_using_the_old_files() {
        let roots = Roots::new("failed");
        let _open = roots.spotter_03();
        // A file where the library folder should be: the copy cannot happen.
        fs::write(roots.data.join("library"), "").unwrap();

        let layout = roots.resolve();
        assert_eq!(layout.db, roots.legacy().db);
        assert_eq!(layout.exports, roots.legacy().exports);
        assert_eq!(titles(&layout.db), ["Outer Wilds"]);
        assert!(
            !roots.data.join(MIGRATED_MARKER).exists(),
            "the next start tries again"
        );

        // Once the obstacle is gone, the next start copies.
        fs::remove_file(roots.data.join("library")).unwrap();
        assert_eq!(roots.resolve(), roots.current());
        assert_eq!(titles(&roots.current().db), ["Outer Wilds"]);
    }

    #[test]
    fn same_directory_exports_are_left_alone() {
        // Windows and macOS: config and data are one root, so the old and new
        // exports folders are the same.
        let dir = std::env::temp_dir().join(format!("spotter_same_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.json"), "a").unwrap();
        copy_new_files(&dir, &dir).unwrap();
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn existing_files_are_not_overwritten() {
        let roots = Roots::new("no_overwrite");
        let (from, to) = (roots.base.join("from"), roots.base.join("to"));
        fs::create_dir_all(&from).unwrap();
        fs::create_dir_all(&to).unwrap();
        fs::write(from.join("x"), "old").unwrap();
        fs::write(to.join("x"), "new").unwrap();
        copy_new_files(&from, &to).unwrap();
        assert_eq!(fs::read_to_string(to.join("x")).unwrap(), "new");
    }

    #[test]
    #[cfg(unix)]
    fn private_files_are_readable_by_the_user_only() {
        use std::os::unix::fs::PermissionsExt;
        let roots = Roots::new("private");
        let path = roots.config.join("tokens.json");
        write_atomic(&path, b"{}", true).unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
        write_atomic(&path, b"{\"a\":\"b\"}", true).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"a\":\"b\"}");
    }
}
