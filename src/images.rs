use crate::db;
use std::io::Read;
use std::path::PathBuf;

/// Maximum size for downloaded images (10 MB).
const MAX_IMAGE_SIZE: u64 = 10_000_000;

pub fn cover_path(game_title: &str) -> PathBuf {
    let safe_name: String = game_title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == ' ' || c == '-' {
                c
            } else if c.is_alphanumeric() {
                // Keep Unicode letters/digits as-is for better readability
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .replace(' ', "_")
        .to_lowercase();
    // Truncate overly long names (Windows MAX_PATH safety) while keeping uniqueness
    let truncated = if safe_name.len() > 120 {
        // Use a simple hash suffix to prevent collisions on truncation
        let hash = simple_hash(game_title);
        format!("{}_{:x}", &safe_name[..100], hash)
    } else {
        safe_name
    };
    db::covers_dir().join(format!("{}.jpg", truncated))
}

/// Simple non-cryptographic hash for filename deduplication.
fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub fn download_cover(url: &str, game_title: &str) -> Result<PathBuf, String> {
    if url.is_empty() {
        return Err("No cover URL".into());
    }

    let path = cover_path(game_title);
    if path.exists() {
        return Ok(path);
    }

    let agent = crate::api_client::download_agent();
    let resp = agent
        .get(url)
        .call()
        .map_err(|e| format!("Download error: {}", e))?;

    let mut bytes = Vec::new();
    resp.into_body()
        .into_reader()
        .take(MAX_IMAGE_SIZE)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Read error: {}", e))?;

    // Validate that the downloaded data looks like an actual image
    if !is_valid_image(&bytes) {
        return Err("Downloaded file is not a recognized image format".into());
    }

    std::fs::write(&path, &bytes).map_err(|e| format!("Write error: {}", e))?;

    Ok(path)
}

/// Check image magic bytes (JPEG, PNG, WebP, GIF).
fn is_valid_image(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    // JPEG: FF D8 FF
    if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return true;
    }
    // PNG: 89 50 4E 47
    if data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 {
        return true;
    }
    // WebP: RIFF....WEBP
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return true;
    }
    // GIF: GIF87a or GIF89a
    if data.len() >= 6 && &data[0..3] == b"GIF" {
        return true;
    }
    false
}

// ───── Achievement icons ─────

pub fn achievement_icons_dir() -> PathBuf {
    let dir = db::cache_dir().join("achievement_icons");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!(
            "[images] Failed to create achievement icons dir {:?}: {}",
            dir, e
        );
    }
    dir
}

/// Local path for a cached achievement icon.
/// `gray` selects between the unlocked (colored) and locked (grayscale) variant.
pub fn achievement_icon_path(appid: u32, api_name: &str, gray: bool) -> PathBuf {
    let safe_name: String = api_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .to_lowercase();
    let suffix = if gray { "_gray" } else { "" };
    achievement_icons_dir().join(format!("{}_{}{}.jpg", appid, safe_name, suffix))
}

/// Download a single achievement icon if not already cached.
fn download_icon(url: &str, path: &std::path::Path) -> Result<(), String> {
    if url.is_empty() {
        return Err("No icon URL".into());
    }
    if path.exists() {
        return Ok(());
    }
    let agent = crate::api_client::download_agent();
    let resp = agent
        .get(url)
        .call()
        .map_err(|e| format!("Download error: {}", e))?;
    let mut bytes = Vec::new();
    resp.into_body()
        .into_reader()
        .take(MAX_IMAGE_SIZE)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Read error: {}", e))?;
    std::fs::write(path, &bytes).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

/// Download all missing achievement icons for a set of achievements.
/// Returns the number of icons successfully downloaded.
pub fn download_achievement_icons(appid: u32, achievements: &[crate::models::Achievement]) -> u32 {
    let mut count = 0u32;
    for ach in achievements {
        // Download the appropriate icon: colored for unlocked, gray for locked
        // We download both variants so switching states later doesn't require re-download
        let colored_path = achievement_icon_path(appid, &ach.api_name, false);
        if download_icon(&ach.icon_url, &colored_path).is_ok() {
            count += 1;
        }
        let gray_path = achievement_icon_path(appid, &ach.api_name, true);
        let _ = download_icon(&ach.icon_gray_url, &gray_path);
    }
    count
}

/// Lightweight variant that accepts only the data needed for icon downloads,
/// avoiding a full clone of Vec<Achievement>.
/// Each tuple is (api_name, icon_url, icon_gray_url).
pub fn download_achievement_icons_minimal(appid: u32, icons: &[(String, String, String)]) -> u32 {
    let mut count = 0u32;
    for (api_name, icon_url, icon_gray_url) in icons {
        let colored_path = achievement_icon_path(appid, api_name, false);
        if download_icon(icon_url, &colored_path).is_ok() {
            count += 1;
        }
        let gray_path = achievement_icon_path(appid, api_name, true);
        let _ = download_icon(icon_gray_url, &gray_path);
    }
    count
}
