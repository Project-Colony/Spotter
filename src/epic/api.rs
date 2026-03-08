use std::collections::HashMap;

use super::types::*;

const LIBRARY_HOST: &str = "https://library-service.live.use1a.on.epicgames.com";
const CATALOG_HOST: &str = "https://catalog-public-service-prod06.ol.epicgames.com";
const ENTITLEMENTS_HOST: &str = "https://entitlement-public-service-prod08.ol.epicgames.com";
const LAUNCHER_HOST: &str = "https://launcher-public-service-prod06.ol.epicgames.com";

/// Fetch all library items with pagination.
pub(super) fn fetch_library(
    agent: &ureq::Agent,
    token: &str,
) -> Result<Vec<LibraryRecord>, String> {
    let mut all_records = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let mut url = format!(
            "{}/library/api/public/items?includeMetadata=true",
            LIBRARY_HOST,
        );
        if let Some(ref c) = cursor {
            url.push_str(&format!("&cursor={}", crate::api_client::url_encode(c)));
        }

        let body = crate::api_client::http_get_bearer(agent, &url, token, 2)?;

        // Diagnostic: dump raw JSON of first record on first page so we can
        // see the actual API field names/structure (including metadata).
        if all_records.is_empty() {
            if let Ok(raw) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(first) = raw.get("records").and_then(|r| r.get(0)) {
                    eprintln!("[epic] RAW first library record: {}", first);
                }
            }
        }

        let resp: LibraryResponse =
            serde_json::from_str(&body).map_err(|e| format!("Library parse error: {}", e))?;

        all_records.extend(resp.records);

        match resp.response_metadata.and_then(|m| m.next_cursor) {
            Some(c) if !c.is_empty() && c != "null" => {
                cursor = Some(c);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            _ => break,
        }
    }

    Ok(all_records)
}

/// Fetch all entitlements with pagination.
pub(super) fn fetch_entitlements(
    agent: &ureq::Agent,
    token: &str,
    account_id: &str,
) -> Result<Vec<Entitlement>, String> {
    let mut all_entitlements = Vec::new();
    let mut start = 0;
    let count = 1000;

    loop {
        let url = format!(
            "{}/entitlement/api/account/{}/entitlements?start={}&count={}",
            ENTITLEMENTS_HOST, account_id, start, count,
        );

        let body = crate::api_client::http_get_bearer(agent, &url, token, 2)?;
        let page: Vec<Entitlement> =
            serde_json::from_str(&body).map_err(|e| format!("Entitlements parse error: {}", e))?;

        let page_len = page.len();
        all_entitlements.extend(page);

        if page_len < count {
            break;
        }
        start += count;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(all_entitlements)
}

/// Fetch launcher assets — the primary endpoint the EGS Launcher uses.
/// This returns all installable games for the user on a given platform.
/// Free claimed games typically appear here even if absent from the library service.
/// Non-fatal: returns empty vec on error rather than failing the whole import.
pub(super) fn fetch_launcher_assets(agent: &ureq::Agent, token: &str) -> Vec<LauncherAsset> {
    let url = format!(
        "{}/launcher/api/public/assets/Windows?label=Live",
        LAUNCHER_HOST,
    );

    match crate::api_client::http_get_bearer(agent, &url, token, 2) {
        Ok(body) => match serde_json::from_str::<Vec<LauncherAsset>>(&body) {
            Ok(assets) => assets,
            Err(e) => {
                eprintln!("[epic] Launcher assets parse error: {}", e);
                eprintln!(
                    "[epic] Launcher assets body prefix: {}",
                    &body[..body.len().min(300)]
                );
                Vec::new()
            }
        },
        Err(e) => {
            eprintln!("[epic] Launcher assets fetch failed (non-fatal): {}", e);
            Vec::new()
        }
    }
}

/// Fetch catalog items in bulk for a given namespace.
/// Returns a map of catalog_item_id → CatalogItem.
pub(super) fn fetch_catalog_bulk(
    agent: &ureq::Agent,
    token: &str,
    namespace: &str,
    catalog_ids: &[&str],
    limiter: &mut crate::api_client::RateLimiter,
) -> HashMap<String, CatalogItem> {
    let mut result = HashMap::new();

    // Process in chunks of 50 (API limit)
    for chunk in catalog_ids.chunks(50) {
        let id_param = chunk.join(",");
        let url = format!(
            "{}/catalog/api/shared/namespace/{}/bulk/items?id={}&includeDLCDetails=true&includeMainGameDetails=true&country=US&locale=en",
            CATALOG_HOST, namespace, id_param,
        );

        match crate::api_client::http_get_bearer(agent, &url, token, 1) {
            Ok(body) => match serde_json::from_str::<HashMap<String, CatalogItem>>(&body) {
                Ok(items) => {
                    result.extend(items);
                    limiter.on_success();
                }
                Err(e) => {
                    eprintln!("[epic] Catalog parse error for ns={}: {}", namespace, e);
                }
            },
            Err(e) => {
                if e.contains("429") {
                    limiter.on_rate_limited();
                }
                eprintln!("[epic] Catalog fetch error for ns={}: {}", namespace, e);
            }
        }

        limiter.wait();
    }

    result
}

/// Try to fetch metadata from the Epic Games Store for a given namespace/catalog ID.
/// Returns (description, cover, tags, release_date, was_rate_limited).
pub(super) fn fetch_store_metadata(
    namespace: &str,
    catalog_id: &str,
) -> (String, String, String, String, bool) {
    let empty = (
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        false,
    );

    if namespace.is_empty() && catalog_id.is_empty() {
        return empty;
    }

    let url = format!(
        "https://store-content-ipv4.ak.epicgames.com/api/en-US/content/products/{}",
        namespace
    );
    let agent = crate::api_client::api_agent();

    let body = match crate::api_client::http_get_retry(&agent, &url, 1) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "[epic] Failed to fetch store metadata for {}: {}",
                namespace, e
            );
            let was_rate_limited = e.contains("429");
            return (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                was_rate_limited,
            );
        }
    };

    let val: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[epic] Failed to parse store JSON for {}: {}", namespace, e);
            return empty;
        }
    };

    let description = val
        .get("pages")
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("data"))
        .and_then(|d| d.get("about"))
        .and_then(|a| a.get("shortDescription"))
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    let cover = val
        .get("pages")
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("data"))
        .and_then(|d| d.get("hero"))
        .and_then(|h| h.get("backgroundImageUrl"))
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();

    let tags = val
        .get("pages")
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("data"))
        .and_then(|d| d.get("meta"))
        .and_then(|m| m.get("tags"))
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();

    let release_date = val
        .get("pages")
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("data"))
        .and_then(|d| d.get("meta"))
        .and_then(|m| m.get("releaseDate"))
        .and_then(|r| r.as_str())
        .unwrap_or("")
        .to_string();

    (description, cover, tags, release_date, false)
}
