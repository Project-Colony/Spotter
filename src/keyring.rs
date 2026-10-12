//! OS-level credential storage for the platform tokens.
//!
//! Uses the platform's native secret store:
//!   - Windows: Credential Manager
//!   - macOS: Keychain
//!   - Linux: Secret Service (GNOME Keyring, KWallet, KeePassXC...)
//!
//! Each secret is stored under the service name "spotter" with a per-field
//! key. When no keyring is usable, the secrets go to a file only the user can
//! read (`paths::Layout::credentials`) instead, unencrypted, so they are never
//! lost; the Profile page says so. Spotter 0.3 and earlier kept that fallback
//! in the database: those columns are emptied once the secrets are safe in the
//! keyring or the file.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use keyring_core::{Entry, Error};
use rusqlite::Connection;

use crate::db;
use crate::error::SpotterError;
use crate::models::UserProfile;

const SERVICE: &str = "spotter";

/// Set on the first keyring failure. For the rest of the session Spotter
/// leaves the keyring alone: secrets are saved to the file, and nothing
/// is deleted from a keyring whose contents could not be read.
static UNAVAILABLE: AtomicBool = AtomicBool::new(false);

/// The secrets this session read from the keyring or stored in it. Only these
/// are ever deleted: a locked store can answer "no entry" (KeePassXC does), and
/// the empty value loaded then must not delete the real one on a later save.
static SEEN: Mutex<BTreeSet<&'static str>> = Mutex::new(BTreeSet::new());

fn seen() -> std::sync::MutexGuard<'static, BTreeSet<&'static str>> {
    SEEN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Held for a whole save, so a read-back that differs means a broken keyring
/// rather than another save running at the same time.
static SAVING: Mutex<()> = Mutex::new(());

/// Make the platform keyring the default store. Called once at startup.
pub fn init() {
    if let Err(e) = ::keyring::Entry::store_status() {
        unavailable("open the system keyring", e);
    }
}

/// True when this session keeps the tokens unencrypted in the file.
pub fn is_unavailable() -> bool {
    UNAVAILABLE.load(Ordering::Relaxed)
}

fn unavailable(action: &str, e: impl std::fmt::Display) {
    if !UNAVAILABLE.swap(true, Ordering::Relaxed) {
        eprintln!(
            "[keyring] Failed to {}: {}. Platform tokens are stored unencrypted in a file.",
            action, e
        );
    }
}

/// The profile fields that hold secrets, with their keyring names.
fn secrets(p: &mut UserProfile) -> [(&'static str, &mut String); 7] {
    [
        ("steam_api_key", &mut p.steam_api_key),
        ("gog_token", &mut p.gog_token),
        ("gog_refresh_token", &mut p.gog_refresh_token),
        ("xbox_api_key", &mut p.xbox_api_key),
        ("psn_npsso", &mut p.psn_npsso),
        ("epic_token", &mut p.epic_token),
        ("epic_refresh_token", &mut p.epic_refresh_token),
    ]
}

fn entry(name: &str) -> Option<Entry> {
    if is_unavailable() {
        return None;
    }
    Entry::new(SERVICE, name)
        .map_err(|e| unavailable(&format!("open {}", name), e))
        .ok()
}

/// The stored secret ("" when there is none), or `None` if it cannot be read.
fn read(name: &'static str) -> Option<String> {
    match entry(name)?.get_secret() {
        Ok(bytes) => {
            seen().insert(name);
            String::from_utf8(bytes)
                .map_err(|e| unavailable(&format!("read {}", name), e))
                .ok()
        }
        Err(Error::NoEntry) => Some(String::new()),
        Err(e) => {
            unavailable(&format!("read {}", name), e);
            None
        }
    }
}

/// Make the keyring hold `secret` (an empty one deletes the entry, if this
/// session saw it), and only report success once reading it back returns that
/// same value.
fn write(name: &'static str, secret: &str) -> bool {
    if secret.is_empty() && !seen().contains(name) {
        return true;
    }
    if read(name).as_deref() == Some(secret) {
        return true;
    }
    let Some(entry) = entry(name) else {
        return false;
    };
    let result = if secret.is_empty() {
        entry.delete_credential()
    } else {
        // Raw UTF-8 bytes: Windows stores passwords as UTF-16, which halves
        // Credential Manager's 2560-byte limit, and Epic's tokens are long JWTs.
        entry.set_secret(secret.as_bytes())
    };
    if let Err(e) = result {
        unavailable(&format!("write {}", name), e);
        return false;
    }
    if read(name).as_deref() != Some(secret) {
        unavailable(
            &format!("verify {}", name),
            "the keyring returned another value",
        );
        return false;
    }
    true
}

/// The secrets in the fallback file, by keyring name. No file means none.
fn read_file(file: &Path) -> Result<BTreeMap<String, String>, SpotterError> {
    match std::fs::read(file) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(e) => Err(e.into()),
    }
}

/// Make the fallback file hold exactly `secrets` (no file when there are
/// none), and only report success once reading it back returns them.
fn write_file(file: &Path, secrets: &BTreeMap<String, String>) -> Result<(), SpotterError> {
    if secrets.is_empty() {
        return match std::fs::remove_file(file) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
            _ => Ok(()),
        };
    }
    crate::paths::write_atomic(file, &serde_json::to_vec_pretty(secrets)?, true)?;
    if read_file(file)? != *secrets {
        return Err(SpotterError::Other("the file read back differently".into()));
    }
    Ok(())
}

/// Load the profile with its secrets from the keyring. Secrets still in the
/// database (Spotter 0.3 and earlier) or in the file (saved while no keyring
/// was usable) are newer than the keyring's, and are moved into it first.
pub fn load_profile(conn: &Connection, file: &Path) -> Result<UserProfile, SpotterError> {
    let mut profile = db::load_profile(conn)?;
    let stored = read_file(file)?;
    let mut in_db = false;
    for (name, value) in secrets(&mut profile) {
        if !value.is_empty() {
            in_db = true;
        } else if let Some(secret) = stored.get(name) {
            value.clone_from(secret);
        } else if let Some(secret) = read(name) {
            *value = secret;
        }
    }
    if in_db || (!stored.is_empty() && !is_unavailable()) {
        save_profile(conn, file, &profile)?;
    }
    Ok(profile)
}

/// Save the profile with its secrets in the keyring and empty in the database.
/// A secret the keyring did not take goes to the file instead, and stays in
/// the database only if the file cannot be written.
pub fn save_profile(
    conn: &Connection,
    file: &Path,
    profile: &UserProfile,
) -> Result<(), SpotterError> {
    let _saving = SAVING.lock().unwrap_or_else(PoisonError::into_inner);
    let mut row = profile.clone();
    let mut kept = BTreeMap::new();
    for (name, value) in secrets(&mut row) {
        let secret = std::mem::take(value);
        if !write(name, &secret) && !secret.is_empty() {
            kept.insert(name.to_string(), secret);
        }
    }
    if let Err(e) = write_file(file, &kept) {
        eprintln!(
            "[keyring] Failed to save the tokens to {:?}: {}. They stay in the database.",
            file, e
        );
        for (name, value) in secrets(&mut row) {
            if let Some(secret) = kept.remove(name) {
                *value = secret;
            }
        }
    }
    db::save_profile(conn, &row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyring_core::mock;

    #[test]
    fn secrets_move_to_the_keyring_or_to_the_file() {
        let dir = std::env::temp_dir().join(format!("spotter_keyring_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("tokens.json");
        let conn = db::open_at(&dir.join("spotter.db")).unwrap();
        let in_db = |conn: &Connection| db::load_profile(conn).unwrap();
        let in_file = |name: &str| read_file(&file).unwrap().get(name).cloned();

        // What Spotter 0.3 and earlier left in the database.
        let legacy = UserProfile {
            steam_api_key: "steam-key".into(),
            gog_refresh_token: "gog-refresh".into(),
            ..UserProfile::default()
        };
        db::save_profile(&conn, &legacy).unwrap();

        // No keyring: the secrets move to the file, readable by the user only.
        keyring_core::unset_default_store();
        let p = load_profile(&conn, &file).unwrap();
        assert_eq!(p.steam_api_key, "steam-key");
        assert_eq!(in_file("steam_api_key").as_deref(), Some("steam-key"));
        assert_eq!(in_db(&conn).steam_api_key, "");
        assert!(is_unavailable());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&file).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }

        // A file that cannot be written: the secret stays in the database.
        let unwritable = dir.join("missing").join("tokens.json");
        let p2 = UserProfile {
            psn_npsso: "psn-old".into(),
            ..p.clone()
        };
        save_profile(&conn, &unwritable, &p2).unwrap();
        assert_eq!(in_db(&conn).psn_npsso, "psn-old");
        // The next load puts it in a file that works.
        let p = load_profile(&conn, &file).unwrap();
        assert_eq!(p.psn_npsso, "psn-old");
        assert_eq!(in_file("psn_npsso").as_deref(), Some("psn-old"));
        assert_eq!(in_db(&conn).psn_npsso, "");

        // A keyring: the next start moves the secrets there and removes the file.
        UNAVAILABLE.store(false, Ordering::Relaxed);
        keyring_core::set_default_store(mock::Store::new().unwrap());
        let mut p = load_profile(&conn, &file).unwrap();
        assert_eq!(p.steam_api_key, "steam-key");
        assert_eq!(p.gog_refresh_token, "gog-refresh");
        assert!(!file.exists());
        assert_eq!(in_db(&conn).steam_api_key, "");
        assert!(!is_unavailable());

        // Saving writes the keyring only; an emptied secret is deleted.
        p.steam_api_key.clear();
        p.psn_npsso.clear();
        p.xbox_api_key = "xbox-key".into();
        save_profile(&conn, &file, &p).unwrap();
        assert_eq!(in_db(&conn).xbox_api_key, "");
        assert!(!file.exists());
        let p = load_profile(&conn, &file).unwrap();
        assert_eq!(p.steam_api_key, "");
        assert_eq!(p.xbox_api_key, "xbox-key");
        assert_eq!(p.gog_refresh_token, "gog-refresh");

        // A new session whose locked keyring answers "no entry": the secret
        // loads empty, and a save does not delete it.
        seen().clear();
        let gog = Entry::new(SERVICE, "gog_refresh_token").unwrap();
        let cred: &mock::Cred = gog.as_any().downcast_ref().unwrap();
        cred.set_error(Error::NoEntry);
        let p = load_profile(&conn, &file).unwrap();
        assert_eq!(p.gog_refresh_token, "");
        save_profile(&conn, &file, &p).unwrap();
        assert_eq!(gog.get_secret().unwrap(), b"gog-refresh");
        assert!(!is_unavailable());

        // Saves running at the same time do not mistake each other's values
        // for a broken keyring.
        std::thread::scope(|s| {
            for t in 0..8 {
                let (conn, p, file) = (db::open_at(&dir.join("spotter.db")).unwrap(), &p, &file);
                s.spawn(move || {
                    for i in 0..200 {
                        let p = UserProfile {
                            steam_api_key: format!("steam-{}-{}", t, i),
                            ..p.clone()
                        };
                        save_profile(&conn, file, &p).unwrap();
                    }
                });
            }
        });
        assert!(!is_unavailable());
        assert_eq!(in_db(&conn).steam_api_key, "");

        // A keyring that fails to read: nothing is deleted from it, and
        // secrets saved afterwards go to the file.
        let xbox = Entry::new(SERVICE, "xbox_api_key").unwrap();
        let cred: &mock::Cred = xbox.as_any().downcast_ref().unwrap();
        cred.set_error(Error::NoStorageAccess("locked".into()));
        let mut p = load_profile(&conn, &file).unwrap();
        assert_eq!(p.xbox_api_key, "");
        assert!(is_unavailable());
        p.psn_npsso = "psn-token".into();
        save_profile(&conn, &file, &p).unwrap();
        assert_eq!(in_file("psn_npsso").as_deref(), Some("psn-token"));
        assert_eq!(in_db(&conn).psn_npsso, "");
        assert_eq!(xbox.get_secret().unwrap(), b"xbox-key");

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
