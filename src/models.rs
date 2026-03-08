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

    pub fn color(&self) -> iced::Color {
        match self {
            Platform::Steam => iced::Color::from_rgb(0.4, 0.6, 0.9),
            Platform::Gog => iced::Color::from_rgb(0.7, 0.5, 0.9),
            Platform::Epic => iced::Color::from_rgb(0.9, 0.9, 0.3),
            Platform::PlayStation => iced::Color::from_rgb(0.2, 0.4, 0.9),
            Platform::Xbox => iced::Color::from_rgb(0.2, 0.8, 0.2),
            Platform::Nintendo => iced::Color::from_rgb(0.9, 0.2, 0.2),
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

    pub fn color(&self) -> iced::Color {
        match self {
            GameStatus::Playing => iced::Color::from_rgb(0.2, 0.8, 0.4),
            GameStatus::Completed => iced::Color::from_rgb(0.3, 0.6, 1.0),
            GameStatus::Unplayed => iced::Color::from_rgb(0.9, 0.7, 0.2),
            GameStatus::Dropped => iced::Color::from_rgb(0.8, 0.3, 0.3),
            GameStatus::Wishlist => iced::Color::from_rgb(0.7, 0.5, 0.9),
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
pub enum ThemeMode {
    Dark,
    Darker,
    Midnight,
}

impl fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeMode::Dark => write!(f, "Dark"),
            ThemeMode::Darker => write!(f, "Darker"),
            ThemeMode::Midnight => write!(f, "Midnight"),
        }
    }
}

impl ThemeMode {
    pub const ALL: [ThemeMode; 3] = [ThemeMode::Dark, ThemeMode::Darker, ThemeMode::Midnight];

    pub fn all() -> &'static [ThemeMode] {
        &Self::ALL
    }

    /// Background color for main content area.
    pub fn bg_main(&self) -> iced::Color {
        match self {
            ThemeMode::Dark => iced::Color::from_rgb(0.1, 0.1, 0.14),
            ThemeMode::Darker => iced::Color::from_rgb(0.06, 0.06, 0.09),
            ThemeMode::Midnight => iced::Color::from_rgb(0.02, 0.02, 0.05),
        }
    }

    /// Background color for sidebar.
    pub fn bg_sidebar(&self) -> iced::Color {
        match self {
            ThemeMode::Dark => iced::Color::from_rgb(0.08, 0.08, 0.12),
            ThemeMode::Darker => iced::Color::from_rgb(0.04, 0.04, 0.07),
            ThemeMode::Midnight => iced::Color::from_rgb(0.01, 0.01, 0.03),
        }
    }

    /// Background color for cards/containers.
    pub fn bg_card(&self) -> iced::Color {
        match self {
            ThemeMode::Dark => iced::Color::from_rgb(0.12, 0.12, 0.16),
            ThemeMode::Darker => iced::Color::from_rgb(0.08, 0.08, 0.11),
            ThemeMode::Midnight => iced::Color::from_rgb(0.04, 0.04, 0.07),
        }
    }

    /// Slightly darker row background inside cards (e.g. settings rows).
    pub fn bg_row(&self) -> iced::Color {
        match self {
            ThemeMode::Dark => iced::Color::from_rgb(0.09, 0.09, 0.13),
            ThemeMode::Darker => iced::Color::from_rgb(0.06, 0.06, 0.09),
            ThemeMode::Midnight => iced::Color::from_rgb(0.03, 0.03, 0.05),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccentColor {
    Blue,
    Purple,
    Green,
    Gold,
    Red,
    Cyan,
}

impl fmt::Display for AccentColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccentColor::Blue => write!(f, "Blue"),
            AccentColor::Purple => write!(f, "Purple"),
            AccentColor::Green => write!(f, "Green"),
            AccentColor::Gold => write!(f, "Gold"),
            AccentColor::Red => write!(f, "Red"),
            AccentColor::Cyan => write!(f, "Cyan"),
        }
    }
}

impl AccentColor {
    pub const ALL: [AccentColor; 6] = [
        AccentColor::Blue,
        AccentColor::Purple,
        AccentColor::Green,
        AccentColor::Gold,
        AccentColor::Red,
        AccentColor::Cyan,
    ];

    pub fn all() -> &'static [AccentColor] {
        &Self::ALL
    }

    pub fn color(&self) -> iced::Color {
        match self {
            AccentColor::Blue => iced::Color::from_rgb(0.6, 0.8, 1.0),
            AccentColor::Purple => iced::Color::from_rgb(0.7, 0.5, 0.9),
            AccentColor::Green => iced::Color::from_rgb(0.4, 0.9, 0.5),
            AccentColor::Gold => iced::Color::from_rgb(1.0, 0.85, 0.3),
            AccentColor::Red => iced::Color::from_rgb(1.0, 0.45, 0.45),
            AccentColor::Cyan => iced::Color::from_rgb(0.3, 0.9, 0.9),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiScale {
    Small,
    Normal,
    Large,
}

impl fmt::Display for UiScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiScale::Small => write!(f, "Small"),
            UiScale::Normal => write!(f, "Normal"),
            UiScale::Large => write!(f, "Large"),
        }
    }
}

impl UiScale {
    pub const ALL: [UiScale; 3] = [UiScale::Small, UiScale::Normal, UiScale::Large];

    pub fn all() -> &'static [UiScale] {
        &Self::ALL
    }

    pub fn factor(&self) -> f32 {
        match self {
            UiScale::Small => 0.85,
            UiScale::Normal => 1.0,
            UiScale::Large => 1.2,
        }
    }
}

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

/// Current settings schema version.
pub const SETTINGS_VERSION: u32 = 1;

fn default_settings_version() -> u32 {
    SETTINGS_VERSION
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Schema version for forward-compatible deserialization.
    #[serde(default = "default_settings_version")]
    pub version: u32,
    // General
    pub default_status: GameStatus,
    pub default_platform: Platform,
    pub confirm_before_delete: bool,
    pub start_screen: StartScreen,
    #[serde(default = "default_true")]
    pub notifications_enabled: bool,
    #[serde(default)]
    pub toast_duration: ToastDuration,
    #[serde(default)]
    pub date_format: DateFormat,
    #[serde(default)]
    pub default_sort_order: SortOrder,
    #[serde(default = "default_true")]
    pub remember_filters: bool,
    #[serde(default)]
    pub last_filter_status: Option<GameStatus>,
    #[serde(default)]
    pub last_filter_platform: Option<Platform>,
    #[serde(default)]
    pub show_game_descriptions: bool,
    #[serde(default)]
    pub achievements_display: AchievementsDisplay,
    #[serde(default = "default_true")]
    pub download_covers_auto: bool,
    #[serde(default)]
    pub favorites: Vec<i64>,
    // Appearances
    pub theme_mode: ThemeMode,
    pub accent_color: AccentColor,
    pub compact_list: bool,
    pub show_covers_in_list: bool,
    pub ui_scale: UiScale,
    pub sidebar_width: u16,
    // Accessibility
    pub high_contrast: bool,
    pub show_status_labels: bool,
    pub large_click_targets: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
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
            theme_mode: ThemeMode::Dark,
            accent_color: AccentColor::Blue,
            compact_list: false,
            show_covers_in_list: true,
            ui_scale: UiScale::Normal,
            sidebar_width: 220,
            high_contrast: false,
            show_status_labels: true,
            large_click_targets: false,
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
