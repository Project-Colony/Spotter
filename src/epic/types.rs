use serde::Deserialize;

/// A single record from the library service.
#[derive(Debug, Deserialize)]
pub(super) struct LibraryRecord {
    #[serde(default)]
    pub namespace: String,
    #[serde(alias = "catalogItemId", default)]
    pub catalog_item_id: String,
    #[serde(alias = "appName", default)]
    pub app_name: String,
    /// Product ID from the library service (e.g. "prod-fn" for Fortnite).
    /// Can sometimes be used as a store slug for metadata lookup.
    #[serde(alias = "productId", default)]
    pub product_id: String,
    /// Sandbox name (e.g. "Live", "UE Marketplace") — useful for filtering.
    #[serde(alias = "sandboxName", default)]
    #[allow(dead_code)]
    pub sandbox_name: String,
    /// Metadata array returned when `includeMetadata=true`.
    /// In practice the API does NOT populate this field (always empty).
    #[serde(default)]
    pub metadata: Vec<MetadataEntry>,
}

#[derive(Debug, Deserialize)]
pub(super) struct MetadataEntry {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: String,
}

/// Extract a value from the metadata array by key.
pub(super) fn metadata_value(metadata: &[MetadataEntry], key: &str) -> Option<String> {
    metadata
        .iter()
        .find(|m| m.key == key)
        .map(|m| m.value.clone())
        .filter(|v| !v.is_empty())
}

/// Response wrapper from the library service.
#[derive(Debug, Deserialize)]
pub(super) struct LibraryResponse {
    #[serde(default)]
    pub records: Vec<LibraryRecord>,
    #[serde(alias = "responseMetadata", default)]
    pub response_metadata: Option<ResponseMetadata>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ResponseMetadata {
    #[serde(alias = "nextCursor", default)]
    pub next_cursor: Option<String>,
}

/// A single entitlement record.
#[derive(Debug, Deserialize)]
pub(super) struct Entitlement {
    #[serde(default)]
    pub namespace: String,
    #[serde(alias = "catalogItemId", default)]
    pub catalog_item_id: String,
    #[serde(alias = "entitlementName", default)]
    pub entitlement_name: String,
}

/// A game asset from the launcher service.
/// This is the primary endpoint that the EGS Launcher uses to list installable games.
#[derive(Debug, Deserialize)]
pub(super) struct LauncherAsset {
    #[serde(alias = "appName", default)]
    pub app_name: String,
    #[serde(alias = "catalogItemId", default)]
    pub catalog_item_id: String,
    #[serde(default)]
    pub namespace: String,
}

/// Catalog item info from the bulk items endpoint.
#[derive(Debug, Deserialize)]
pub(super) struct CatalogItem {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(alias = "longDescription", default)]
    pub long_description: String,
    #[serde(alias = "keyImages", default)]
    pub key_images: Vec<KeyImage>,
    #[serde(default)]
    pub categories: Vec<CatalogCategory>,
    #[serde(alias = "releaseInfo", default)]
    pub release_info: Vec<ReleaseInfo>,
}

#[derive(Debug, Deserialize)]
pub(super) struct KeyImage {
    #[serde(alias = "type", default)]
    pub image_type: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct CatalogCategory {
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct ReleaseInfo {
    #[serde(alias = "dateAdded", default)]
    pub date_added: Option<String>,
}

/// Epic Games Launcher manifest entry (*.item files).
#[derive(Debug, Deserialize)]
pub(super) struct EpicManifest {
    #[serde(alias = "DisplayName", default)]
    pub display_name: String,
    #[serde(alias = "CatalogItemId", default)]
    pub catalog_item_id: String,
    #[serde(alias = "CatalogNamespace", default)]
    pub catalog_namespace: String,
    #[serde(alias = "AppName", default)]
    pub app_name: String,
    #[serde(alias = "AppCategories", default)]
    pub app_categories: Vec<String>,
}

/// Internal struct to track games pending enrichment during online import.
pub(super) struct PendingGame {
    pub game: crate::models::Game,
    pub product_id: String,
    pub namespace: String,
}
