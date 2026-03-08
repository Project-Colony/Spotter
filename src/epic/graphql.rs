/// Use the Epic Games Store GraphQL API to resolve a namespace to a product slug
/// and optionally a title. Tries multiple strategies:
///   1. catalogNs.mappings query → pageSlug
///   2. searchStore query with namespace filter → title + slug
///
/// Returns (slug, optional_title) if found.
pub(super) fn fetch_graphql_product_info(
    agent: &ureq::Agent,
    namespace: &str,
) -> Option<(String, Option<String>)> {
    // The correct EGS GraphQL endpoint (confirmed from epicstore_api library)
    let graphql_url = "https://store.epicgames.com/graphql";

    // Strategy 1: catalogNs.mappings → pageSlug
    let query1 = serde_json::json!({
        "query": "query catalogNs($namespace: String!) { Catalog { catalogNs(namespace: $namespace) { mappings(pageType: \"productHome\") { pageSlug pageType } } } }",
        "variables": { "namespace": namespace }
    });
    if let Some(result) = graphql_post(agent, graphql_url, &query1, namespace) {
        // Extract pageSlug from: data.Catalog.catalogNs.mappings[0].pageSlug
        let slug = result
            .get("data")
            .and_then(|d| d.get("Catalog"))
            .and_then(|c| c.get("catalogNs"))
            .and_then(|n| n.get("mappings"))
            .and_then(|m| m.as_array())
            .and_then(|arr| arr.first())
            .and_then(|m| m.get("pageSlug"))
            .and_then(|s| s.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        if let Some(slug) = slug {
            return Some((slug, None));
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(200));

    // Strategy 2: searchStore with namespace → title + slug
    let query2 = serde_json::json!({
        "query": "query searchStore($namespace: String!) { Catalog { searchStore(namespace: $namespace, count: 1, category: \"games\") { elements { title catalogNs { mappings(pageType: \"productHome\") { pageSlug } } offerMappings { pageSlug } } } } }",
        "variables": { "namespace": namespace }
    });
    if let Some(result) = graphql_post(agent, graphql_url, &query2, namespace) {
        let element = result
            .get("data")
            .and_then(|d| d.get("Catalog"))
            .and_then(|c| c.get("searchStore"))
            .and_then(|s| s.get("elements"))
            .and_then(|e| e.as_array())
            .and_then(|arr| arr.first());
        if let Some(el) = element {
            let title = el
                .get("title")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string());
            // Try catalogNs.mappings first, then offerMappings
            let slug = el
                .get("catalogNs")
                .and_then(|c| c.get("mappings"))
                .and_then(|m| m.as_array())
                .and_then(|arr| arr.first())
                .and_then(|m| m.get("pageSlug"))
                .and_then(|s| s.as_str())
                .or_else(|| {
                    el.get("offerMappings")
                        .and_then(|m| m.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|m| m.get("pageSlug"))
                        .and_then(|s| s.as_str())
                })
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            if slug.is_some() || title.is_some() {
                return Some((slug.unwrap_or_default(), title));
            }
        }
    }

    None
}

/// Helper: POST a GraphQL query and return the parsed JSON response.
pub(super) fn graphql_post(
    agent: &ureq::Agent,
    url: &str,
    query: &serde_json::Value,
    namespace: &str,
) -> Option<serde_json::Value> {
    let query_str = query.to_string();
    let resp = agent
        .post(url)
        .header("Content-Type", "application/json")
        .send(&query_str)
        .map_err(|e| {
            eprintln!("[epic] GraphQL request failed for ns={}: {}", namespace, e);
            e
        })
        .ok()?;

    let body: String = resp
        .into_body()
        .read_to_string()
        .map_err(|e| {
            eprintln!("[epic] GraphQL read error: {}", e);
            e
        })
        .ok()?;

    eprintln!(
        "[epic] GraphQL response for ns={}: {}",
        namespace,
        if body.len() > 200 {
            &body[..200]
        } else {
            &body
        }
    );

    serde_json::from_str(&body)
        .map_err(|e| {
            eprintln!("[epic] GraphQL parse error: {}", e);
            e
        })
        .ok()
}

/// Fetch community rating for a game via GraphQL RatingsPolls API.
/// Returns a review percentage (0–100) if available.
pub(super) fn fetch_graphql_rating(agent: &ureq::Agent, namespace: &str) -> Option<u32> {
    let graphql_url = "https://store.epicgames.com/graphql";
    let query = serde_json::json!({
        "query": "query getProductResult($sandboxId: String!) { RatingsPolls { getProductResult(sandboxId: $sandboxId) { averageRating } } }",
        "variables": { "sandboxId": namespace }
    });

    let result = graphql_post(agent, graphql_url, &query, namespace)?;

    let rating = result
        .get("data")
        .and_then(|d| d.get("RatingsPolls"))
        .and_then(|r| r.get("getProductResult"))
        .and_then(|p| p.get("averageRating"))
        .and_then(|a| a.as_f64())?;

    if rating > 0.0 {
        Some((rating / 5.0 * 100.0).round() as u32)
    } else {
        None
    }
}

/// Fetch achievement definitions for a game via the Epic Store GraphQL API.
/// Returns the total number of achievements defined for the game.
pub(super) fn fetch_achievement_definitions(agent: &ureq::Agent, namespace: &str) -> Option<u32> {
    let graphql_url = "https://store.epicgames.com/graphql";
    let query = serde_json::json!({
        "query": concat!(
            "query Achievement($sandboxId: String!, $locale: String) { ",
            "Achievement { productAchievementsRecordBySandbox(",
            "sandboxId: $sandboxId, locale: $locale) { ",
            "totalAchievements ",
            "achievementSets { totalAchievements } ",
            "} } }"
        ),
        "variables": { "sandboxId": namespace, "locale": "en-US" }
    });

    let result = graphql_post(agent, graphql_url, &query, namespace)?;

    // Check for errors
    if let Some(errors) = result.get("errors") {
        eprintln!(
            "[epic] Achievement definitions error for ns={}: {}",
            namespace, errors
        );
        return None;
    }

    let record = result
        .get("data")
        .and_then(|d| d.get("Achievement"))
        .and_then(|a| a.get("productAchievementsRecordBySandbox"))?;

    // Try totalAchievements at the top level first
    if let Some(total) = record.get("totalAchievements").and_then(|t| t.as_u64()) {
        if total > 0 {
            return Some(total as u32);
        }
    }

    // Fallback: sum totalAchievements from each achievementSet
    if let Some(sets) = record.get("achievementSets").and_then(|s| s.as_array()) {
        let total: u64 = sets
            .iter()
            .filter_map(|set| set.get("totalAchievements").and_then(|t| t.as_u64()))
            .sum();
        if total > 0 {
            return Some(total as u32);
        }
    }

    None
}

/// Fetch achievement total for a game.
/// Returns (0, achievements_total).
///
/// NOTE: Player-specific unlock counts are unavailable — Epic's launcher OAuth
/// client (launcherAppClient2) lacks the "achievements" scope, so all known
/// GraphQL endpoints reject the Bearer token for playerAchievementGameRecords.
/// Heroic and Legendary have the same limitation. We only fetch the public
/// definition count (total achievements) which works without auth.
pub(super) fn fetch_achievements(agent: &ureq::Agent, namespace: &str) -> (u32, u32) {
    let total = match fetch_achievement_definitions(agent, namespace) {
        Some(t) => t,
        None => return (0, 0),
    };

    (0, total)
}

/// Fetch product metadata via the Epic Store GraphQL `searchStore` query.
/// This is a fallback when the store-content API returns 404 (delisted / very new games).
/// Returns (description, cover_url, tags, release_date).
pub(super) fn fetch_graphql_offer_metadata(
    agent: &ureq::Agent,
    namespace: &str,
) -> (String, String, String, String) {
    let graphql_url = "https://store.epicgames.com/graphql";
    let query = serde_json::json!({
        "query": concat!(
            "query searchStore($namespace: String!) { ",
            "Catalog { searchStore(namespace: $namespace, count: 1, category: \"games\") { ",
            "elements { title description keyImages { type url } ",
            "tags { id name } effectiveDate } } } }"
        ),
        "variables": { "namespace": namespace }
    });

    let empty = (String::new(), String::new(), String::new(), String::new());

    let result = match graphql_post(agent, graphql_url, &query, namespace) {
        Some(r) => r,
        None => return empty,
    };

    let element = result
        .get("data")
        .and_then(|d| d.get("Catalog"))
        .and_then(|c| c.get("searchStore"))
        .and_then(|s| s.get("elements"))
        .and_then(|e| e.as_array())
        .and_then(|arr| arr.first());

    let el = match element {
        Some(e) => e,
        None => return empty,
    };

    let description = el
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("")
        .to_string();

    // Try same image type priority as extract_catalog_metadata, plus wider fallbacks
    let cover = el
        .get("keyImages")
        .and_then(|ki| ki.as_array())
        .and_then(|arr| {
            let find_type = |t: &str| {
                arr.iter()
                    .find(|ki| ki.get("type").and_then(|v| v.as_str()) == Some(t))
            };
            find_type("DieselGameBoxTall")
                .or_else(|| find_type("OfferImageTall"))
                .or_else(|| find_type("DieselGameBox"))
                .or_else(|| find_type("Thumbnail"))
                .or_else(|| find_type("DieselStoreFrontWide"))
                .or_else(|| find_type("OfferImageWide"))
                .or_else(|| find_type("CodeRedemption_340x440"))
                .or_else(|| arr.first()) // any image is better than none
        })
        .and_then(|ki| ki.get("url").and_then(|u| u.as_str()))
        .unwrap_or("")
        .to_string();

    let tags = el
        .get("tags")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();

    let release_date = el
        .get("effectiveDate")
        .and_then(|d| d.as_str())
        .map(|d| d.get(..10).unwrap_or(d).to_string())
        .unwrap_or_default();

    if !description.is_empty() || !cover.is_empty() {
        eprintln!(
            "[epic] GraphQL metadata: desc={}B, cover={}, tags={}",
            description.len(),
            if cover.is_empty() { "none" } else { "found" },
            if tags.is_empty() { "none" } else { &tags },
        );
    }

    (description, cover, tags, release_date)
}
