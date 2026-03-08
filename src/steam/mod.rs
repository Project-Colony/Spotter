pub mod auth;

use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::models::*;

// ───── Deserialization structs ─────

#[derive(Debug, Deserialize)]
struct SteamOwnedGamesResponse {
    response: SteamOwnedGamesInner,
}

#[derive(Debug, Deserialize)]
struct SteamOwnedGamesInner {
    #[serde(default)]
    games: Vec<SteamGameInfo>,
}

#[derive(Debug, Deserialize)]
struct SteamGameInfo {
    appid: u32,
    #[serde(default)]
    name: String,
    #[serde(default)]
    playtime_forever: u32,
}

/// Result of an achievement scrape (global page + player page).
pub struct AchievementResult {
    pub unlocked: u32,
    pub total: u32,
    pub details: Vec<Achievement>,
}

/// Data scraped from a Steam store page (replaces the Store API).
#[derive(Debug, Default)]
struct ScrapedData {
    genre: String,
    description: String,
    release_date: String,
    tags: String,
    review_percent: Option<u32>,
    cover_url: Option<String>,
}

/// Number of concurrent scraping threads.
const SCRAPE_THREADS: usize = 5;

// ───── Shared helpers ─────

fn make_agent() -> ureq::Agent {
    crate::api_client::api_agent()
}

/// HTTP GET with one retry — used for the Steam Web API (owned games).
fn http_get_with_retry(agent: &ureq::Agent, url: &str) -> Result<String, String> {
    crate::api_client::http_get_retry(agent, url, 1)
}

// ───── HTML scraping helpers ─────

/// Decode common HTML entities in scraped text (single pass, no intermediate Strings).
/// Used only in tests — scraper handles entity decoding automatically in production.
#[allow(dead_code)]
fn decode_html_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut result = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        result.push_str(&rest[..amp]);
        let after = &rest[amp..];
        if let Some(end) = after.find(';') {
            let entity = &after[..end + 1];
            match entity {
                "&amp;" => result.push('&'),
                "&lt;" => result.push('<'),
                "&gt;" => result.push('>'),
                "&quot;" => result.push('"'),
                "&#39;" | "&apos;" => result.push('\''),
                _ => result.push_str(entity),
            }
            rest = &after[end + 1..];
        } else {
            result.push('&');
            rest = &after[1..];
        }
    }
    result.push_str(rest);
    result
}

/// Extract text content from all elements matching `css_selector`.
/// e.g. `r#"a[href*="/genre/"]"#` finds genre links, `"a.app_tag"` finds user tags.
/// The scraper crate provides proper HTML5 parsing and CSS selector support,
/// making this resilient to attribute ordering and nested elements.
fn extract_anchors(html: &str, css_selector: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let Ok(selector) = Selector::parse(css_selector) else {
        return Vec::new();
    };
    document
        .select(&selector)
        .map(|el| el.text().collect::<String>())
        .map(|t| t.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Extract text from the first element matching `.class_name`.
fn extract_div_text(html: &str, class_name: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector_str = format!(".{}", class_name);
    let Ok(selector) = Selector::parse(&selector_str) else {
        return None;
    };
    let text = document
        .select(&selector)
        .next()?
        .text()
        .collect::<String>();
    let text = text.trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Extract the release date from the `.release_date .date` element.
fn extract_release_date_html(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let Ok(selector) = Selector::parse(".release_date .date") else {
        return None;
    };
    let text = document
        .select(&selector)
        .next()?
        .text()
        .collect::<String>();
    let text = text.trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Extract the `content` attribute from a `<meta property="...">` tag.
/// Works regardless of attribute ordering (content before or after property).
/// e.g. `extract_meta_content(html, "og:image")` finds the og:image URL.
fn extract_meta_content(html: &str, property: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector_str = format!(r#"meta[property="{}"]"#, property);
    let Ok(selector) = Selector::parse(&selector_str) else {
        return None;
    };
    let value = document.select(&selector).next()?.attr("content")?;
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// Extract review percentage from text like "98% of the 376,982 user reviews".
fn extract_review_percent_html(html: &str) -> Option<u32> {
    let marker = "% of the ";
    let idx = html.find(marker)?;
    // Walk backwards to find the start of the digit sequence
    let before = html.get(..idx)?;
    let num_start = match before.rfind(|c: char| !c.is_ascii_digit()) {
        Some(pos) => pos + 1,
        // All characters before the marker are digits (e.g. "100% of the ...")
        None => 0,
    };
    let num_str = before.get(num_start..)?;
    if num_str.is_empty() {
        return None;
    }
    num_str.parse().ok()
}

// ───── Data fetching ─────

pub fn fetch_owned_games(api_key: &str, steam_id: &str) -> Result<Vec<Game>, String> {
    if api_key.is_empty() || steam_id.is_empty() {
        return Err("Steam API key and Steam ID are required".into());
    }

    eprintln!("[steam] Fetching owned games for Steam ID {}...", steam_id);

    let agent = make_agent();
    let url = format!(
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&format=json&include_appinfo=1",
        api_key, steam_id
    );

    let body = http_get_with_retry(&agent, &url)?;
    let resp: SteamOwnedGamesResponse =
        serde_json::from_str(&body).map_err(|e| format!("Parse error: {}", e))?;

    eprintln!("[steam] Got {} games from API", resp.response.games.len());

    let games: Vec<Game> = resp
        .response
        .games
        .into_iter()
        .filter(|g| !g.name.is_empty())
        .map(|g| {
            let cover = cover_url(g.appid);
            Game {
                id: None,
                title: g.name,
                platform: Platform::Steam,
                playtime_minutes: g.playtime_forever,
                achievements_unlocked: 0,
                achievements_total: 0,
                status: if g.playtime_forever > 0 {
                    GameStatus::Playing
                } else {
                    GameStatus::Unplayed
                },
                rating: None,
                genre: String::new(),
                last_played: String::new(),
                cover_url: cover,
                steam_appid: Some(g.appid),
                gog_id: None,
                epic_id: None,
                xbox_id: None,
                psn_id: None,
                notes: String::new(),
                description: String::new(),
                release_date: String::new(),
                review_percent: None,
                tags: String::new(),
            }
        })
        .collect();

    Ok(games)
}

/// Data parsed from a single achievement row on the player profile page.
struct PlayerAchData {
    description: String,
    unlocked: bool,
    unlock_time: u64,
    icon_url: String,
}

/// Parse the global achievements page to get all achievement names + colored icons.
/// Returns Vec<(display_name, colored_icon_url)>.
fn parse_global_achievements(html: &str) -> Vec<(String, String)> {
    let document = Html::parse_document(html);
    let row_sel = Selector::parse(".achieveRow").unwrap();
    let h3_sel = Selector::parse("h3").unwrap();
    let img_sel = Selector::parse("img").unwrap();

    let mut results = Vec::with_capacity(64);
    for row in document.select(&row_sel) {
        let name = row
            .select(&h3_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let icon_url = row
            .select(&img_sel)
            .next()
            .and_then(|el| el.attr("src"))
            .unwrap_or_default()
            .to_string();
        if !name.is_empty() {
            results.push((name, icon_url));
        }
    }
    results
}

/// Parse the player-specific achievements page to get unlock status, descriptions,
/// unlock times, and displayed icons.
/// Returns HashMap<display_name, PlayerAchData>.
fn parse_player_achievements(html: &str) -> HashMap<String, PlayerAchData> {
    let document = Html::parse_document(html);
    let row_sel = Selector::parse(".achieveRow").unwrap();
    let h3_sel = Selector::parse("h3").unwrap();
    let h5_sel = Selector::parse("h5").unwrap();
    let img_sel = Selector::parse("img").unwrap();
    let unlock_sel = Selector::parse(".achieveUnlockTime").unwrap();
    let hidden_sel = Selector::parse(".achieveHiddenBox").unwrap();

    let mut results = HashMap::with_capacity(64);
    for row in document.select(&row_sel) {
        // Skip the hidden achievements summary row
        if row.select(&hidden_sel).next().is_some() {
            continue;
        }

        let name = row
            .select(&h3_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        if name.is_empty() {
            continue;
        }

        let description = row
            .select(&h5_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let unlocked = row.select(&unlock_sel).next().is_some();

        // Extract unlock time: text inside .achieveUnlockTime like "Unlocked Nov 20, 2024 @ 6:52am"
        let unlock_time = row
            .select(&unlock_sel)
            .next()
            .map(|el| {
                let text = el.text().collect::<String>();
                let raw = text.trim();
                let raw = raw.strip_prefix("Unlocked ").unwrap_or(raw).trim();
                parse_steam_unlock_date(raw).unwrap_or(0)
            })
            .unwrap_or(0);

        let icon_url = row
            .select(&img_sel)
            .next()
            .and_then(|el| el.attr("src"))
            .unwrap_or_default()
            .to_string();

        results.insert(
            name,
            PlayerAchData {
                description,
                unlocked,
                unlock_time,
                icon_url,
            },
        );
    }
    results
}

/// Parse Steam's unlock date format "Nov 20, 2024 @ 6:52am" into a Unix timestamp.
fn parse_steam_unlock_date(date_str: &str) -> Option<u64> {
    // Normalize: "Nov 20, 2024 @ 6:52am" → "Nov 20 2024 6:52am"
    let normalized = date_str.replace(" @ ", " ").replace(',', "");
    // Try formats: "Nov 20 2024 6:52am" or "Nov 20 2024 6:52pm"
    let formats = [
        "%b %d %Y %I:%M%P", // "Nov 20 2024 6:52am"
        "%b %d %Y %I:%M%p", // "Nov 20 2024 6:52AM"
        "%b %d %Y %H:%M",   // "Nov 20 2024 18:52"
    ];
    for fmt in &formats {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&normalized, fmt) {
            return Some(dt.and_utc().timestamp() as u64);
        }
    }
    None
}

/// Scrape achievements from Steam community pages (no API key needed).
/// Combines the global page (all achievements + colored icons) with the player
/// profile page (descriptions + unlock status + unlock dates).
pub fn scrape_achievements(
    agent: &ureq::Agent,
    steam_id: &str,
    appid: u32,
) -> Result<AchievementResult, String> {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
              (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

    // 1. Scrape global page — all achievements with colored icons
    let global_url = format!("https://steamcommunity.com/stats/{}/achievements", appid);
    let global_html = match agent.get(&global_url).header("User-Agent", ua).call() {
        Ok(r) => r
            .into_body()
            .read_to_string()
            .map_err(|e| format!("{}", e))?,
        Err(ureq::Error::StatusCode(code)) if (400..500).contains(&(code as u32)) => {
            return Ok(AchievementResult {
                unlocked: 0,
                total: 0,
                details: Vec::new(),
            });
        }
        Err(e) => return Err(format!("Global page: {}", e)),
    };

    let global_achievements = parse_global_achievements(&global_html);
    if global_achievements.is_empty() {
        // Either the game truly has no achievements, or the HTML structure changed.
        if global_html.contains("achieveRow") {
            eprintln!(
                "[steam] WARN: appid {}: achieveRow markers found but parsing returned 0 achievements \
                 (HTML structure may have changed)",
                appid
            );
        }
        return Ok(AchievementResult {
            unlocked: 0,
            total: 0,
            details: Vec::new(),
        });
    }

    // 2. Scrape player page — unlock status, descriptions, unlock times
    let player_url = format!(
        "https://steamcommunity.com/profiles/{}/stats/appid/{}/achievements/",
        steam_id, appid
    );
    let player_data = match agent.get(&player_url).header("User-Agent", ua).call() {
        Ok(r) => {
            let html = r
                .into_body()
                .read_to_string()
                .map_err(|e| format!("{}", e))?;
            if html.contains("profile_fatalerror") || html.contains("This profile is private") {
                HashMap::new()
            } else {
                parse_player_achievements(&html)
            }
        }
        Err(_) => HashMap::new(), // Profile unavailable — treat all as locked
    };

    // 3. Combine: global page provides the full list + icons,
    //    player page enriches with descriptions + unlock data
    let total = global_achievements.len() as u32;
    let mut unlocked_count = 0u32;
    let mut details = Vec::with_capacity(global_achievements.len());

    for (name, colored_icon) in &global_achievements {
        let (description, unlocked, unlock_time, gray_icon) =
            if let Some(pd) = player_data.get(name.as_str()) {
                (
                    pd.description.clone(),
                    pd.unlocked,
                    pd.unlock_time,
                    if !pd.unlocked {
                        pd.icon_url.clone()
                    } else {
                        String::new()
                    },
                )
            } else {
                // Not found on player page → hidden locked achievement
                ("Hidden achievement".into(), false, 0u64, String::new())
            };

        if unlocked {
            unlocked_count += 1;
        }

        details.push(Achievement {
            api_name: name.clone(),
            display_name: name.clone(),
            description,
            icon_url: colored_icon.clone(),
            icon_gray_url: gray_icon,
            unlocked,
            unlock_time,
        });
    }

    Ok(AchievementResult {
        unlocked: unlocked_count,
        total,
        details,
    })
}

/// Scrape a Steam store page to extract genre, tags, description,
/// release date, and review percentage — all in a single HTTP request.
/// Sends cookies to bypass age gates for mature-rated games.
fn scrape_store_page(agent: &ureq::Agent, appid: u32) -> Result<ScrapedData, String> {
    let url = format!("https://store.steampowered.com/app/{}/", appid);

    let resp = agent
        .get(&url)
        .header(
            "Cookie",
            "birthtime=0; wants_mature_content=1; lastagecheckage=1-0-2000; mature_content=1",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .call()
        .map_err(|e| format!("{}", e))?;

    let html = resp
        .into_body()
        .read_to_string()
        .map_err(|e| format!("{}", e))?;

    if html.contains("agegate_box") {
        return Err("Blocked by age gate".into());
    }

    // Cover image — og:image meta tag (most reliable, always present)
    let cover_url = extract_meta_content(&html, "og:image");
    if cover_url.is_none() {
        eprintln!(
            "[steam] WARN: appid {}: og:image meta tag not found (Steam HTML may have changed)",
            appid
        );
    }

    // Genres — from links like <a href="...genre/Action/...">Action</a>
    let genre_list = extract_anchors(&html, r#"a[href*="/genre/"]"#);
    if genre_list.is_empty() && !html.contains("game_area_genre") {
        eprintln!(
            "[steam] WARN: appid {}: no genre links found (HTML structure may have changed)",
            appid
        );
    }
    let genre = genre_list.join(", ");

    // User-defined tags — from <a class="app_tag" ...>RPG</a>
    let tag_list = extract_anchors(&html, "a.app_tag");
    if tag_list.is_empty() && !html.contains("app_tag") {
        eprintln!(
            "[steam] WARN: appid {}: no app_tag elements found (HTML structure may have changed)",
            appid
        );
    }
    let tags = tag_list.join(", ");

    // Short description — try HTML div first, fall back to og:description meta tag
    let description = extract_div_text(&html, "game_description_snippet")
        .or_else(|| extract_meta_content(&html, "og:description"))
        .unwrap_or_default();
    if description.is_empty() {
        eprintln!(
            "[steam] WARN: appid {}: no description found in HTML",
            appid
        );
    }

    // Release date
    let release_date = extract_release_date_html(&html).unwrap_or_default();

    // Review percentage
    let review_percent = extract_review_percent_html(&html);

    Ok(ScrapedData {
        genre,
        description,
        release_date,
        tags,
        review_percent,
        cover_url,
    })
}

pub fn cover_url(appid: u32) -> String {
    format!(
        "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/{}/header.jpg",
        appid
    )
}

// ───── Tests ─────

#[cfg(test)]
mod tests {
    use super::*;

    // ── HTML entity decoding ──

    #[test]
    fn decode_entities_basic() {
        assert_eq!(decode_html_entities("&amp; &lt; &gt;"), "& < >");
        assert_eq!(decode_html_entities("&quot;hello&quot;"), "\"hello\"");
        assert_eq!(decode_html_entities("it&#39;s"), "it's");
        assert_eq!(decode_html_entities("plain text"), "plain text");
    }

    // ── extract_anchors ──

    #[test]
    fn extract_genre_anchors() {
        let html = r#"
            <a href="/genre/Action/">Action</a>
            <a href="/genre/RPG/">RPG</a>
            <a href="/other/">Other</a>
        "#;
        let genres = extract_anchors(html, r#"a[href*="/genre/"]"#);
        assert_eq!(genres, vec!["Action", "RPG"]);
    }

    #[test]
    fn extract_anchors_empty_html() {
        let genres = extract_anchors("", r#"a[href*="/genre/"]"#);
        assert!(genres.is_empty());
    }

    #[test]
    fn extract_anchors_no_match() {
        let html = r#"<a href="/other/">Other</a>"#;
        let genres = extract_anchors(html, r#"a[href*="/genre/"]"#);
        assert!(genres.is_empty());
    }

    #[test]
    fn extract_anchors_with_nested_tags() {
        // scraper collects text recursively — nested elements are handled correctly.
        let html = r#"<a href="/genre/Action/"><span>Action</span></a>"#;
        let genres = extract_anchors(html, r#"a[href*="/genre/"]"#);
        assert_eq!(genres, vec!["Action"]);
    }

    #[test]
    fn extract_app_tags() {
        let html = r#"
            <a class="app_tag" style="...">Open World</a>
            <a class="app_tag" style="...">Survival</a>
        "#;
        let tags = extract_anchors(html, "a.app_tag");
        assert_eq!(tags, vec!["Open World", "Survival"]);
    }

    // ── extract_div_text ──

    #[test]
    fn extract_description_snippet() {
        let html = r#"<div class="game_description_snippet">A great game about exploration.</div>"#;
        let desc = extract_div_text(html, "game_description_snippet");
        assert_eq!(desc, Some("A great game about exploration.".to_string()));
    }

    #[test]
    fn extract_div_text_missing_class() {
        let html = r#"<div class="other_class">text</div>"#;
        assert_eq!(extract_div_text(html, "game_description_snippet"), None);
    }

    #[test]
    fn extract_div_text_empty_content() {
        let html = r#"<div class="game_description_snippet">   </div>"#;
        assert_eq!(extract_div_text(html, "game_description_snippet"), None);
    }

    // ── extract_meta_content ──

    #[test]
    fn extract_og_image() {
        let html = r#"<meta property="og:image" content="https://cdn.example.com/image.jpg">"#;
        let img = extract_meta_content(html, "og:image");
        assert_eq!(img, Some("https://cdn.example.com/image.jpg".to_string()));
    }

    #[test]
    fn extract_og_description() {
        let html = r#"<meta content="A fun game" property="og:description">"#;
        let desc = extract_meta_content(html, "og:description");
        assert_eq!(desc, Some("A fun game".to_string()));
    }

    #[test]
    fn extract_meta_content_missing() {
        let html = r#"<meta property="og:title" content="Title">"#;
        assert_eq!(extract_meta_content(html, "og:image"), None);
    }

    // ── extract_release_date_html ──

    #[test]
    fn extract_release_date_typical() {
        let html = r#"
            <div class="release_date">
                <div class="subtitle">Release Date:</div>
                <div class="date">Feb 26, 2016</div>
            </div>
        "#;
        let date = extract_release_date_html(html);
        assert_eq!(date, Some("Feb 26, 2016".to_string()));
    }

    #[test]
    fn extract_release_date_missing() {
        let html = r#"<div class="game_details">No date here</div>"#;
        assert_eq!(extract_release_date_html(html), None);
    }

    // ── extract_review_percent_html ──

    #[test]
    fn extract_review_percent_typical() {
        // In real HTML, the percentage is preceded by non-digit characters
        let html = r#"<span>98% of the 376,982 user reviews are positive</span>"#;
        assert_eq!(extract_review_percent_html(html), Some(98));
    }

    #[test]
    fn extract_review_percent_low() {
        let html = r#"<span>5% of the 100 user reviews</span>"#;
        assert_eq!(extract_review_percent_html(html), Some(5));
    }

    #[test]
    fn extract_review_percent_missing() {
        let html = r#"No reviews available"#;
        assert_eq!(extract_review_percent_html(html), None);
    }

    #[test]
    fn extract_review_percent_at_start() {
        // Edge case: digits at the very start of the string (no preceding non-digit)
        let html = r#"100% of the 500 user reviews"#;
        assert_eq!(extract_review_percent_html(html), Some(100));
    }

    // ── parse_steam_unlock_date ──

    #[test]
    fn parse_unlock_date_am() {
        let ts = parse_steam_unlock_date("Nov 20, 2024 @ 6:52am");
        assert!(ts.is_some(), "Should parse AM time");
        // Nov 20, 2024 6:52 AM UTC = 1732085520 (approximately)
        let ts = ts.unwrap();
        assert!(ts > 1_700_000_000, "Timestamp should be recent");
        assert!(ts < 1_800_000_000, "Timestamp should be in range");
    }

    #[test]
    fn parse_unlock_date_pm() {
        let ts = parse_steam_unlock_date("Jan 1, 2025 @ 11:30pm");
        assert!(ts.is_some(), "Should parse PM time");
    }

    #[test]
    fn parse_unlock_date_invalid() {
        assert_eq!(parse_steam_unlock_date("not a date"), None);
        assert_eq!(parse_steam_unlock_date(""), None);
    }

    // ── parse_global_achievements ──

    #[test]
    fn parse_global_achievements_basic() {
        let html = r#"
            <div class="achieveRow">
                <img src="https://example.com/icon1.jpg">
                <h3>First Blood</h3>
            </div>
            <div class="achieveRow">
                <img src="https://example.com/icon2.jpg">
                <h3>Victory</h3>
            </div>
        "#;
        let results = parse_global_achievements(html);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "First Blood");
        assert_eq!(results[0].1, "https://example.com/icon1.jpg");
        assert_eq!(results[1].0, "Victory");
    }

    #[test]
    fn parse_global_achievements_empty() {
        let html = r#"<div class="noAchievements">None</div>"#;
        assert!(parse_global_achievements(html).is_empty());
    }

    // ── parse_player_achievements ──

    #[test]
    fn parse_player_achievements_mixed() {
        let html = r#"
            <div class="achieveRow">
                <img src="https://example.com/unlocked.jpg">
                <h3>First Blood</h3>
                <h5>Kill an enemy</h5>
                <div class="achieveUnlockTime">Unlocked Nov 20, 2024 @ 6:52am<br></div>
            </div>
            <div class="achieveRow">
                <img src="https://example.com/locked_gray.jpg">
                <h3>Victory</h3>
                <h5>Win the game</h5>
            </div>
        "#;
        let results = parse_player_achievements(html);
        assert_eq!(results.len(), 2);

        let fb = results.get("First Blood").unwrap();
        assert!(fb.unlocked);
        assert_eq!(fb.description, "Kill an enemy");
        assert!(fb.unlock_time > 0);

        let v = results.get("Victory").unwrap();
        assert!(!v.unlocked);
        assert_eq!(v.description, "Win the game");
        assert_eq!(v.unlock_time, 0);
    }

    #[test]
    fn parse_player_achievements_hidden_box_skipped() {
        let html = r#"
            <div class="achieveRow">
                <div class="achieveHiddenBox">Hidden</div>
                <h3>Secret</h3>
            </div>
            <div class="achieveRow">
                <h3>Visible</h3>
                <h5>A visible achievement</h5>
            </div>
        "#;
        let results = parse_player_achievements(html);
        assert_eq!(results.len(), 1);
        assert!(results.contains_key("Visible"));
        assert!(!results.contains_key("Secret"));
    }

    // ── cover_url ──

    #[test]
    fn cover_url_format() {
        let url = cover_url(730);
        assert!(url.contains("730"));
        assert!(url.starts_with("https://"));
        assert!(url.ends_with(".jpg"));
    }

    // ── extract_anchors edge cases ──

    #[test]
    fn extract_anchors_multiple_same_link() {
        // Duplicate href should produce duplicate entries (scraper faithfully reflects the DOM).
        let html = r#"
            <a href="/genre/Action/">Action</a>
            <a href="/genre/Action/">Action</a>
        "#;
        let genres = extract_anchors(html, r#"a[href*="/genre/"]"#);
        assert_eq!(genres.len(), 2);
    }

    #[test]
    fn extract_anchors_whitespace_trimmed() {
        let html = r#"<a href="/genre/Action/">  Action  </a>"#;
        let genres = extract_anchors(html, r#"a[href*="/genre/"]"#);
        assert_eq!(genres, vec!["Action"]);
    }

    #[test]
    fn extract_meta_content_property_after_content() {
        // Some Steam pages put content= before property= in the attribute order.
        let html = r#"<meta content="https://cdn.example.com/img.jpg" property="og:image">"#;
        let img = extract_meta_content(html, "og:image");
        assert_eq!(img, Some("https://cdn.example.com/img.jpg".to_string()));
    }

    #[test]
    fn extract_meta_content_empty_value_returns_none() {
        let html = r#"<meta property="og:image" content="">"#;
        assert_eq!(extract_meta_content(html, "og:image"), None);
    }

    #[test]
    fn extract_release_date_coming_soon_returns_none() {
        // Pages for unreleased games may have TBA or no date element at all.
        let html = r#"<div class="release_date"><div class="subtitle">Release Date:</div></div>"#;
        assert_eq!(extract_release_date_html(html), None);
    }

    #[test]
    fn extract_div_text_multiple_classes() {
        // The scraper `.class_name` selector matches even when the element has additional classes.
        let html = r#"<div class="game_description_snippet highlighted">A cool game.</div>"#;
        let desc = extract_div_text(html, "game_description_snippet");
        assert_eq!(desc, Some("A cool game.".to_string()));
    }

    // ── Full store page scrape simulation ──

    #[test]
    fn scrape_store_page_mock() {
        // Simulate a simplified Steam store page HTML
        let html = r#"
            <html>
            <head>
                <meta property="og:image" content="https://cdn.store.steampowered.com/header.jpg">
                <meta property="og:description" content="An amazing game">
            </head>
            <body>
                <a href="/genre/Action/">Action</a>
                <a href="/genre/Adventure/">Adventure</a>
                <a class="app_tag" style="">RPG</a>
                <a class="app_tag" style="">Open World</a>
                <div class="game_description_snippet">Explore a vast world.</div>
                <div class="release_date">
                    <div class="date">Mar 15, 2024</div>
                </div>
                <span>95% of the 10,000 user reviews</span>
            </body>
            </html>
        "#;

        // Test individual extractors against mock HTML
        let cover = extract_meta_content(html, "og:image");
        assert_eq!(
            cover,
            Some("https://cdn.store.steampowered.com/header.jpg".to_string())
        );

        let genres = extract_anchors(html, r#"a[href*="/genre/"]"#);
        assert_eq!(genres, vec!["Action", "Adventure"]);

        let tags = extract_anchors(html, "a.app_tag");
        assert_eq!(tags, vec!["RPG", "Open World"]);

        let desc = extract_div_text(html, "game_description_snippet");
        assert_eq!(desc, Some("Explore a vast world.".to_string()));

        let date = extract_release_date_html(html);
        assert_eq!(date, Some("Mar 15, 2024".to_string()));

        let review = extract_review_percent_html(html);
        assert_eq!(review, Some(95));
    }
}

/// Import all Steam games with full details using concurrent HTML scraping.
///
/// `existing` maps steam_appid → (has_store_data, achievements_100_percent).
/// Games that already have store data are skipped; games with 100% achievements
/// skip the achievement re-fetch.  This makes re-imports near-instant when the
/// library is already enriched.
pub fn full_import(
    api_key: &str,
    steam_id: &str,
    existing: HashMap<u32, (bool, bool)>,
) -> Result<Vec<Game>, String> {
    let logger = crate::api_client::ImportLogger::new(&crate::api_client::log_dir());
    logger.log("steam", "Starting Steam full import");

    eprintln!("[steam] Starting full import...");
    let games = fetch_owned_games(api_key, steam_id)?;
    let total = games.len();

    // Count how many we can skip
    let mut skip_scrape = 0usize;
    let mut skip_ach = 0usize;
    for g in &games {
        if let Some(appid) = g.steam_appid {
            let (has_store, ach_done) = existing.get(&appid).copied().unwrap_or((false, false));
            if has_store {
                skip_scrape += 1;
            }
            if ach_done {
                skip_ach += 1;
            }
        }
    }

    let need_scrape = total - skip_scrape;
    let need_ach = total - skip_ach;
    eprintln!(
        "[steam] {} games: {} need scraping ({} cached), {} need achievement update ({} complete)",
        total, need_scrape, skip_scrape, need_ach, skip_ach
    );

    let existing = Arc::new(existing);
    // Pre-extract appids to avoid locking games just for the ID
    let appids: Arc<Vec<u32>> =
        Arc::new(games.iter().map(|g| g.steam_appid.unwrap_or(0)).collect());
    let games = Arc::new(Mutex::new(games));
    let progress = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(Mutex::new(Vec::<String>::new()));
    // AtomicUsize counter replaces Mutex<Vec<usize>> queue — no lock contention
    let next_idx = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();

    for _ in 0..SCRAPE_THREADS {
        let games = Arc::clone(&games);
        let progress = Arc::clone(&progress);
        let failed = Arc::clone(&failed);
        let next_idx = Arc::clone(&next_idx);
        let existing = Arc::clone(&existing);
        let appids = Arc::clone(&appids);
        let steam_id = steam_id.to_string();

        let handle = std::thread::spawn(move || {
            let agent = make_agent();

            loop {
                let i = next_idx.fetch_add(1, Ordering::SeqCst);
                if i >= total {
                    break;
                }

                let appid = appids[i];
                if appid == 0 {
                    continue;
                }

                let &(has_store, ach_done) = existing.get(&appid).unwrap_or(&(false, false));

                let mut warnings: Vec<&str> = Vec::new();
                let mut skipped_all = true;

                // ── Store page scrape (skip if already enriched) ──
                if has_store {
                    // Already have genre + description + release_date → skip
                } else {
                    skipped_all = false;
                    let mut scrape_ok = false;
                    for attempt in 0..3u32 {
                        if attempt > 0 {
                            let wait = (attempt as u64 + 1) * 2;
                            std::thread::sleep(std::time::Duration::from_secs(wait));
                        }
                        match scrape_store_page(&agent, appid) {
                            Ok(data) => {
                                let mut g = match games.lock() {
                                    Ok(g) => g,
                                    Err(e) => {
                                        eprintln!("[steam] lock poisoned: {}", e);
                                        break;
                                    }
                                };
                                if let Some(url) = data.cover_url {
                                    g[i].cover_url = url;
                                }
                                if !data.genre.is_empty() {
                                    g[i].genre = data.genre;
                                }
                                if !data.description.is_empty() {
                                    g[i].description = data.description;
                                }
                                if !data.release_date.is_empty() {
                                    g[i].release_date = data.release_date;
                                }
                                if !data.tags.is_empty() {
                                    g[i].tags = data.tags;
                                }
                                g[i].review_percent = data.review_percent;
                                scrape_ok = true;
                                break;
                            }
                            Err(e) => {
                                eprintln!(
                                    "[steam]   appid {}: scrape attempt {} failed: {}",
                                    appid,
                                    attempt + 1,
                                    e
                                );
                            }
                        }
                    }
                    if !scrape_ok {
                        warnings.push("store page");
                    }
                }

                // ── Achievements (skip if 100% complete) ──
                if ach_done {
                    // Already unlocked all achievements → skip
                } else {
                    skipped_all = false;
                    let mut ach_ok = false;
                    for attempt in 0..3u32 {
                        if attempt > 0 {
                            let wait = 1u64 << attempt;
                            std::thread::sleep(std::time::Duration::from_secs(wait));
                        }
                        match scrape_achievements(&agent, &steam_id, appid) {
                            Ok(result) => {
                                if let Ok(mut g) = games.lock() {
                                    g[i].achievements_unlocked = result.unlocked;
                                    g[i].achievements_total = result.total;
                                }
                                if !result.details.is_empty() {
                                    if let Ok(conn) = crate::db::open() {
                                        if let Err(e) = crate::db::save_achievements(
                                            &conn,
                                            appid,
                                            &result.details,
                                        ) {
                                            eprintln!(
                                                "[steam]   appid {}: DB save achievements failed: {}",
                                                appid, e
                                            );
                                        }
                                    }
                                    // Pre-cache achievement icons during import
                                    let icon_count = crate::images::download_achievement_icons(
                                        appid,
                                        &result.details,
                                    );
                                    if icon_count > 0 {
                                        eprintln!(
                                            "[steam]   appid {}: cached {} achievement icons",
                                            appid, icon_count
                                        );
                                    }
                                }
                                ach_ok = true;
                                break;
                            }
                            Err(e) => {
                                eprintln!(
                                    "[steam]   appid {}: achievements attempt {} failed: {}",
                                    appid,
                                    attempt + 1,
                                    e
                                );
                            }
                        }
                    }
                    if !ach_ok {
                        warnings.push("achievements");
                    }
                }

                // ── Progress ──
                let done = progress.fetch_add(1, Ordering::SeqCst) + 1;

                if skipped_all {
                    // Don't log every skipped game to avoid spam
                } else {
                    // Read title from locked games only for logging (avoids cloning
                    // all titles upfront — saves ~150KB+ for large libraries)
                    let title = games.lock().map(|g| g[i].title.clone()).unwrap_or_default();
                    if warnings.is_empty() {
                        eprintln!("[steam] [{}/{}] {}", done, total, title);
                    } else {
                        let msg = format!("'{}': missing {}", title, warnings.join(", "));
                        eprintln!("[steam] [{}/{}] WARN: {}", done, total, msg);
                        if let Ok(mut f) = failed.lock() {
                            f.push(msg);
                        }
                    }
                }

                // Only pause between actual HTTP requests
                if !skipped_all {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.join().map_err(|_| "Worker thread panicked".to_string())?;
    }

    let games = Arc::try_unwrap(games)
        .map_err(|_| "Failed to unwrap Arc<games>")?
        .into_inner()
        .unwrap_or_else(|e| e.into_inner());
    let failed = Arc::try_unwrap(failed)
        .map_err(|_| "Failed to unwrap Arc<failed>")?
        .into_inner()
        .unwrap_or_else(|e| e.into_inner());

    eprintln!(
        "[steam] ===== Import complete: {} games processed =====",
        total
    );
    if failed.is_empty() {
        eprintln!("[steam] All games enriched successfully!");
        logger.log(
            "steam",
            &format!("Import complete: {} games, all enriched", total),
        );
    } else {
        eprintln!("[steam] {} game(s) with missing data:", failed.len());
        for msg in &failed {
            eprintln!("[steam]   - {}", msg);
            logger.log("steam", &format!("WARN: {}", msg));
        }
        logger.log(
            "steam",
            &format!(
                "Import complete: {} games, {} with missing data",
                total,
                failed.len()
            ),
        );
    }

    Ok(games)
}
