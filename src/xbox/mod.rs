use serde::Deserialize;

use crate::api_client;
use crate::models::*;

/// OpenXBL API response for title history (games played).
#[derive(Debug, Deserialize)]
struct XblTitleHistory {
    #[serde(default)]
    titles: Vec<XblTitle>,
    /// Continuation token for pagination — present when more results are available.
    #[serde(alias = "continuationToken")]
    continuation_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct XblTitle {
    #[serde(alias = "titleId", default)]
    title_id: String,
    #[serde(default)]
    name: String,
    #[serde(alias = "displayImage", default)]
    display_image: String,
    #[serde(alias = "minutesPlayed", default)]
    minutes_played: Option<u32>,
    #[serde(alias = "lastTimePlayed", default)]
    last_time_played: String,
    #[serde(alias = "currentAchievements", default)]
    current_achievements: Option<u32>,
    #[serde(alias = "totalAchievements", default)]
    total_achievements: Option<u32>,
    #[serde(default)]
    devices: Vec<String>,
}

/// OpenXBL account response (to get XUID).
#[derive(Debug, Deserialize)]
struct XblAccountResponse {
    #[serde(alias = "profileUsers", default)]
    profile_users: Vec<XblProfileUser>,
}

#[derive(Debug, Deserialize)]
struct XblProfileUser {
    #[serde(default)]
    id: String,
}

/// OpenXBL achievement response for a specific title.
#[derive(Debug, Deserialize)]
struct XblAchievementResponse {
    #[serde(default)]
    achievements: Vec<XblAchievement>,
}

#[derive(Debug, Deserialize)]
struct XblAchievement {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(alias = "progressState", default)]
    progress_state: String,
    #[serde(alias = "mediaAssets", default)]
    media_assets: Vec<XblMediaAsset>,
    #[serde(default)]
    progression: Option<XblProgression>,
}

#[derive(Debug, Deserialize)]
struct XblMediaAsset {
    #[serde(default)]
    name: String,
    #[serde(default)]
    url: String,
}

#[derive(Debug, Deserialize)]
struct XblProgression {
    #[serde(alias = "timeUnlocked", default)]
    time_unlocked: String,
}

/// OpenXBL marketplace details response.
#[derive(Debug, Deserialize)]
struct XblMarketplaceResponse {
    #[serde(alias = "Products", default)]
    products: Vec<XblProduct>,
}

#[derive(Debug, Deserialize)]
struct XblProduct {
    #[serde(alias = "ProductId", default)]
    product_id: String,
    #[serde(alias = "LocalizedProperties", default)]
    localized_properties: Vec<XblLocalizedProperties>,
    #[serde(alias = "MarketProperties", default)]
    market_properties: Vec<XblMarketProperties>,
}

#[derive(Debug, Deserialize)]
struct XblLocalizedProperties {
    #[serde(alias = "ProductDescription", default)]
    product_description: String,
    #[serde(alias = "ShortDescription", default)]
    short_description: String,
}

#[derive(Debug, Deserialize)]
struct XblMarketProperties {
    #[serde(alias = "OriginalReleaseDate", default)]
    original_release_date: String,
}

/// Game metadata from the marketplace.
struct GameMetadata {
    description: String,
    release_date: String,
}

/// Filter titles to likely be games (not apps).
fn is_game(title: &XblTitle) -> bool {
    if title.name.is_empty() {
        return false;
    }
    if title.total_achievements.unwrap_or(0) > 0 {
        return true;
    }
    let game_devices = ["XboxOne", "XboxSeries", "Xbox360", "PC", "Win32"];
    title
        .devices
        .iter()
        .any(|d| game_devices.iter().any(|gd| d.contains(gd)))
}

/// Maximum number of pages to fetch (safety cap against infinite loops).
const MAX_PAGES: usize = 50;

/// Fetch the authenticated user's XUID from OpenXBL.
fn fetch_xuid(api_key: &str) -> Result<String, String> {
    let body = api_client::http_get_headers(
        "https://xbl.io/api/v2/account",
        &[("X-Authorization", api_key), ("Accept", "application/json")],
        2,
    )?;

    let resp: XblAccountResponse =
        serde_json::from_str(&body).map_err(|e| format!("Account parse error: {}", e))?;

    resp.profile_users
        .first()
        .map(|u| u.id.clone())
        .filter(|id| !id.is_empty())
        .ok_or_else(|| "Could not determine XUID from account response".into())
}

/// Fetch per-game achievements from the modern OpenXBL endpoint.
fn fetch_achievements(
    api_key: &str,
    xuid: &str,
    title_id: &str,
) -> Result<Vec<Achievement>, String> {
    let url = format!(
        "https://xbl.io/api/v2/achievements/player/{}/{}",
        xuid, title_id
    );

    let body = api_client::http_get_headers(
        &url,
        &[("X-Authorization", api_key), ("Accept", "application/json")],
        2,
    )?;

    let resp: XblAchievementResponse =
        serde_json::from_str(&body).map_err(|e| format!("Achievement parse error: {}", e))?;

    let achievements: Vec<Achievement> = resp
        .achievements
        .into_iter()
        .map(|a| {
            let unlocked = a.progress_state == "Achieved";
            let unlock_time = a
                .progression
                .as_ref()
                .and_then(|p| parse_iso_datetime(&p.time_unlocked))
                .unwrap_or(0);

            // Icon: prefer the "Icon" media asset
            let icon_url = a
                .media_assets
                .iter()
                .find(|m| m.name == "Icon")
                .or_else(|| a.media_assets.first())
                .map(|m| m.url.clone())
                .unwrap_or_default();

            Achievement {
                api_name: a.id,
                display_name: a.name,
                description: a.description,
                icon_url,
                icon_gray_url: String::new(),
                unlocked,
                unlock_time,
            }
        })
        .collect();

    Ok(achievements)
}

/// Fetch achievements from the Xbox 360 endpoint.
/// Uses flexible JSON parsing because the x360 response format differs from modern:
/// - `id` is a number (not string)
/// - `unlockedOnline` instead of `progressState`
/// - `lockedDescription`/`unlockedDescription` instead of `description`
fn fetch_achievements_x360(
    api_key: &str,
    xuid: &str,
    title_id: &str,
) -> Result<Vec<Achievement>, String> {
    let url = format!(
        "https://xbl.io/api/v2/achievements/x360/{}/title/{}",
        xuid, title_id
    );

    let body = api_client::http_get_headers(
        &url,
        &[("X-Authorization", api_key), ("Accept", "application/json")],
        2,
    )?;

    // Parse as generic JSON — the x360 format uses different field types/names
    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("x360 parse error: {}", e))?;

    let achievements = json
        .get("achievements")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| {
                    // id can be number or string
                    let id = match a.get("id") {
                        Some(serde_json::Value::Number(n)) => n.to_string(),
                        Some(serde_json::Value::String(s)) => s.clone(),
                        _ => return None,
                    };
                    let name = a
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if name.is_empty() {
                        return None;
                    }

                    let unlocked = a
                        .get("unlockedOnline")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    let description = if unlocked {
                        a.get("unlockedDescription")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                    } else {
                        a.get("lockedDescription")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                    };

                    let unlock_time = a
                        .get("timeUnlocked")
                        .and_then(|v| v.as_str())
                        .and_then(parse_iso_datetime)
                        .unwrap_or(0);

                    Some(Achievement {
                        api_name: id,
                        display_name: name,
                        description: description.to_string(),
                        icon_url: String::new(),
                        icon_gray_url: String::new(),
                        unlocked,
                        unlock_time,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(achievements)
}

/// Parse ISO 8601 datetime (e.g. "2024-11-20T06:52:00Z") into Unix timestamp.
fn parse_iso_datetime(s: &str) -> Option<u64> {
    if s.is_empty() || s == "0001-01-01T00:00:00.0000000Z" {
        return None;
    }
    // Try common ISO formats
    let formats = [
        "%Y-%m-%dT%H:%M:%S%.fZ",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%dT%H:%M:%S",
    ];
    for fmt in &formats {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt.and_utc().timestamp() as u64);
        }
    }
    None
}

/// Fetch metadata from the OpenXBL marketplace for a batch of title IDs.
fn fetch_marketplace_details(
    api_key: &str,
    title_ids: &[&str],
) -> std::collections::HashMap<String, GameMetadata> {
    use std::collections::HashMap;

    let mut metadata: HashMap<String, GameMetadata> = HashMap::new();
    if title_ids.is_empty() {
        return metadata;
    }

    // The marketplace endpoint uses product IDs, not title IDs.
    // Try sending title IDs — if the API doesn't recognize them, we get empty results (harmless).
    let products_param = title_ids.join(",");
    let url = "https://xbl.io/api/v2/marketplace/details";

    let agent = api_client::api_agent();
    let resp = agent
        .post(url)
        .header("X-Authorization", api_key)
        .header("Accept", "application/json")
        .content_type("application/json")
        .send(format!(r#"{{"products":"{}"}}"#, products_param));

    let body = match resp {
        Ok(r) => match r.into_body().read_to_string() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[xbox] Marketplace read error: {}", e);
                return metadata;
            }
        },
        Err(e) => {
            eprintln!("[xbox] Marketplace API error: {}", e);
            return metadata;
        }
    };

    let market_resp: XblMarketplaceResponse = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[xbox] Marketplace parse error: {}", e);
            return metadata;
        }
    };

    for product in market_resp.products {
        let description = product
            .localized_properties
            .first()
            .map(|lp| {
                if !lp.short_description.is_empty() {
                    lp.short_description.clone()
                } else {
                    // Truncate long descriptions
                    let desc = &lp.product_description;
                    if desc.len() > 500 {
                        format!("{}...", &desc[..497])
                    } else {
                        desc.clone()
                    }
                }
            })
            .unwrap_or_default();

        let release_date = product
            .market_properties
            .first()
            .map(|mp| {
                // Format: "2024-11-20T00:00:00Z" → "Nov 20, 2024"
                mp.original_release_date.get(..10).unwrap_or("").to_string()
            })
            .unwrap_or_default();

        if !description.is_empty() || !release_date.is_empty() {
            metadata.insert(
                product.product_id.clone(),
                GameMetadata {
                    description,
                    release_date,
                },
            );
        }
    }

    if !metadata.is_empty() {
        eprintln!(
            "[xbox] Marketplace: got metadata for {} products",
            metadata.len()
        );
    }

    metadata
}

/// Full Xbox import with achievements and metadata enrichment.
/// Returns (games, optional XUID for persistence).
pub fn full_import(api_key: &str) -> Result<(Vec<Game>, Option<String>), String> {
    if api_key.is_empty() {
        return Err("Xbox API key is required. Get one at xbl.io".into());
    }

    let logger = api_client::ImportLogger::new(&api_client::log_dir());
    logger.log("xbox", "Starting Xbox import");

    // Step 1: Get XUID (needed for achievement endpoints)
    eprintln!("[xbox] Fetching account info (XUID)...");
    let xuid = match fetch_xuid(api_key) {
        Ok(id) => {
            eprintln!("[xbox] XUID: {}", id);
            logger.log("xbox", &format!("XUID: {}", id));
            id
        }
        Err(e) => {
            eprintln!(
                "[xbox] Could not fetch XUID: {}. Achievements will be count-only.",
                e
            );
            logger.log("xbox", &format!("XUID fetch failed: {}", e));
            String::new()
        }
    };

    std::thread::sleep(std::time::Duration::from_millis(300));

    // Step 2: Fetch title history (games list)
    eprintln!("[xbox] Fetching title history from OpenXBL...");
    let mut all_titles: Vec<XblTitle> = Vec::with_capacity(500);
    let mut continuation_token: Option<String> = None;

    for page in 0..MAX_PAGES {
        let url = match &continuation_token {
            Some(token) => format!(
                "https://xbl.io/api/v2/player/titleHistory?continuationToken={}",
                api_client::url_encode(token)
            ),
            None => "https://xbl.io/api/v2/player/titleHistory".to_string(),
        };

        let body = api_client::http_get_headers(
            &url,
            &[
                ("X-Authorization", api_key),
                ("Accept", "application/json"),
                ("Accept-Language", "en-US"),
            ],
            3,
        )?;

        let history: XblTitleHistory =
            serde_json::from_str(&body).map_err(|e| format!("Xbox API parse error: {}", e))?;

        let count = history.titles.len();
        eprintln!(
            "[xbox] Page {}: {} titles{}",
            page + 1,
            count,
            if history.continuation_token.is_some() {
                " (more available)"
            } else {
                ""
            }
        );

        all_titles.extend(history.titles);

        match history.continuation_token {
            Some(token) if !token.is_empty() => {
                continuation_token = Some(token);
                std::thread::sleep(std::time::Duration::from_millis(300));
            }
            _ => break,
        }
    }

    eprintln!("[xbox] Found {} total titles", all_titles.len());

    // Filter to games only
    let game_titles: Vec<XblTitle> = all_titles.into_iter().filter(is_game).collect();
    let total = game_titles.len();
    eprintln!("[xbox] {} are games", total);

    // Step 3: Try marketplace metadata (single batch request)
    let title_ids: Vec<&str> = game_titles.iter().map(|t| t.title_id.as_str()).collect();
    let marketplace = fetch_marketplace_details(api_key, &title_ids);
    if !marketplace.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    // Step 4: Build games with achievements enrichment
    let mut games = Vec::with_capacity(total);

    for (i, title) in game_titles.into_iter().enumerate() {
        let last_played = if !title.last_time_played.is_empty() {
            title.last_time_played.get(..10).unwrap_or("").to_string()
        } else {
            String::new()
        };

        // Metadata from marketplace (if available)
        let meta = marketplace.get(&title.title_id);
        let description = meta.map(|m| m.description.clone()).unwrap_or_default();
        let release_date = meta.map(|m| m.release_date.clone()).unwrap_or_default();

        // Fetch detailed achievements if we have a XUID.
        // Always try — the title history often reports 0 achievements even for
        // games that have them (especially Xbox 360 titles).
        let (ach_unlocked, ach_total) = if !xuid.is_empty() {
            // Try modern endpoint first
            let details = match fetch_achievements(api_key, &xuid, &title.title_id) {
                Ok(d) if !d.is_empty() => {
                    eprintln!("[xbox]   Modern endpoint: {} achievements", d.len());
                    Ok(d)
                }
                modern_result => {
                    // Modern endpoint returned empty or error — try x360 endpoint.
                    // Many older titles (Fable II, Fallout 3, etc.) only return data
                    // from the x360 endpoint, even when not tagged as "Xbox360" in devices.
                    eprintln!(
                        "[xbox]   Modern endpoint empty for {}, trying x360...",
                        title.name
                    );
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    match fetch_achievements_x360(api_key, &xuid, &title.title_id) {
                        Ok(d) if !d.is_empty() => {
                            eprintln!("[xbox]   x360 endpoint: {} achievements", d.len());
                            Ok(d)
                        }
                        x360_result => {
                            if let Err(ref e) = x360_result {
                                eprintln!("[xbox]   x360 endpoint error: {}", e);
                            }
                            // Use modern result (might be Ok(empty) or Err)
                            modern_result
                        }
                    }
                }
            };

            match details {
                Ok(details) if !details.is_empty() => {
                    let unlocked = details.iter().filter(|a| a.unlocked).count() as u32;
                    let total_ach = details.len() as u32;

                    // Save to DB
                    let platform_key = format!("xbox:{}", title.title_id);
                    if let Ok(conn) = crate::db::open() {
                        if let Err(e) =
                            crate::db::save_achievements_by_platform(&conn, &platform_key, &details)
                        {
                            eprintln!("[xbox]   {}: DB save failed: {}", title.name, e);
                        }
                    }

                    eprintln!(
                        "[xbox] [{}/{}] {} — {} achievements ({} unlocked)",
                        i + 1,
                        total,
                        title.name,
                        total_ach,
                        unlocked
                    );

                    (unlocked, total_ach)
                }
                Ok(_) => {
                    eprintln!(
                        "[xbox] [{}/{}] {} — no achievements",
                        i + 1,
                        total,
                        title.name
                    );
                    (0, 0)
                }
                Err(e) => {
                    eprintln!(
                        "[xbox] [{}/{}] {} — achievement fetch failed: {}",
                        i + 1,
                        total,
                        title.name,
                        e
                    );
                    // Fall back to counts from title history
                    (
                        title.current_achievements.unwrap_or(0),
                        title.total_achievements.unwrap_or(0),
                    )
                }
            }
        } else {
            eprintln!("[xbox] [{}/{}] {}", i + 1, total, title.name);
            (
                title.current_achievements.unwrap_or(0),
                title.total_achievements.unwrap_or(0),
            )
        };

        let status = if ach_unlocked == ach_total && ach_total > 0 {
            GameStatus::Completed
        } else if title.minutes_played.unwrap_or(0) > 0 || ach_unlocked > 0 {
            GameStatus::Playing
        } else {
            GameStatus::Unplayed
        };

        games.push(Game {
            id: None,
            title: title.name,
            platform: Platform::Xbox,
            playtime_minutes: title.minutes_played.unwrap_or(0),
            achievements_unlocked: ach_unlocked,
            achievements_total: ach_total,
            status,
            rating: None,
            genre: String::new(),
            last_played,
            cover_url: title.display_image,
            steam_appid: None,
            gog_id: None,
            epic_id: None,
            xbox_id: Some(title.title_id),
            psn_id: None,
            notes: String::new(),
            description,
            release_date,
            review_percent: None,
            tags: String::new(),
        });

        // Rate limit between achievement fetches
        if !xuid.is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
    }

    eprintln!("[xbox] Imported {} games", games.len());
    logger.log("xbox", &format!("Import complete: {} games", games.len()));

    let xuid_result = if xuid.is_empty() { None } else { Some(xuid) };
    Ok((games, xuid_result))
}
