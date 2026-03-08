use std::collections::HashMap;

use super::api::*;
use super::filters::*;
use super::formatting::*;
use super::graphql::*;
use super::types::*;
use crate::models::*;

/// Import the full Epic Games library using the account API.
/// This retrieves ALL owned games (not just installed) with metadata.
/// Returns (games, optionally refreshed tokens).
pub fn full_import_online(
    token: &str,
    account_id: &str,
    refresh_token: &str,
) -> Result<(Vec<Game>, Option<crate::epic::auth::EpicLoginResult>), String> {
    let logger = crate::api_client::ImportLogger::new(&crate::api_client::log_dir());
    logger.log("epic", "Starting Epic Games online import");

    // Try with current token first; if it fails with 401, refresh and retry
    match import_with_token(token, account_id, &logger) {
        Ok(games) => Ok((games, None)),
        Err(e) if e.contains("401") && !refresh_token.is_empty() => {
            eprintln!("[epic] Token expired, refreshing...");
            logger.log("epic", "Token expired, refreshing");
            let refreshed = crate::epic::auth::refresh_token(refresh_token)?;
            let games = import_with_token(&refreshed.0, &refreshed.2, &logger)?;
            Ok((games, Some(refreshed)))
        }
        Err(e) => Err(e),
    }
}

/// Internal: perform the actual import with a valid token.
fn import_with_token(
    token: &str,
    account_id: &str,
    logger: &crate::api_client::ImportLogger,
) -> Result<Vec<Game>, String> {
    let agent = crate::api_client::api_agent();

    // Step 1: Fetch library items (paginated)
    let library_items = fetch_library(&agent, token)?;
    eprintln!("[epic] Library: {} items", library_items.len());
    logger.log(
        "epic",
        &format!("Library returned {} items", library_items.len()),
    );

    // Diagnostic: dump first 5 library records so we can see actual API structure
    for (i, item) in library_items.iter().take(5).enumerate() {
        let meta_keys: Vec<&str> = item.metadata.iter().map(|m| m.key.as_str()).collect();
        let meta_title = metadata_value(&item.metadata, "title");
        eprintln!(
            "[epic] Sample record {}: ns={}, catId={}, appName={}, metadata_keys={:?}, meta_title={:?}",
            i, item.namespace, item.catalog_item_id, item.app_name, meta_keys, meta_title
        );
    }

    // Step 2: Fetch entitlements to catch items not in library service
    let entitlements = fetch_entitlements(&agent, token, account_id)?;
    eprintln!("[epic] Entitlements: {} items", entitlements.len());
    logger.log(
        "epic",
        &format!("Entitlements returned {} items", entitlements.len()),
    );

    // Diagnostic: dump all entitlements
    for (i, ent) in entitlements.iter().enumerate() {
        eprintln!(
            "[epic] Entitlement {}: ns={}, catId={}, name={}",
            i, ent.namespace, ent.catalog_item_id, ent.entitlement_name
        );
    }

    // Step 2b: Fetch launcher assets — this is the primary source used by the
    // actual EGS Launcher to list installable games. Free claimed games often
    // appear here even when absent from the library service.
    let launcher_assets = fetch_launcher_assets(&agent, token);
    eprintln!("[epic] Launcher assets: {} items", launcher_assets.len());
    logger.log(
        "epic",
        &format!("Launcher assets returned {} items", launcher_assets.len()),
    );
    for (i, asset) in launcher_assets.iter().take(10).enumerate() {
        eprintln!(
            "[epic] Asset {}: ns={}, catId={}, appName={}",
            i, asset.namespace, asset.catalog_item_id, asset.app_name
        );
    }

    // Step 3: Merge library + entitlements + launcher assets, dedup by (namespace, catalogItemId)
    let mut seen_catalog: std::collections::HashSet<(String, String)> =
        std::collections::HashSet::new();
    let mut all_items: Vec<LibraryRecord> = Vec::new();

    for item in library_items {
        // Skip Unreal Engine content: namespace "ue" or app_name starting with "UE_"
        if is_ue_content(&item.namespace, &item.app_name) {
            continue;
        }

        // Skip non-game items based on app_name patterns
        if is_non_game_app_name(&item.app_name) {
            eprintln!("[epic] Skipping non-game: appName={}", item.app_name);
            continue;
        }

        // Dedup by (namespace, catalog_item_id)
        if !item.namespace.is_empty()
            && !item.catalog_item_id.is_empty()
            && !seen_catalog.insert((item.namespace.clone(), item.catalog_item_id.clone()))
        {
            continue;
        }

        all_items.push(item);
    }

    // Add entitlements that aren't already covered
    for ent in entitlements {
        if is_ue_content(&ent.namespace, &ent.entitlement_name) {
            continue;
        }
        if is_non_game_app_name(&ent.entitlement_name) {
            continue;
        }

        if !ent.namespace.is_empty()
            && !ent.catalog_item_id.is_empty()
            && seen_catalog.insert((ent.namespace.clone(), ent.catalog_item_id.clone()))
        {
            all_items.push(LibraryRecord {
                namespace: ent.namespace,
                catalog_item_id: ent.catalog_item_id,
                app_name: ent.entitlement_name,
                product_id: String::new(),
                sandbox_name: String::new(),
                metadata: Vec::new(),
            });
        }
    }

    // Add launcher assets that aren't already covered (free claimed games live here)
    for asset in launcher_assets {
        if is_ue_content(&asset.namespace, &asset.app_name) {
            continue;
        }
        if is_non_game_app_name(&asset.app_name) {
            continue;
        }

        if !asset.namespace.is_empty()
            && !asset.catalog_item_id.is_empty()
            && seen_catalog.insert((asset.namespace.clone(), asset.catalog_item_id.clone()))
        {
            eprintln!(
                "[epic] New game from launcher assets: ns={}, appName={}",
                asset.namespace, asset.app_name
            );
            all_items.push(LibraryRecord {
                namespace: asset.namespace,
                catalog_item_id: asset.catalog_item_id,
                app_name: asset.app_name,
                product_id: String::new(),
                sandbox_name: String::new(),
                metadata: Vec::new(),
            });
        }
    }

    // Log all items that survived filtering
    for (i, item) in all_items.iter().enumerate() {
        eprintln!(
            "[epic] Filtered item {}: ns={}, catId={}, appName={}, productId={}",
            i, item.namespace, item.catalog_item_id, item.app_name, item.product_id
        );
    }

    eprintln!("[epic] Items after filtering & dedup: {}", all_items.len());
    logger.log(
        "epic",
        &format!("Items after filtering: {}", all_items.len()),
    );

    // Step 4: Fetch catalog metadata in batches grouped by namespace.
    let mut pending: Vec<PendingGame> = Vec::new();
    let mut limiter = crate::api_client::RateLimiter::new(std::time::Duration::from_millis(200));

    let mut by_namespace: HashMap<String, Vec<&LibraryRecord>> = HashMap::new();
    for item in &all_items {
        if !item.namespace.is_empty() && !item.catalog_item_id.is_empty() {
            by_namespace
                .entry(item.namespace.clone())
                .or_default()
                .push(item);
        }
    }

    let total_namespaces = by_namespace.len();
    for (ns_idx, (namespace, items)) in by_namespace.iter().enumerate() {
        let catalog_ids: Vec<&str> = items.iter().map(|i| i.catalog_item_id.as_str()).collect();
        let catalog_map = fetch_catalog_bulk(&agent, token, namespace, &catalog_ids, &mut limiter);

        for item in items {
            let catalog = catalog_map.get(&item.catalog_item_id);

            // Title resolution: catalog title > humanized app_name
            let cat_title = catalog.map(|c| c.title.clone()).filter(|t| !t.is_empty());
            let title = cat_title.unwrap_or_else(|| humanize_app_name(&item.app_name));

            if title.is_empty() {
                continue;
            }

            // Skip items that are clearly not games based on resolved title
            if is_non_game_title_strict(&title) {
                eprintln!("[epic] Skipping non-game title: {}", title);
                continue;
            }

            // Check categories to filter out non-games (DLC, add-ons, engines)
            if let Some(cat) = catalog {
                let has_non_game_cat = cat.categories.iter().any(|c| {
                    c.path == "addons"
                        || c.path == "addons/durable"
                        || c.path == "engines"
                        || c.path == "engines/ue"
                });
                let has_game_cat = cat.categories.iter().any(|c| {
                    c.path == "games" || c.path == "games/edition" || c.path == "games/demo"
                });
                if has_non_game_cat && !has_game_cat {
                    eprintln!("[epic] Skipping non-game category: {}", title);
                    continue;
                }
            }

            let (description, cover_url, release_date) = extract_catalog_metadata(catalog);

            // Log catalog data availability for debugging sparse items
            if let Some(cat) = catalog {
                if cat.description.is_empty() && cat.key_images.is_empty() {
                    eprintln!(
                        "[epic] Catalog sparse for {}: no desc, no images (cats={:?})",
                        title,
                        cat.categories.iter().map(|c| &c.path).collect::<Vec<_>>()
                    );
                } else {
                    let img_types: Vec<&str> = cat
                        .key_images
                        .iter()
                        .map(|ki| ki.image_type.as_str())
                        .collect();
                    eprintln!(
                        "[epic] Catalog for {}: desc={}B, images={:?}, cover={}",
                        title,
                        cat.description.len(),
                        img_types,
                        if cover_url.is_empty() {
                            "none"
                        } else {
                            "found"
                        }
                    );
                }
            } else {
                eprintln!("[epic] Catalog miss for {}: no catalog entry found", title);
            }

            let epic_id = if !item.catalog_item_id.is_empty() {
                Some(item.catalog_item_id.clone())
            } else if !item.app_name.is_empty() {
                Some(item.app_name.clone())
            } else {
                None
            };

            pending.push(PendingGame {
                game: Game {
                    id: None,
                    title,
                    platform: Platform::Epic,
                    playtime_minutes: 0,
                    achievements_unlocked: 0,
                    achievements_total: 0,
                    status: GameStatus::Unplayed,
                    rating: None,
                    genre: String::new(),
                    last_played: String::new(),
                    cover_url,
                    steam_appid: None,
                    gog_id: None,
                    epic_id,
                    xbox_id: None,
                    psn_id: None,
                    notes: String::new(),
                    description,
                    release_date,
                    review_percent: None,
                    tags: String::new(),
                },
                product_id: item.product_id.clone(),
                namespace: item.namespace.clone(),
            });
        }

        if (ns_idx + 1) % 10 == 0 || ns_idx + 1 == total_namespaces {
            eprintln!(
                "[epic] Catalog progress: {}/{} namespaces, {} pending so far",
                ns_idx + 1,
                total_namespaces,
                pending.len()
            );
        }
    }

    // Step 5: Enrich and resolve unresolved titles.
    let mut enriched_count = 0;
    let mut enrich_limiter =
        crate::api_client::RateLimiter::new(std::time::Duration::from_millis(300));

    // Pre-resolve namespaces that contain hex-titled items using GraphQL API.
    let mut namespace_slugs: HashMap<String, String> = HashMap::new();
    let mut namespace_titles: HashMap<String, String> = HashMap::new();
    {
        let hex_namespaces: std::collections::HashSet<String> = pending
            .iter()
            .filter(|pg| is_hex_id(&pg.game.title) && !pg.namespace.is_empty())
            .map(|pg| pg.namespace.clone())
            .collect();
        for ns in &hex_namespaces {
            if let Some((slug, title)) = fetch_graphql_product_info(&agent, ns) {
                if !slug.is_empty() {
                    eprintln!(
                        "[epic] GraphQL resolved namespace {} -> slug \"{}\"",
                        ns, slug
                    );
                    namespace_slugs.insert(ns.clone(), slug);
                }
                if let Some(t) = title {
                    eprintln!(
                        "[epic] GraphQL resolved namespace {} -> title \"{}\"",
                        ns, t
                    );
                    namespace_titles.insert(ns.clone(), t);
                }
            } else {
                eprintln!("[epic] GraphQL: no info found for namespace {}", ns);
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    for pg in &mut pending {
        let is_hex_title = is_hex_id(&pg.game.title);
        let needs_enrich = pg.game.cover_url.is_empty() && pg.game.description.is_empty();

        if !is_hex_title && !needs_enrich {
            continue;
        }

        let mut resolved = false;

        // (a) Try GraphQL-resolved info for the namespace
        if !resolved && is_hex_title {
            // If GraphQL returned a title directly, use it
            if let Some(graphql_title) = namespace_titles.get(&pg.namespace) {
                eprintln!(
                    "[epic] Resolved hex title via GraphQL title: {} -> {}",
                    pg.game.title, graphql_title
                );
                pg.game.title = graphql_title.clone();
            }

            // Derive title from GraphQL slug BEFORE trying store API.
            if let Some(slug) = namespace_slugs.get(&pg.namespace) {
                if is_hex_id(&pg.game.title) {
                    let readable = slug_to_title(slug);
                    eprintln!(
                        "[epic] Resolved hex title via GraphQL slug: {} -> {}",
                        pg.game.title, readable
                    );
                    pg.game.title = readable;
                }
                resolved = true;

                // Try to enrich with store metadata (description, cover, etc.)
                eprintln!("[epic] Trying store lookup with GraphQL slug: {}", slug);
                let (desc, cover, tags, rd, was_rate_limited) = fetch_store_metadata(slug, "");

                let found = !desc.is_empty() || !cover.is_empty();
                if found {
                    enriched_count += 1;
                }
                if !desc.is_empty() {
                    pg.game.description = desc;
                }
                if !cover.is_empty() {
                    pg.game.cover_url = cover;
                }
                if !tags.is_empty() {
                    pg.game.tags = tags;
                }
                if !rd.is_empty() {
                    pg.game.release_date = rd;
                }
                if was_rate_limited {
                    enrich_limiter.on_rate_limited();
                } else {
                    enrich_limiter.on_success();
                }
                enrich_limiter.wait();
            }
            // If we got a title from GraphQL but no slug, still mark as resolved
            if !resolved && !is_hex_id(&pg.game.title) {
                resolved = true;
            }
        }

        // (b) Try productId as slug (strip "prod-" prefix if present)
        if !resolved && !pg.product_id.is_empty() {
            let slug = pg
                .product_id
                .strip_prefix("prod-")
                .unwrap_or(&pg.product_id);
            if !slug.is_empty() && !is_hex_id(slug) {
                eprintln!("[epic] Trying store lookup with productId slug: {}", slug);
                let (desc, cover, tags, rd, was_rate_limited) = fetch_store_metadata(slug, "");

                let found = !desc.is_empty() || !cover.is_empty();
                if found {
                    if is_hex_title {
                        let readable = capitalize_words(&slug.replace('-', " "));
                        eprintln!(
                            "[epic] Resolved hex title via productId: {} -> {}",
                            pg.game.title, readable
                        );
                        pg.game.title = readable;
                    }
                    enriched_count += 1;
                    resolved = true;
                }
                if !desc.is_empty() {
                    pg.game.description = desc;
                }
                if !cover.is_empty() {
                    pg.game.cover_url = cover;
                }
                if !tags.is_empty() {
                    pg.game.tags = tags;
                }
                if !rd.is_empty() {
                    pg.game.release_date = rd;
                }
                if was_rate_limited {
                    enrich_limiter.on_rate_limited();
                } else {
                    enrich_limiter.on_success();
                }
                enrich_limiter.wait();
            }
        }

        // (c) Try title slug (only for non-hex titles)
        if !resolved && !is_hex_title && needs_enrich {
            let slug = title_to_store_slug(&pg.game.title);
            if !slug.is_empty() {
                let (desc, cover, tags, rd, was_rate_limited) = fetch_store_metadata(&slug, "");

                let did_enrich = !desc.is_empty() || !cover.is_empty();
                if !desc.is_empty() {
                    pg.game.description = desc;
                }
                if !cover.is_empty() {
                    pg.game.cover_url = cover;
                }
                if !tags.is_empty() {
                    pg.game.tags = tags;
                }
                if !rd.is_empty() {
                    pg.game.release_date = rd;
                }
                if did_enrich {
                    enriched_count += 1;
                }
                if was_rate_limited {
                    enrich_limiter.on_rate_limited();
                } else {
                    enrich_limiter.on_success();
                }
                enrich_limiter.wait();
            }
        }

        // (d) GraphQL searchStore fallback — when all store-content lookups 404'd
        let still_needs = pg.game.cover_url.is_empty() && pg.game.description.is_empty();
        if still_needs && !pg.namespace.is_empty() {
            eprintln!(
                "[epic] Trying GraphQL metadata fallback for ns={}",
                pg.namespace
            );
            let (desc, cover, tags, rd) = fetch_graphql_offer_metadata(&agent, &pg.namespace);
            let found = !desc.is_empty() || !cover.is_empty();
            if found {
                enriched_count += 1;
            }
            if !desc.is_empty() {
                pg.game.description = desc;
            }
            if !cover.is_empty() {
                pg.game.cover_url = cover;
            }
            if !tags.is_empty() {
                pg.game.tags = tags;
            }
            if !rd.is_empty() {
                pg.game.release_date = rd;
            }
            enrich_limiter.wait();
        }
    }

    if enriched_count > 0 {
        eprintln!(
            "[epic] Enriched {} games via store/GraphQL APIs",
            enriched_count
        );
    }

    // Step 5b: Fetch community ratings via GraphQL RatingsPolls API.
    {
        let mut ns_ratings: HashMap<String, Option<u32>> = HashMap::new();
        for pg in &mut pending {
            if pg.namespace.is_empty() || pg.game.review_percent.is_some() {
                continue;
            }
            if !ns_ratings.contains_key(&pg.namespace) {
                let rating = fetch_graphql_rating(&agent, &pg.namespace);
                if let Some(pct) = rating {
                    eprintln!("[epic] Rating for ns={}: {}%", pg.namespace, pct);
                }
                ns_ratings.insert(pg.namespace.clone(), rating);
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            if let Some(Some(pct)) = ns_ratings.get(&pg.namespace) {
                pg.game.review_percent = Some(*pct);
            }
        }
        let rated = ns_ratings.values().filter(|v| v.is_some()).count();
        if rated > 0 {
            eprintln!("[epic] Fetched ratings for {} games", rated);
        }
    }

    // Step 5c: Fetch achievement definitions via GraphQL (public, no auth needed).
    {
        let mut ns_achievements: HashMap<String, u32> = HashMap::new();
        for pg in &mut pending {
            if pg.namespace.is_empty() {
                continue;
            }
            if !ns_achievements.contains_key(&pg.namespace) {
                let (_, total) = fetch_achievements(&agent, &pg.namespace);
                if total > 0 {
                    eprintln!(
                        "[epic] Achievements for ns={}: {} total",
                        pg.namespace, total
                    );
                }
                ns_achievements.insert(pg.namespace.clone(), total);
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            if let Some(&total) = ns_achievements.get(&pg.namespace) {
                pg.game.achievements_total = total;
            }
        }
        let with_achievements = ns_achievements.values().filter(|t| **t > 0).count();
        if with_achievements > 0 {
            eprintln!(
                "[epic] Fetched achievement definitions for {} games",
                with_achievements
            );
        }
    }

    // Step 6: Build final game list, filtering out items that still have unresolvable titles.
    let mut games: Vec<Game> = Vec::new();
    for pg in pending {
        // Filter out items that still have hex/UUID titles after all resolution attempts
        if is_hex_id(&pg.game.title) {
            eprintln!(
                "[epic] Dropping unresolvable item: title={}, productId={}",
                pg.game.title, pg.product_id
            );
            continue;
        }
        // Also apply the full junk filter now
        if is_non_game_title(&pg.game.title) {
            eprintln!("[epic] Dropping non-game title: {}", pg.game.title);
            continue;
        }
        eprintln!("[epic] + {}", pg.game.title);
        games.push(pg.game);
    }

    // Final dedup by title (case-insensitive)
    let mut seen_titles: std::collections::HashSet<String> = std::collections::HashSet::new();
    games.retain(|g| seen_titles.insert(g.title.to_lowercase()));

    eprintln!("[epic] Final game count: {}", games.len());
    logger.log(
        "epic",
        &format!("Online import complete: {} games", games.len()),
    );
    Ok(games)
}
