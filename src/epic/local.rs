use std::path::PathBuf;

use super::api::fetch_store_metadata;
use super::types::EpicManifest;
use crate::models::*;

/// Return possible directories where Epic Games Launcher stores manifest files.
fn manifest_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // Windows: C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests\
    #[cfg(target_os = "windows")]
    {
        if let Some(pd) = std::env::var_os("PROGRAMDATA") {
            dirs.push(
                PathBuf::from(pd)
                    .join("Epic")
                    .join("EpicGamesLauncher")
                    .join("Data")
                    .join("Manifests"),
            );
        }
    }

    // macOS: /Users/Shared/Epic Games/...
    #[cfg(target_os = "macos")]
    {
        dirs.push(PathBuf::from(
            "/Users/Shared/Epic Games/Launcher/Data/Manifests",
        ));
    }

    // Linux: common Heroic/Legendary locations
    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            // Heroic Games Launcher
            dirs.push(home.join(".config/heroic/store_cache"));
            // Legendary (CLI launcher)
            dirs.push(
                home.join(".config/legendary/installed.json")
                    .parent()
                    .unwrap_or(&home)
                    .to_path_buf(),
            );
        }
    }

    // Also try standard Windows path under Wine/Proton (Linux only)
    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".wine/drive_c/ProgramData/Epic/EpicGamesLauncher/Data/Manifests"));
        }
    }

    dirs
}

/// Scan local Epic Games Launcher manifest files for installed games.
fn scan_local_manifests() -> Vec<EpicManifest> {
    let mut manifests = Vec::new();

    for dir in manifest_dirs() {
        if !dir.exists() {
            continue;
        }
        eprintln!("[epic] Scanning manifest directory: {:?}", dir);

        // Handle the Legendary installed.json format
        if dir.join("installed.json").exists() {
            if let Ok(content) = std::fs::read_to_string(dir.join("installed.json")) {
                if let Ok(map) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(obj) = map.as_object() {
                        for (_key, val) in obj {
                            if let (Some(title), Some(app_name)) = (
                                val.get("title").and_then(|v| v.as_str()),
                                val.get("app_name").and_then(|v| v.as_str()),
                            ) {
                                manifests.push(EpicManifest {
                                    display_name: title.to_string(),
                                    catalog_item_id: val
                                        .get("catalog_item_id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    catalog_namespace: val
                                        .get("namespace")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    app_name: app_name.to_string(),
                                    app_categories: vec!["games".to_string()],
                                });
                            }
                        }
                    }
                }
            }
            continue;
        }

        // Standard .item manifest files
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "item") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(manifest) = serde_json::from_str::<EpicManifest>(&content) {
                    // Only include actual games
                    if manifest.app_categories.iter().any(|c| c == "games")
                        && !manifest.display_name.is_empty()
                    {
                        manifests.push(manifest);
                    }
                }
            }
        }
    }

    manifests
}

/// Local-only import (no account needed). Scans filesystem for installed games.
pub fn full_import_local() -> Result<Vec<Game>, String> {
    let manifests = scan_local_manifests();
    if manifests.is_empty() {
        return Err(
            "No Epic Games found locally. Make sure the Epic Games Launcher is installed, \
             or use Heroic/Legendary on Linux."
                .into(),
        );
    }

    let logger = crate::api_client::ImportLogger::new(&crate::api_client::log_dir());
    logger.log("epic", "Starting Epic Games local import");

    eprintln!("[epic] Found {} local games", manifests.len());
    logger.log("epic", &format!("Found {} local games", manifests.len()));

    let mut games = Vec::new();
    let mut limiter = crate::api_client::RateLimiter::new(std::time::Duration::from_millis(300));

    for (i, manifest) in manifests.iter().enumerate() {
        eprintln!(
            "[epic] [{}/{}] {}",
            i + 1,
            manifests.len(),
            manifest.display_name
        );

        let (description, cover, tags, release_date, was_rate_limited) =
            fetch_store_metadata(&manifest.catalog_namespace, &manifest.catalog_item_id);

        let epic_id = if !manifest.catalog_item_id.is_empty() {
            Some(manifest.catalog_item_id.clone())
        } else if !manifest.app_name.is_empty() {
            Some(manifest.app_name.clone())
        } else {
            None
        };

        games.push(Game {
            id: None,
            title: manifest.display_name.clone(),
            platform: Platform::Epic,
            playtime_minutes: 0,
            achievements_unlocked: 0,
            achievements_total: 0,
            status: GameStatus::Unplayed,
            rating: None,
            genre: String::new(),
            last_played: String::new(),
            cover_url: cover,
            steam_appid: None,
            gog_id: None,
            epic_id,
            xbox_id: None,
            psn_id: None,
            notes: String::new(),
            description,
            release_date,
            review_percent: None,
            tags,
        });

        if !manifest.catalog_namespace.is_empty() {
            if was_rate_limited {
                limiter.on_rate_limited();
            } else {
                limiter.on_success();
            }
            limiter.wait();
        }
    }

    logger.log(
        "epic",
        &format!("Local import complete: {} games", games.len()),
    );
    Ok(games)
}
