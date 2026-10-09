use serde::Deserialize;

use crate::models::*;

/// PSN Android app OAuth client credentials (public, used by all PSN API libraries).
const PSN_CLIENT_ID: &str = "09515159-7ef5-4e03-b682-b2c3e0d58564";
const PSN_AUTH_BASIC: &str =
    "Basic MDk1MTUxNTktN2VmNS00ZTAzLWI2ODItYjJjM2UwZDU4NTY0OnVjUGprYTV0bnRCMktxc1A=";

/// PSN OAuth token response (extra fields needed for serde deserialization).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PsnTokenResponse {
    access_token: String,
    #[serde(default)]
    token_type: String,
    #[serde(default)]
    expires_in: u64,
    #[serde(default)]
    refresh_token: String,
}

/// PSN game list response.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PsnGameListResponse {
    #[serde(alias = "titles", default)]
    titles: Vec<PsnTitle>,
    #[serde(alias = "totalItemCount", default)]
    total_item_count: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PsnTitle {
    #[serde(alias = "titleId", default)]
    title_id: String,
    #[serde(default)]
    name: String,
    #[serde(alias = "imageUrl", default)]
    image_url: String,
    #[serde(alias = "category", default)]
    category: String,
    #[serde(alias = "playDuration", default)]
    play_duration: Option<String>,
    #[serde(alias = "lastPlayedDateTime", default)]
    last_played_date_time: String,
}

/// PSN trophy titles response.
#[derive(Debug, Deserialize)]
struct PsnTrophyTitlesResponse {
    #[serde(alias = "trophyTitles", default)]
    trophy_titles: Vec<PsnTrophyTitle>,
}

#[derive(Debug, Deserialize)]
struct PsnTrophyTitle {
    #[serde(alias = "npCommunicationId", default)]
    np_communication_id: String,
    #[serde(alias = "trophyTitleName", default)]
    trophy_title_name: String,
    #[serde(alias = "definedTrophies", default)]
    defined_trophies: PsnTrophyCounts,
    #[serde(alias = "earnedTrophies", default)]
    earned_trophies: PsnTrophyCounts,
    #[serde(alias = "trophyTitleIconUrl", default)]
    trophy_title_icon_url: String,
    #[serde(alias = "lastUpdatedDateTime", default)]
    last_updated_date_time: String,
}

#[derive(Debug, Default, Deserialize)]
struct PsnTrophyCounts {
    #[serde(default)]
    bronze: u32,
    #[serde(default)]
    silver: u32,
    #[serde(default)]
    gold: u32,
    #[serde(default)]
    platinum: u32,
}

impl PsnTrophyCounts {
    fn total(&self) -> u32 {
        self.bronze
            .saturating_add(self.silver)
            .saturating_add(self.gold)
            .saturating_add(self.platinum)
    }
}

/// Exchange NPSSO token for an access token.
fn exchange_npsso(npsso: &str) -> Result<String, String> {
    if npsso.is_empty() {
        return Err("PSN NPSSO token is required.".into());
    }

    eprintln!("[psn] Exchanging NPSSO for authorization code...");

    // Step 1: Get authorization code using NPSSO cookie
    // Use an agent with redirects disabled and status-as-error disabled
    // so we can capture the 302 Location header.
    let agent = ureq::Agent::config_builder()
        .max_redirects(0)
        .http_status_as_error(false)
        .build()
        .new_agent();

    let auth_url = "https://ca.account.sony.com/api/authz/v3/oauth/authorize";
    let params = format!(
        "access_type=offline&client_id={}&redirect_uri=com.scee.psxandroid.scecompcall://redirect&response_type=code&scope=psn:mobile.v2.core psn:clientapp",
        PSN_CLIENT_ID
    );

    let full_url = format!("{}?{}", auth_url, params);
    let resp = agent
        .get(&full_url)
        .header("Cookie", &format!("npsso={}", npsso))
        .call()
        .map_err(|e| format!("PSN auth error: {}", e))?;

    // The API returns a 302 redirect with the code in the Location header
    let status = resp.status().as_u16();
    let code = if status == 302 {
        // Extract code from Location header redirect
        let location = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if let Some(code_start) = location.find("code=") {
            let code_part = &location[code_start + 5..];
            code_part.split('&').next().unwrap_or("").to_string()
        } else {
            return Err("Could not extract auth code from redirect".into());
        }
    } else if status == 200 {
        // Sometimes it returns 200 with JSON containing the code
        let body = resp
            .into_body()
            .read_to_string()
            .map_err(|e| format!("Read error: {}", e))?;
        let json: serde_json::Value = serde_json::from_str(&body)
            .map_err(|_| "NPSSO token may be expired. Get a new one from PlayStation website.")?;
        json.get("code")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                "NPSSO token may be expired. Get a new one from PlayStation website.".to_string()
            })?
    } else {
        return Err(format!("PSN auth error: HTTP {}", status));
    };

    if code.is_empty() {
        return Err("Empty authorization code. NPSSO token may be expired.".into());
    }

    eprintln!("[psn] Got authorization code, exchanging for access token...");

    // Step 2: Exchange code for access token
    let token_url = "https://ca.account.sony.com/api/authz/v3/oauth/token";
    let token_agent = crate::api_client::api_agent();
    let token_resp = token_agent.post(token_url)
        .content_type("application/x-www-form-urlencoded")
        .header("Authorization", PSN_AUTH_BASIC)
        .send(format!(
            "code={}&grant_type=authorization_code&redirect_uri=com.scee.psxandroid.scecompcall://redirect",
            code
        ))
        .map_err(|e| format!("PSN token exchange error: {}", e))?;

    let token: PsnTokenResponse = token_resp
        .into_body()
        .read_json()
        .map_err(|e| format!("PSN token parse error: {}", e))?;

    Ok(token.access_token)
}

/// Fetch trophy titles (which serve as the PSN game list with achievement data).
fn fetch_trophy_titles(access_token: &str) -> Result<Vec<PsnTrophyTitle>, String> {
    let mut all_titles = Vec::with_capacity(200);
    let mut offset = 0;
    let limit = 100;
    const MAX_PAGES: usize = 200; // Safety cap: 200 pages × 100 = 20 000 titles max
    let agent = crate::api_client::api_agent();

    for _page in 0..MAX_PAGES {
        let url = format!(
            "https://m.np.playstation.com/api/trophy/v1/users/me/trophyTitles?limit={}&offset={}",
            limit, offset
        );

        let resp = agent
            .get(&url)
            .header("Authorization", &format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .call()
            .map_err(|e| format!("PSN trophy API error: {}", e))?;

        let response: PsnTrophyTitlesResponse = resp
            .into_body()
            .read_json()
            .map_err(|e| format!("PSN trophy parse error: {}", e))?;

        let count = response.trophy_titles.len();
        all_titles.extend(response.trophy_titles);

        if count < limit {
            break;
        }
        offset += limit;

        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    Ok(all_titles)
}

/// Fetch the user's game list (recently played).
fn fetch_game_list(access_token: &str) -> Result<Vec<PsnTitle>, String> {
    let mut all_titles = Vec::with_capacity(200);
    let mut offset = 0;
    let limit = 100;
    const MAX_PAGES: usize = 200; // Safety cap: 200 pages × 100 = 20 000 titles max
    let agent = crate::api_client::api_agent();

    for _page in 0..MAX_PAGES {
        let url = format!(
            "https://m.np.playstation.com/api/gamelist/v2/users/me/titles?limit={}&offset={}&categories=ps4_game,ps5_native_game",
            limit, offset
        );

        let resp = agent
            .get(&url)
            .header("Authorization", &format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .call();

        match resp {
            Ok(r) => {
                let response: PsnGameListResponse = r
                    .into_body()
                    .read_json()
                    .map_err(|e| format!("PSN game list parse error: {}", e))?;
                let count = response.titles.len();
                all_titles.extend(response.titles);
                if count < limit {
                    break;
                }
                offset += limit;
            }
            Err(e) => {
                eprintln!(
                    "[psn] Game list API unavailable: {}; falling back to trophy titles",
                    e
                );
                break;
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    Ok(all_titles)
}

/// Parse a PSN ISO-8601 duration like "PT123H45M6S" into minutes.
fn parse_play_duration(duration: &str) -> u32 {
    if !duration.starts_with("PT") {
        return 0;
    }
    let rest = &duration[2..];
    let mut hours = 0u32;
    let mut minutes = 0u32;
    let mut num = String::new();

    for ch in rest.chars() {
        if ch.is_ascii_digit() {
            num.push(ch);
        } else {
            let val: u32 = num.parse().unwrap_or(0);
            match ch {
                'H' => hours = val,
                'M' => minutes = val,
                _ => {}
            }
            num.clear();
        }
    }

    hours * 60 + minutes
}

pub fn full_import(npsso: &str) -> Result<Vec<Game>, String> {
    let access_token = exchange_npsso(npsso)?;

    let logger = crate::api_client::ImportLogger::new(&crate::api_client::log_dir());
    logger.log("psn", "Starting PlayStation import");

    // Fetch trophy titles (primary source — has achievement data)
    eprintln!("[psn] Fetching trophy titles...");
    let trophy_titles = fetch_trophy_titles(&access_token)?;
    eprintln!("[psn] Found {} trophy titles", trophy_titles.len());
    logger.log(
        "psn",
        &format!("Found {} trophy titles", trophy_titles.len()),
    );

    // Also try to fetch game list for additional data (playtime, cover images)
    eprintln!("[psn] Fetching game list...");
    let game_list = fetch_game_list(&access_token).unwrap_or_default();
    eprintln!("[psn] Found {} games in game list", game_list.len());

    // Build lookup from game list for playtime and cover enrichment
    let game_map: std::collections::HashMap<&str, &PsnTitle> = game_list
        .iter()
        .filter(|t| !t.title_id.is_empty())
        .map(|t| (t.title_id.as_str(), t))
        .collect();

    let mut games = Vec::with_capacity(trophy_titles.len());

    for (i, trophy_title) in trophy_titles.iter().enumerate() {
        if trophy_title.trophy_title_name.is_empty() {
            continue;
        }

        eprintln!(
            "[psn] [{}/{}] {}",
            i + 1,
            trophy_titles.len(),
            trophy_title.trophy_title_name
        );

        let total_trophies = trophy_title.defined_trophies.total();
        let earned_trophies = trophy_title.earned_trophies.total();

        // Check if we have game list data for this title
        let (playtime, cover_url, last_played) = if let Some(game_info) = trophy_title
            .np_communication_id
            .strip_prefix("NPWR")
            .and_then(|_| {
                // Try to match by name since IDs differ between APIs
                game_list.iter().find(|g| {
                    g.name.to_lowercase() == trophy_title.trophy_title_name.to_lowercase()
                })
            })
            .or_else(|| {
                game_map
                    .get(trophy_title.np_communication_id.as_str())
                    .copied()
            }) {
            let playtime = game_info
                .play_duration
                .as_deref()
                .map(parse_play_duration)
                .unwrap_or(0);
            let cover = game_info.image_url.clone();
            let last = game_info
                .last_played_date_time
                .get(..10)
                .unwrap_or("")
                .to_string();
            (playtime, cover, last)
        } else {
            let cover = trophy_title.trophy_title_icon_url.clone();
            let last = trophy_title
                .last_updated_date_time
                .get(..10)
                .unwrap_or("")
                .to_string();
            (0, cover, last)
        };

        let status = if earned_trophies == total_trophies && total_trophies > 0 {
            GameStatus::Completed
        } else if earned_trophies > 0 || playtime > 0 {
            GameStatus::Playing
        } else {
            GameStatus::Unplayed
        };

        games.push(Game {
            id: None,
            title: trophy_title.trophy_title_name.clone(),
            platform: Platform::PlayStation,
            playtime_minutes: playtime,
            achievements_unlocked: earned_trophies,
            achievements_total: total_trophies,
            status,
            rating: None,
            genre: String::new(),
            last_played,
            cover_url,
            steam_appid: None,
            gog_id: None,
            epic_id: None,
            xbox_id: None,
            psn_id: Some(trophy_title.np_communication_id.clone()),
            notes: String::new(),
            description: String::new(),
            release_date: String::new(),
            review_percent: None,
            tags: String::new(),
        });
    }

    eprintln!("[psn] Imported {} games", games.len());
    logger.log("psn", &format!("Import complete: {} games", games.len()));
    Ok(games)
}
