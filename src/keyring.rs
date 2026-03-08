/// OS-level credential storage for sensitive tokens.
///
/// Uses the platform's native secret store:
///   - Windows: Credential Manager
///   - macOS: Keychain
///   - Linux: Secret Service (libsecret / KWallet)
///
/// Each secret is stored under the service name "spotter" with a per-field key.
/// All operations are best-effort — errors are logged but never surface to the caller,
/// so the DB fallback path always remains valid.
use keyring::Entry;

const SERVICE: &str = "spotter";

/// Named credential slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialKey {
    SteamApiKey,
    GogToken,
    GogRefreshToken,
    XboxApiKey,
    PsnNpsso,
    EpicToken,
    EpicRefreshToken,
}

impl CredentialKey {
    fn as_str(self) -> &'static str {
        match self {
            Self::SteamApiKey => "steam_api_key",
            Self::GogToken => "gog_token",
            Self::GogRefreshToken => "gog_refresh_token",
            Self::XboxApiKey => "xbox_api_key",
            Self::PsnNpsso => "psn_npsso",
            Self::EpicToken => "epic_token",
            Self::EpicRefreshToken => "epic_refresh_token",
        }
    }
}

/// Store a secret in the OS keyring.
/// Returns `true` on success, `false` if the keyring is unavailable.
pub fn store(key: CredentialKey, secret: &str) -> bool {
    match Entry::new(SERVICE, key.as_str()) {
        Ok(entry) => {
            if secret.is_empty() {
                // Treat empty string as "delete the credential"
                match entry.delete_credential() {
                    Ok(()) => true,
                    Err(keyring::Error::NoEntry) => true, // Already absent — not an error
                    Err(e) => {
                        eprintln!("[keyring] Failed to delete {}: {}", key.as_str(), e);
                        false
                    }
                }
            } else {
                match entry.set_password(secret) {
                    Ok(()) => true,
                    Err(e) => {
                        eprintln!("[keyring] Failed to store {}: {}", key.as_str(), e);
                        false
                    }
                }
            }
        }
        Err(e) => {
            eprintln!(
                "[keyring] Failed to create entry for {}: {}",
                key.as_str(),
                e
            );
            false
        }
    }
}

/// Retrieve a secret from the OS keyring.
/// Returns `None` if the credential does not exist or the keyring is unavailable.
pub fn load(key: CredentialKey) -> Option<String> {
    let entry = Entry::new(SERVICE, key.as_str())
        .map_err(|e| {
            eprintln!(
                "[keyring] Failed to create entry for {}: {}",
                key.as_str(),
                e
            )
        })
        .ok()?;

    match entry.get_password() {
        Ok(secret) => Some(secret),
        Err(keyring::Error::NoEntry) => None,
        Err(e) => {
            eprintln!("[keyring] Failed to load {}: {}", key.as_str(), e);
            None
        }
    }
}

/// Delete a stored credential.
/// Returns `true` on success or if the entry did not exist.
#[allow(dead_code)]
pub fn delete(key: CredentialKey) -> bool {
    match Entry::new(SERVICE, key.as_str()) {
        Ok(entry) => match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => true,
            Err(e) => {
                eprintln!("[keyring] Failed to delete {}: {}", key.as_str(), e);
                false
            }
        },
        Err(e) => {
            eprintln!(
                "[keyring] Failed to create entry for {}: {}",
                key.as_str(),
                e
            );
            false
        }
    }
}

/// Store all sensitive fields from a UserProfile into the OS keyring.
/// Returns `true` if all writes succeeded.
pub fn store_profile_secrets(profile: &crate::models::UserProfile) -> bool {
    let results = [
        store(CredentialKey::SteamApiKey, &profile.steam_api_key),
        store(CredentialKey::GogToken, &profile.gog_token),
        store(CredentialKey::GogRefreshToken, &profile.gog_refresh_token),
        store(CredentialKey::XboxApiKey, &profile.xbox_api_key),
        store(CredentialKey::PsnNpsso, &profile.psn_npsso),
        store(CredentialKey::EpicToken, &profile.epic_token),
        store(CredentialKey::EpicRefreshToken, &profile.epic_refresh_token),
    ];
    results.iter().all(|&ok| ok)
}

/// Load all sensitive fields from the OS keyring into a UserProfile.
/// Fields that are not found in the keyring are left unchanged (DB fallback).
pub fn load_profile_secrets(profile: &mut crate::models::UserProfile) {
    if let Some(v) = load(CredentialKey::SteamApiKey) {
        profile.steam_api_key = v;
    }
    if let Some(v) = load(CredentialKey::GogToken) {
        profile.gog_token = v;
    }
    if let Some(v) = load(CredentialKey::GogRefreshToken) {
        profile.gog_refresh_token = v;
    }
    if let Some(v) = load(CredentialKey::XboxApiKey) {
        profile.xbox_api_key = v;
    }
    if let Some(v) = load(CredentialKey::PsnNpsso) {
        profile.psn_npsso = v;
    }
    if let Some(v) = load(CredentialKey::EpicToken) {
        profile.epic_token = v;
    }
    if let Some(v) = load(CredentialKey::EpicRefreshToken) {
        profile.epic_refresh_token = v;
    }
}
