pub mod auth;

use crate::api_client;
use crate::models::*;
use serde::{Deserialize, Deserializer};

/// Deserialize a JSON value that may be `null`, a string, or missing → always `String`.
fn nullable_string<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Option::<String>::deserialize(d).map(|o| o.unwrap_or_default())
}

// ── Filtered products listing (embed.gog.com, auth required) ───

#[derive(Debug, Deserialize)]
struct FilteredResponse {
    #[serde(default)]
    products: Vec<FilteredProduct>,
    #[serde(default, rename = "totalPages")]
    total_pages: u32,
}

#[derive(Debug, Deserialize)]
struct FilteredProduct {
    #[serde(default)]
    id: i64,
    #[serde(default, deserialize_with = "nullable_string")]
    title: String,
    #[serde(default, deserialize_with = "nullable_string")]
    image: String,
    #[serde(default, deserialize_with = "nullable_string")]
    category: String,
}

// ── Store product details (api.gog.com, public) ───

#[derive(Debug, Deserialize)]
struct GogProduct {
    #[serde(default)]
    images: GogImages,
    #[serde(default)]
    description: GogDescription,
    #[serde(default, deserialize_with = "nullable_string")]
    release_date: String,
}

#[derive(Debug, Default, Deserialize)]
struct GogImages {
    #[serde(default, deserialize_with = "nullable_string")]
    logo2x: String,
}

#[derive(Debug, Default, Deserialize)]
struct GogDescription {
    #[serde(default, deserialize_with = "nullable_string")]
    lead: String,
}

/// Metadata fetched from the GOG store for a single game.
struct StoreMetadata {
    description: String,
    release_date: String,
    tags: String,
    cover: String,
    was_rate_limited: bool,
}

// ── Import logic ───

pub fn full_import(token: &str) -> Result<Vec<Game>, String> {
    if token.is_empty() {
        return Err("GOG token is required.".into());
    }

    let logger = api_client::ImportLogger::new(&api_client::log_dir());
    logger.log("gog", "Starting GOG import");

    // Step 1: Get all owned games from the filtered listing (games only, no DLCs).
    let listings = fetch_all_listings(token)?;
    eprintln!("[gog] Found {} games in library", listings.len());
    logger.log("gog", &format!("Found {} games in library", listings.len()));

    // Step 2: For each game, get title + image from listing,
    //         then enrich with description from public API.
    let mut games = Vec::with_capacity(listings.len());
    let total = listings.len();
    let agent = api_client::api_agent();
    let mut limiter = api_client::RateLimiter::new(std::time::Duration::from_millis(150));

    for (i, listing) in listings.iter().enumerate() {
        if listing.title.is_empty() {
            continue;
        }

        // Cover from listing image
        let cover = if !listing.image.is_empty() {
            let img = &listing.image;
            if img.starts_with("//") {
                format!("https:{}", img)
            } else {
                img.clone()
            }
        } else {
            String::new()
        };

        // Enrich with store metadata (description, release date, better cover)
        let meta = fetch_store_metadata(&agent, listing.id);

        let final_cover = if !meta.cover.is_empty() {
            meta.cover
        } else {
            cover
        };

        eprintln!("[gog] [{}/{}] {}", i + 1, total, listing.title);
        games.push(Game {
            id: None,
            title: listing.title.clone(),
            platform: Platform::Gog,
            playtime_minutes: 0,
            achievements_unlocked: 0,
            achievements_total: 0,
            status: GameStatus::Unplayed,
            rating: None,
            genre: listing.category.clone(),
            last_played: String::new(),
            cover_url: final_cover,
            steam_appid: None,
            gog_id: Some(listing.id.to_string()),
            epic_id: None,
            xbox_id: None,
            psn_id: None,
            notes: String::new(),
            description: meta.description,
            release_date: meta.release_date,
            review_percent: None,
            tags: meta.tags,
        });

        // Adaptive rate-limit between store API calls
        if meta.was_rate_limited {
            limiter.on_rate_limited();
        } else {
            limiter.on_success();
        }
        limiter.wait();
    }

    eprintln!("[gog] Imported {} games", games.len());
    logger.log("gog", &format!("Import complete: {} games", games.len()));
    Ok(games)
}

/// Fetch all pages of /account/getFilteredProducts (games only).
fn fetch_all_listings(token: &str) -> Result<Vec<FilteredProduct>, String> {
    const MAX_PAGES: u32 = 100; // Safety cap to prevent infinite loops on API errors

    let mut all = Vec::with_capacity(100);
    let mut page = 1u32;
    let agent = api_client::api_agent();

    loop {
        let url = format!(
            "https://embed.gog.com/account/getFilteredProducts?mediaType=1&page={}",
            page
        );
        let body = api_client::http_get_bearer(&agent, &url, token, 3)?;
        let resp: FilteredResponse = serde_json::from_str(&body)
            .map_err(|e| format!("Listing parse error (page {}): {}", page, e))?;

        let total_pages = resp.total_pages;
        eprintln!(
            "[gog] Page {}/{}: {} products",
            page,
            total_pages,
            resp.products.len()
        );
        all.extend(resp.products);

        if page >= total_pages || page >= MAX_PAGES {
            break;
        }
        page += 1;
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    Ok(all)
}

/// Fetch description, release date, tags and cover from the GOG public store API.
fn fetch_store_metadata(agent: &ureq::Agent, game_id: i64) -> StoreMetadata {
    let empty = StoreMetadata {
        description: String::new(),
        release_date: String::new(),
        tags: String::new(),
        cover: String::new(),
        was_rate_limited: false,
    };

    let url = format!(
        "https://api.gog.com/products/{}?expand=description",
        game_id
    );

    let resp = match agent.get(&url).call() {
        Ok(r) => r,
        Err(ureq::Error::StatusCode(429)) => {
            eprintln!(
                "[gog] 429 rate-limited fetching metadata for game {}",
                game_id
            );
            return StoreMetadata {
                was_rate_limited: true,
                ..empty
            };
        }
        Err(e) => {
            eprintln!("[gog] Failed to fetch metadata for game {}: {}", game_id, e);
            return empty;
        }
    };

    let body = match resp.into_body().read_to_string() {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "[gog] Failed to read metadata body for game {}: {}",
                game_id, e
            );
            return empty;
        }
    };

    // Deserialize structured product and raw Value from the same body
    // (avoids cloning the entire JSON tree)
    let product: GogProduct = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "[gog] Failed to deserialize product for game {}: {}",
                game_id, e
            );
            return empty;
        }
    };

    let description = clean_html(&product.description.lead);
    let release_date = product.release_date;

    // Parse again as Value only for genre extraction (lightweight since body is already in memory)
    let tags = match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(val) => extract_genres_from_value(&val),
        Err(_) => String::new(),
    };

    let cover = if !product.images.logo2x.is_empty() {
        let logo = &product.images.logo2x;
        if logo.starts_with("//") {
            format!("https:{}", logo)
        } else {
            logo.clone()
        }
    } else {
        String::new()
    };

    StoreMetadata {
        description,
        release_date,
        tags,
        cover,
        was_rate_limited: false,
    }
}

/// Minimal HTML tag stripping for GOG descriptions.
fn clean_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }
    // Trim in-place to avoid reallocating
    let trimmed = result.trim();
    if trimmed.len() == result.len() {
        result
    } else {
        trimmed.to_string()
    }
}

/// Extract genre names from the parsed GOG product JSON.
/// Navigates the structured Value instead of searching raw text,
/// so a "genres" substring inside a description cannot cause a false match.
fn extract_genres_from_value(val: &serde_json::Value) -> String {
    val.get("genres")
        .and_then(|g| g.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    item.get("name")
                        .and_then(|n| n.as_str())
                        .or_else(|| item.as_str())
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}
