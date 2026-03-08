/// Check if an item is Unreal Engine content (not a game).
pub(super) fn is_ue_content(namespace: &str, app_name: &str) -> bool {
    namespace == "ue"
        || app_name.starts_with("UE_")
        || app_name == "UELauncher"
        || app_name.starts_with("UE-")
}

/// Check if an app_name matches known non-game patterns (promos, coupons, free tokens).
pub(super) fn is_non_game_app_name(app_name: &str) -> bool {
    let upper = app_name.to_uppercase();
    // Hex-only strings (32+ chars) are catalog IDs, not real app names
    if app_name.len() >= 32 && app_name.chars().all(|c| c.is_ascii_hexdigit()) {
        return false; // Don't skip — might resolve to a game via catalog
    }
    upper.contains("PROMO")
        || upper.contains("COUPON")
        || upper.contains("TRIAL_")
        || upper.ends_with("_FREE")
        || upper.starts_with("FREEBIE_")
        || upper.contains("_SUBS_")
        || upper == "FORTNITE_FREE"
}

/// Check if a resolved title looks like non-game content (includes hex IDs).
pub(super) fn is_non_game_title(title: &str) -> bool {
    is_epic_junk_title(title)
}

/// Like `is_non_game_title` but WITHOUT the hex ID check.
/// Used during Step 4 when we still want to give hex-titled items a chance
/// to be resolved via the store-content API.
pub(super) fn is_non_game_title_strict(title: &str) -> bool {
    let lower = title.to_lowercase();

    // Unreal Engine content
    if lower.contains("unreal engine")
        || lower.starts_with("ue ")
        || lower.starts_with("ue_")
        || lower.starts_with("ue-")
        || lower == "uelauncher"
        || lower == "ue launcher"
    {
        return true;
    }

    // Promotions, coupons, free tokens
    if lower.contains("_promo")
        || lower.contains("promo_")
        || lower.contains(" promo")
        || lower.contains("promo ")
        || lower.contains("coupon")
        || lower.contains("_subs_")
    {
        return true;
    }

    // Free access tokens
    if lower.ends_with("_free") || lower.ends_with(" free") {
        return true;
    }

    // Trial and freebie tokens
    if lower.starts_with("trial_")
        || lower.starts_with("freebie_")
        || lower.starts_with("trial ")
        || lower.starts_with("freebie ")
    {
        return true;
    }

    false
}

/// Check if a string looks like a hex ID (catalog ID, not a game title).
pub(super) fn is_hex_id(s: &str) -> bool {
    if s.len() >= 8 && s.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }
    // UUID-like: xxxxxxxx-xxxx-...
    s.len() >= 36 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-') && s.contains('-')
}

/// Public: check whether an Epic game title is clearly non-game content
/// (UE engine items, promos, hex IDs, free tokens, etc.).
/// Used both during import filtering and for cleaning up old bad data in the DB.
pub fn is_epic_junk_title(title: &str) -> bool {
    let lower = title.to_lowercase();

    // Pure hex strings (8+ chars) are unresolved catalog IDs
    if title.len() >= 8 && title.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }
    // UUID-like patterns (xxxxxxxx-xxxx-...)
    if title.len() >= 36
        && title.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
        && title.contains('-')
    {
        return true;
    }

    // Unreal Engine content
    if lower.contains("unreal engine")
        || lower.starts_with("ue ")
        || lower.starts_with("ue_")
        || lower.starts_with("ue-")
        || lower == "uelauncher"
        || lower == "ue launcher"
    {
        return true;
    }

    // Promotions, coupons, free tokens
    if lower.contains("_promo")
        || lower.contains("promo_")
        || lower.contains(" promo")
        || lower.contains("promo ")
        || lower.contains("coupon")
        || lower.contains("_subs_")
    {
        return true;
    }

    // Free access tokens (e.g. "Fortnite_Free", "Fortnite Free")
    if lower.ends_with("_free") || lower.ends_with(" free") {
        // "Fortnite_Free" is not the actual game, it's a free access entitlement
        return true;
    }

    // Trial and freebie tokens
    if lower.starts_with("trial_")
        || lower.starts_with("freebie_")
        || lower.starts_with("trial ")
        || lower.starts_with("freebie ")
    {
        return true;
    }

    false
}
