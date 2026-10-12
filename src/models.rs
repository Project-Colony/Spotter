use colony_ui::preferences::StandardPreferences;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Game {
    pub id: Option<i64>,
    pub title: String,
    pub platform: Platform,
    pub playtime_minutes: u32,
    pub achievements_unlocked: u32,
    pub achievements_total: u32,
    pub status: GameStatus,
    pub rating: Option<u8>,
    pub genre: String,
    pub last_played: String,
    pub cover_url: String,
    pub steam_appid: Option<u32>,
    pub gog_id: Option<String>,
    pub epic_id: Option<String>,
    pub xbox_id: Option<String>,
    pub psn_id: Option<String>,
    pub notes: String,
    pub description: String,
    pub release_date: String,
    pub review_percent: Option<u32>,
    pub tags: String,
}

impl Game {
    pub fn playtime_display(&self) -> String {
        format_playtime(self.playtime_minutes)
    }

    pub fn achievement_percent(&self) -> f32 {
        if self.achievements_total == 0 {
            return 0.0;
        }
        (self.achievements_unlocked as f32 / self.achievements_total as f32) * 100.0
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum Platform {
    Steam,
    Gog,
    Epic,
    PlayStation,
    Xbox,
    Nintendo,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Steam => write!(f, "Steam"),
            Platform::Gog => write!(f, "GOG"),
            Platform::Epic => write!(f, "Epic"),
            Platform::PlayStation => write!(f, "PlayStation"),
            Platform::Xbox => write!(f, "Xbox"),
            Platform::Nintendo => write!(f, "Nintendo"),
        }
    }
}

impl Platform {
    pub const ALL: [Platform; 6] = [
        Platform::Steam,
        Platform::Gog,
        Platform::Epic,
        Platform::PlayStation,
        Platform::Xbox,
        Platform::Nintendo,
    ];

    pub fn all() -> &'static [Platform] {
        &Self::ALL
    }

    /// The platform's identity colour: the shared accent closest to its brand.
    /// Not `colony_ui::app_tint`, whose name hash gives Steam and PlayStation
    /// the same red, and Epic and Nintendo the same indigo.
    pub fn color(&self) -> iced::Color {
        crate::theme::shared_accent(self.accent_key())
    }

    /// The shared Colony accent that identifies this platform.
    pub fn accent_key(&self) -> &'static str {
        match self {
            Platform::Steam => "blue",
            Platform::Gog => "violet",
            Platform::Epic => "yellow",
            Platform::PlayStation => "indigo",
            Platform::Xbox => "green",
            Platform::Nintendo => "red",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s {
            "Steam" => Platform::Steam,
            "GOG" | "Gog" => Platform::Gog,
            "Epic" => Platform::Epic,
            "PlayStation" => Platform::PlayStation,
            "Xbox" => Platform::Xbox,
            "Nintendo" => Platform::Nintendo,
            _ => {
                eprintln!(
                    "[models] Warning: unknown platform '{}', defaulting to Steam",
                    s
                );
                Platform::Steam
            }
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Platform::Steam => "\u{f1b6}",        // nf-fa-steam
            Platform::Gog => "\u{f1b2}",          // nf-fa-cube (galaxy)
            Platform::Epic => "\u{f0e7}",         // nf-fa-bolt
            Platform::PlayStation => "\u{f0414}", // nf-md-sony_playstation
            Platform::Xbox => "\u{f05b0}",        // nf-md-microsoft_xbox_controller
            Platform::Nintendo => "\u{f07e1}",    // nf-md-nintendo_switch
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum GameStatus {
    Playing,
    Completed,
    Unplayed,
    Dropped,
    Wishlist,
}

impl fmt::Display for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameStatus::Playing => write!(f, "Playing"),
            GameStatus::Completed => write!(f, "Completed"),
            GameStatus::Unplayed => write!(f, "Unplayed"),
            GameStatus::Dropped => write!(f, "Dropped"),
            GameStatus::Wishlist => write!(f, "Wishlist"),
        }
    }
}

impl GameStatus {
    pub const ALL: [GameStatus; 5] = [
        GameStatus::Playing,
        GameStatus::Completed,
        GameStatus::Unplayed,
        GameStatus::Dropped,
        GameStatus::Wishlist,
    ];

    pub fn all() -> &'static [GameStatus] {
        &Self::ALL
    }

    /// The status colour, a shared accent so that it reads the same on every
    /// theme and no two statuses share it.
    pub fn color(&self) -> iced::Color {
        crate::theme::shared_accent(self.accent_key())
    }

    /// The shared Colony accent that identifies this status.
    pub fn accent_key(&self) -> &'static str {
        match self {
            GameStatus::Playing => "green",
            GameStatus::Completed => "blue",
            GameStatus::Unplayed => "amber",
            GameStatus::Dropped => "red",
            GameStatus::Wishlist => "violet",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s {
            "Playing" => GameStatus::Playing,
            "Completed" => GameStatus::Completed,
            "Unplayed" | "Backlog" => GameStatus::Unplayed,
            "Dropped" => GameStatus::Dropped,
            "Wishlist" => GameStatus::Wishlist,
            _ => {
                eprintln!(
                    "[models] Warning: unknown game status '{}', defaulting to Unplayed",
                    s
                );
                GameStatus::Unplayed
            }
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            GameStatus::Playing => "\u{f04b}",   // nf-fa-play
            GameStatus::Completed => "\u{f091}", // nf-fa-trophy
            GameStatus::Unplayed => "\u{f10c}",  // nf-fa-circle-o
            GameStatus::Dropped => "\u{f00d}",   // nf-fa-times
            GameStatus::Wishlist => "\u{f08a}",  // nf-fa-heart-o
        }
    }
}

/// A single achievement with schema info + player unlock status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Achievement {
    pub api_name: String,
    pub display_name: String,
    pub description: String,
    pub icon_url: String,
    pub icon_gray_url: String,
    pub unlocked: bool,
    pub unlock_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,
    pub avatar_path: String,
    pub steam_api_key: String,
    pub steam_id: String,
    pub gog_token: String,
    pub gog_refresh_token: String,
    pub xbox_api_key: String,
    pub xbox_gamertag: String,
    pub xbox_xuid: String,
    pub psn_npsso: String,
    pub epic_token: String,
    pub epic_refresh_token: String,
    pub epic_account_id: String,
    pub epic_display_name: String,
    pub member_since: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    #[default]
    TitleAsc,
    TitleDesc,
    PlaytimeDesc,
    RatingDesc,
    LastPlayedDesc,
    FavoritesFirst,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::TitleAsc => write!(f, "A-Z"),
            SortOrder::TitleDesc => write!(f, "Z-A"),
            SortOrder::PlaytimeDesc => write!(f, "Playtime"),
            SortOrder::RatingDesc => write!(f, "Rating"),
            SortOrder::LastPlayedDesc => write!(f, "Recent"),
            SortOrder::FavoritesFirst => write!(f, "Favorites"),
        }
    }
}

impl SortOrder {
    pub const ALL: [SortOrder; 6] = [
        SortOrder::TitleAsc,
        SortOrder::TitleDesc,
        SortOrder::PlaytimeDesc,
        SortOrder::RatingDesc,
        SortOrder::LastPlayedDesc,
        SortOrder::FavoritesFirst,
    ];

    pub fn all() -> &'static [SortOrder] {
        &Self::ALL
    }
}

/// Format a minute count as a human-readable string (e.g. "123h 45m" or "3.5h").
/// Shared across stats, profile, and detail views.
pub fn format_playtime(minutes: u32) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours > 0 {
        format!("{}h {:02}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            username: "Player".into(),
            avatar_path: String::new(),
            steam_api_key: String::new(),
            steam_id: String::new(),
            gog_token: String::new(),
            gog_refresh_token: String::new(),
            xbox_api_key: String::new(),
            xbox_gamertag: String::new(),
            xbox_xuid: String::new(),
            psn_npsso: String::new(),
            epic_token: String::new(),
            epic_refresh_token: String::new(),
            epic_account_id: String::new(),
            epic_display_name: String::new(),
            member_since: chrono::Local::now().format("%Y-%m-%d").to_string(),
        }
    }
}

// ───── Settings ─────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsSection {
    General,
    Appearances,
    Accessibility,
}

impl fmt::Display for SettingsSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsSection::General => write!(f, "General"),
            SettingsSection::Appearances => write!(f, "Appearances"),
            SettingsSection::Accessibility => write!(f, "Accessibility"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateFormat {
    #[default]
    Ymd, // 2026-02-17
    Dmy, // 17/02/2026
    Mdy, // 02/17/2026
}

impl fmt::Display for DateFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateFormat::Ymd => write!(f, "YYYY-MM-DD"),
            DateFormat::Dmy => write!(f, "DD/MM/YYYY"),
            DateFormat::Mdy => write!(f, "MM/DD/YYYY"),
        }
    }
}

impl DateFormat {
    pub const ALL: [DateFormat; 3] = [DateFormat::Ymd, DateFormat::Dmy, DateFormat::Mdy];

    pub fn all() -> &'static [DateFormat] {
        &Self::ALL
    }

    pub fn format_str(&self) -> &str {
        match self {
            DateFormat::Ymd => "%Y-%m-%d",
            DateFormat::Dmy => "%d/%m/%Y",
            DateFormat::Mdy => "%m/%d/%Y",
        }
    }

    /// Reformat a "YYYY-MM-DD" date string into the chosen format.
    /// Returns the original string unchanged if parsing fails.
    pub fn format_date(&self, date: &str) -> String {
        if date.is_empty() || date == "\u{2014}" {
            return date.to_string();
        }
        match chrono::NaiveDate::parse_from_str(date.get(..10).unwrap_or(date), "%Y-%m-%d") {
            Ok(d) => d.format(self.format_str()).to_string(),
            Err(_) => date.to_string(),
        }
    }

    /// Reformat a Unix timestamp into the chosen date format with time.
    pub fn format_timestamp(&self, ts: u64) -> String {
        chrono::DateTime::from_timestamp(ts as i64, 0)
            .map(|dt| {
                let date_part = dt.format(self.format_str()).to_string();
                let time_part = dt.format("%H:%M").to_string();
                format!("{} {}", date_part, time_part)
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToastDuration {
    Short, // 2s
    #[default]
    Normal, // 4s
    Long,  // 8s
}

impl fmt::Display for ToastDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToastDuration::Short => write!(f, "Short (2s)"),
            ToastDuration::Normal => write!(f, "Normal (4s)"),
            ToastDuration::Long => write!(f, "Long (8s)"),
        }
    }
}

impl ToastDuration {
    pub const ALL: [ToastDuration; 3] = [
        ToastDuration::Short,
        ToastDuration::Normal,
        ToastDuration::Long,
    ];

    pub fn all() -> &'static [ToastDuration] {
        &Self::ALL
    }

    pub fn millis(&self) -> u64 {
        match self {
            ToastDuration::Short => 2000,
            ToastDuration::Normal => 4000,
            ToastDuration::Long => 8000,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementsDisplay {
    #[default]
    Full,
    Summary,
    Hidden,
}

impl fmt::Display for AchievementsDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AchievementsDisplay::Full => write!(f, "Full"),
            AchievementsDisplay::Summary => write!(f, "Summary"),
            AchievementsDisplay::Hidden => write!(f, "Hidden"),
        }
    }
}

impl AchievementsDisplay {
    pub const ALL: [AchievementsDisplay; 3] = [
        AchievementsDisplay::Full,
        AchievementsDisplay::Summary,
        AchievementsDisplay::Hidden,
    ];

    pub fn all() -> &'static [AchievementsDisplay] {
        &Self::ALL
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartScreen {
    #[default]
    Library,
    Statistics,
    Import,
    Profile,
}

impl fmt::Display for StartScreen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StartScreen::Library => write!(f, "Library"),
            StartScreen::Statistics => write!(f, "Statistics"),
            StartScreen::Import => write!(f, "Import"),
            StartScreen::Profile => write!(f, "Profile"),
        }
    }
}

#[allow(dead_code)]
impl StartScreen {
    pub const ALL: [StartScreen; 4] = [
        StartScreen::Library,
        StartScreen::Statistics,
        StartScreen::Import,
        StartScreen::Profile,
    ];

    pub fn all() -> &'static [StartScreen] {
        &Self::ALL
    }

    pub fn from_str_name(s: &str) -> Self {
        match s {
            "Library" => StartScreen::Library,
            "Statistics" => StartScreen::Statistics,
            "Import" => StartScreen::Import,
            "Profile" => StartScreen::Profile,
            _ => StartScreen::Library,
        }
    }
}

/// Current settings schema version. 2: the settings moved from the database
/// to `preferences.json`, with the shared Colony preferences inside.
pub const SETTINGS_VERSION: u32 = 2;

/// Everything the user chose. Saved to `preferences.json` (see
/// `crate::preferences`); a field missing from the file takes its default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Schema version for forward-compatible deserialization.
    pub version: u32,
    /// Theme, accent, text sizes and the accessibility switches every Colony
    /// program shares, stored under the same names as theirs.
    #[serde(flatten)]
    pub standard: StandardPreferences,
    // General
    pub default_status: GameStatus,
    pub default_platform: Platform,
    pub confirm_before_delete: bool,
    pub start_screen: StartScreen,
    pub notifications_enabled: bool,
    pub toast_duration: ToastDuration,
    pub date_format: DateFormat,
    pub default_sort_order: SortOrder,
    pub remember_filters: bool,
    pub last_filter_status: Option<GameStatus>,
    pub last_filter_platform: Option<Platform>,
    pub show_game_descriptions: bool,
    pub achievements_display: AchievementsDisplay,
    pub download_covers_auto: bool,
    pub favorites: Vec<i64>,
    // Appearance
    pub compact_list: bool,
    pub show_covers_in_list: bool,
    pub sidebar_width: u16,
    // Accessibility
    pub show_status_labels: bool,
    pub large_click_targets: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            standard: StandardPreferences::default(),
            default_status: GameStatus::Unplayed,
            default_platform: Platform::Steam,
            confirm_before_delete: true,
            start_screen: StartScreen::Library,
            notifications_enabled: true,
            toast_duration: ToastDuration::Normal,
            date_format: DateFormat::Ymd,
            default_sort_order: SortOrder::TitleAsc,
            remember_filters: true,
            last_filter_status: None,
            last_filter_platform: None,
            show_game_descriptions: false,
            achievements_display: AchievementsDisplay::Full,
            download_covers_auto: true,
            favorites: Vec::new(),
            compact_list: false,
            show_covers_in_list: true,
            sidebar_width: 220,
            show_status_labels: true,
            large_click_targets: false,
        }
    }
}

impl Settings {
    /// Vertical padding of buttons and chips, larger with "Large click targets".
    pub fn button_padding(&self) -> u16 {
        if self.large_click_targets {
            12
        } else {
            8
        }
    }
}

/// Case-insensitive contains check. Uses fast ASCII path when possible,
/// falls back to streaming char-by-char comparison for Unicode (no allocation).
/// The `needle` MUST already be lowercase.
pub fn contains_ci(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.is_ascii() {
        // Fast path: no allocation
        let h = haystack.as_bytes();
        let n = needle.as_bytes();
        if n.len() > h.len() {
            return false;
        }
        h.windows(n.len()).any(|w| {
            w.iter()
                .zip(n.iter())
                .all(|(a, b)| a.to_ascii_lowercase() == *b)
        })
    } else {
        // Unicode path: streaming comparison without allocating a full lowercased String.
        let needle_chars: Vec<char> = needle.chars().collect();
        let nlen = needle_chars.len();
        let haystack_lower: Vec<char> = haystack.chars().flat_map(char::to_lowercase).collect();
        if haystack_lower.len() < nlen {
            return false;
        }
        haystack_lower
            .windows(nlen)
            .any(|w| w == needle_chars.as_slice())
    }
}
