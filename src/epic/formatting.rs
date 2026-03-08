use super::types::CatalogItem;

/// Capitalize the first letter of each word.
pub(super) fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert an app_name codename to a readable title.
/// E.g. "SaltAndSanctuary" → "Salt And Sanctuary", "MyGame_v2" → "My Game V2"
pub(super) fn humanize_app_name(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }
    // Insert spaces before uppercase letters in CamelCase, and replace underscores
    let mut result = String::with_capacity(name.len() + 8);
    for (i, ch) in name.chars().enumerate() {
        if ch == '_' {
            result.push(' ');
        } else if i > 0
            && ch.is_uppercase()
            && !name
                .as_bytes()
                .get(i - 1)
                .is_none_or(|b| b.is_ascii_uppercase() || *b == b'_')
        {
            result.push(' ');
            result.push(ch);
        } else {
            result.push(ch);
        }
    }
    result
}

/// Extract description, cover URL, and release date from catalog data.
pub(super) fn extract_catalog_metadata(catalog: Option<&CatalogItem>) -> (String, String, String) {
    match catalog {
        Some(cat) => {
            let desc = if !cat.description.is_empty() {
                cat.description.clone()
            } else {
                cat.long_description.clone()
            };

            let cover = cat
                .key_images
                .iter()
                .find(|ki| ki.image_type == "DieselGameBoxTall")
                .or_else(|| {
                    cat.key_images
                        .iter()
                        .find(|ki| ki.image_type == "OfferImageTall")
                })
                .or_else(|| {
                    cat.key_images
                        .iter()
                        .find(|ki| ki.image_type == "DieselGameBox")
                })
                .or_else(|| {
                    cat.key_images
                        .iter()
                        .find(|ki| ki.image_type == "Thumbnail")
                })
                .or_else(|| {
                    cat.key_images
                        .iter()
                        .find(|ki| ki.image_type == "DieselStoreFrontWide")
                })
                .or_else(|| {
                    cat.key_images
                        .iter()
                        .find(|ki| ki.image_type == "OfferImageWide")
                })
                .or_else(|| {
                    cat.key_images
                        .iter()
                        .find(|ki| ki.image_type == "CodeRedemption_340x440")
                })
                .or_else(|| cat.key_images.first()) // any image is better than none
                .map(|ki| ki.url.clone())
                .unwrap_or_default();

            let rd = cat
                .release_info
                .first()
                .and_then(|ri| ri.date_added.as_deref())
                .map(|d| d.get(..10).unwrap_or(d).to_string())
                .unwrap_or_default();

            (desc, cover, rd)
        }
        None => (String::new(), String::new(), String::new()),
    }
}

/// Convert a store slug to a readable title.
/// Strips trailing hex hash suffixes (e.g. "hexguardian-2b0cbe" → "Hexguardian")
/// and capitalizes words.
pub(super) fn slug_to_title(slug: &str) -> String {
    // Strip trailing hex hash suffix added by Epic (e.g. "-2b0cbe")
    let clean = if let Some(last_dash) = slug.rfind('-') {
        let suffix = &slug[last_dash + 1..];
        if suffix.len() >= 4 && suffix.len() <= 8 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
            &slug[..last_dash]
        } else {
            slug
        }
    } else {
        slug
    };
    capitalize_words(&clean.replace('-', " "))
}

/// Derive a store slug from a game title for the store-content API.
pub(super) fn title_to_store_slug(title: &str) -> String {
    title
        .to_lowercase()
        .replace(' ', "-")
        .replace(':', "")
        .replace("'", "")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}
