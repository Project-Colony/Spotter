use std::collections::{BTreeSet, HashSet, VecDeque};

use iced::{Element, Task, Theme};

use crate::db;
use crate::messages::{AuthMessage, SettingsMessage};
use crate::models::*;
use crate::views;

/// Type alias to reduce complexity in DataLoaded / load_data signatures.
pub type LoadResult = (Vec<Game>, UserProfile, Vec<(String, u32)>, Settings);

/// GOG import result: games + optionally refreshed (access_token, refresh_token).
pub type GogImportResult = (Vec<Game>, Option<(String, String)>);

/// Epic import result: games + optionally refreshed tokens.
pub type EpicImportResult = (Vec<Game>, Option<crate::epic::auth::EpicLoginResult>);

/// Xbox import result: games + optionally discovered XUID.
pub type XboxImportResult = (Vec<Game>, Option<String>);

/// Key for loading achievements from the DB (Steam uses appid, others use string ID).
enum AchievementKey {
    Steam(u32),
    Platform(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Library,
    GameDetail(i64),
    Statistics,
    Profile,
    Import,
    AddGame,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavigateTo(Screen),
    // Search & Filter
    SearchChanged(String),
    FilterStatus(Option<GameStatus>),
    FilterPlatform(Option<Platform>),
    SetSortOrder(SortOrder),
    // Game actions (by game ID)
    SetGameStatus(i64, GameStatus),
    SetGameRating(i64, Option<u8>),
    SetGameNotes(i64, String),
    // Delete with confirmation
    ConfirmDeleteGame(i64),
    CancelDelete,
    DeleteGame(i64),
    ConfirmBulkDelete,
    // Profile
    UpdateUsername(String),
    UpdateSteamApiKey(String),
    UpdateXboxApiKey(String),
    UpdateXboxGamertag(String),
    UpdatePsnNpsso(String),
    SaveProfile,
    ProfileSaved(Result<(), String>),
    ProfileAutoSaved(Result<(), String>),
    // Import
    ImportSteam,
    ImportGog,
    ImportEpic,
    ImportXbox,
    ImportPlayStation,
    ImportJson,
    SteamImportComplete(Result<Vec<Game>, String>),
    GogImportComplete(Result<GogImportResult, String>),
    EpicImportComplete(Result<EpicImportResult, String>),
    XboxImportComplete(Result<XboxImportResult, String>),
    PlayStationImportComplete(Result<Vec<Game>, String>),
    ImportJsonComplete(Result<Vec<Game>, String>),
    // Auth (delegated to handlers/auth.rs)
    Auth(AuthMessage),
    // Data persistence
    DataLoaded(Box<Result<LoadResult, String>>),
    DataSaved(Result<(), String>),
    /// Newly-persisted games: carries (title, platform_str, db_id) for games
    /// that had no ID before save, so we can patch in-memory copies.
    GameIdsMapped(Result<Vec<(String, String, i64)>, String>),
    // Images
    CoversDownloaded(Result<Vec<(usize, String)>, String>),
    DownloadCovers,
    // Export
    ExportData,
    ExportCsv,
    ExportComplete(Result<String, String>),
    // Achievements
    AchievementsLoaded(Result<Vec<Achievement>, String>),
    AchievementIconsDownloaded(()),
    // Add Game form
    AddGameTitleChanged(String),
    AddGameGenreChanged(String),
    AddGamePlatformChanged(Platform),
    AddGameStatusChanged(GameStatus),
    SaveNewGame,
    // Settings (delegated to handlers/settings.rs)
    Settings(SettingsMessage),
    // Undo deletion
    UndoDelete,
    PermanentDelete(Vec<i64>),
    // Favorites
    ToggleFavorite(i64),
    // Bulk operations
    ToggleBulkMode,
    ToggleBulkSelect(i64),
    BulkSetStatus(GameStatus),
    BulkDelete,
    BulkSelectAll,
    BulkDeselectAll,
    // Onboarding
    DismissOnboarding,
    // Pagination
    ShowMoreCards,
    // Open external URL
    OpenUrl(String),
    // Clear all filters
    ClearAllFilters,
    // Search focus
    FocusSearch,
    // Feedback
    ToastTick(u32),
    ProfileSaveShown,
    // Cascade import
    DrainImportQueue,
    // No-op (used by keyboard subscription for unhandled events)
    NoOp,
}

impl From<SettingsMessage> for Message {
    fn from(msg: SettingsMessage) -> Self {
        Message::Settings(msg)
    }
}

impl From<AuthMessage> for Message {
    fn from(msg: AuthMessage) -> Self {
        Message::Auth(msg)
    }
}

pub struct Spotter {
    pub screen: Screen,
    pub games: Vec<Game>,
    pub search_query: String,
    pub filter_status: Option<GameStatus>,
    pub filter_platform: Option<Platform>,
    pub sort_order: SortOrder,
    pub profile: UserProfile,
    pub playtime_data: Vec<(String, u32)>,
    /// Platforms currently importing (e.g. "Steam", "GOG"). Multiple can run in parallel.
    pub importing: BTreeSet<String>,
    pub import_status: String,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub data_loaded: bool,
    pub steam_login_active: bool,
    pub gog_login_active: bool,
    pub gog_code_input: String,
    pub epic_login_active: bool,
    pub epic_code_input: String,
    /// Achievements for the currently viewed game (loaded on demand).
    pub achievements: Vec<Achievement>,
    /// True while achievements are being loaded from DB.
    pub achievements_loading: bool,
    /// Counter to track which toast is active (for auto-dismiss).
    pub toast_id: u32,
    /// Game ID pending deletion (confirmation required).
    pub confirm_delete: Option<i64>,
    /// Bulk delete pending confirmation.
    pub confirm_bulk_delete: bool,
    // Add-game form fields
    pub new_game_title: String,
    pub new_game_genre: String,
    pub new_game_platform: Platform,
    pub new_game_status: GameStatus,
    // Settings
    pub settings: Settings,
    pub settings_section: SettingsSection,
    // Undo delete (supports single + bulk)
    pub recently_deleted: Vec<(Game, i64)>,
    // Profile save feedback
    pub profile_just_saved: bool,
    // Settings save feedback
    pub settings_just_saved: bool,
    // First launch onboarding
    pub first_launch: bool,
    // Bulk operations
    pub bulk_mode: bool,
    pub bulk_selected: HashSet<i64>,
    // Import history log
    pub import_history: VecDeque<String>,
    // Cascade import queue (games appear one-by-one)
    pub import_queue: Vec<Game>,
    pub import_queue_total: usize,
    // Cache of game titles that have a cover on disk (avoids stat() per render)
    pub cover_cache: HashSet<String>,
    // Cached filtered+sorted game indices (avoids O(n log n) per render)
    pub filtered_cache: Vec<usize>,
    pub filter_generation: u64,
    filter_cache_generation: u64,
    // Visible card limit for pagination (avoids building 500+ widgets)
    pub visible_card_limit: usize,
}

const CARD_PAGE_SIZE: usize = 200;

impl Default for Spotter {
    fn default() -> Self {
        Self {
            screen: Screen::Library,
            games: Vec::new(),
            search_query: String::new(),
            filter_status: None,
            filter_platform: None,
            sort_order: SortOrder::TitleAsc,
            profile: UserProfile::default(),
            playtime_data: Vec::new(),
            importing: BTreeSet::new(),
            import_status: String::new(),
            error_message: None,
            success_message: None,
            data_loaded: false,
            steam_login_active: false,
            gog_login_active: false,
            gog_code_input: String::new(),
            epic_login_active: false,
            epic_code_input: String::new(),
            achievements: Vec::new(),
            achievements_loading: false,
            toast_id: 0,
            confirm_delete: None,
            confirm_bulk_delete: false,
            new_game_title: String::new(),
            new_game_genre: String::new(),
            new_game_platform: Platform::Steam,
            new_game_status: GameStatus::Unplayed,
            settings: Settings::default(),
            settings_section: SettingsSection::General,
            recently_deleted: Vec::new(),
            profile_just_saved: false,
            settings_just_saved: false,
            first_launch: false,
            bulk_mode: false,
            bulk_selected: HashSet::new(),
            import_history: VecDeque::with_capacity(50),
            import_queue: Vec::new(),
            import_queue_total: 0,
            cover_cache: HashSet::new(),
            filtered_cache: Vec::new(),
            filter_generation: 0,
            filter_cache_generation: u64::MAX, // force initial compute
            visible_card_limit: CARD_PAGE_SIZE,
        }
    }
}

impl Spotter {
    pub(crate) fn find_game_index(&self, id: i64) -> Option<usize> {
        self.games.iter().position(|g| g.id == Some(id))
    }

    /// Mark the filtered game list as stale so it will be recomputed before next view.
    pub(crate) fn invalidate_filter_cache(&mut self) {
        self.filter_generation = self.filter_generation.wrapping_add(1);
        self.visible_card_limit = CARD_PAGE_SIZE;
    }

    /// Recompute the filtered+sorted game indices if stale.
    pub(crate) fn ensure_filter_cache(&mut self) {
        if self.filter_cache_generation == self.filter_generation {
            return;
        }
        self.filter_cache_generation = self.filter_generation;
        let filtered = self.filtered_games();
        self.filtered_cache = filtered.into_iter().map(|(i, _)| i).collect();
    }

    /// Rebuild the cover cache by scanning games that have a cover file on disk.
    pub(crate) fn refresh_cover_cache(&mut self) {
        self.cover_cache.clear();
        for g in &self.games {
            if crate::images::cover_path(&g.title).exists() {
                self.cover_cache.insert(g.title.clone());
            }
        }
        for g in &self.import_queue {
            if crate::images::cover_path(&g.title).exists() {
                self.cover_cache.insert(g.title.clone());
            }
        }
    }

    pub(crate) fn show_success(&mut self, msg: impl Into<String>) {
        if !self.settings.notifications_enabled {
            return;
        }
        self.success_message = Some(msg.into());
        self.error_message = None;
        self.toast_id += 1;
    }

    pub(crate) fn show_error(&mut self, msg: impl Into<String>) {
        // Errors always show regardless of notifications setting
        self.error_message = Some(msg.into());
        self.success_message = None;
        self.toast_id += 1;
    }

    fn toast_dismiss_timer(id: u32, duration_ms: u64) -> Task<Message> {
        delay_task(duration_ms, move || Message::ToastTick(id))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let old_toast_id = self.toast_id;
        let old_gen = self.filter_generation;
        let task = self.process_message(message);
        // Rebuild filter cache if it was invalidated during this update
        if self.filter_generation != old_gen {
            self.ensure_filter_cache();
        }
        if self.toast_id != old_toast_id {
            let duration = self.settings.toast_duration.millis();
            Task::batch([task, Self::toast_dismiss_timer(self.toast_id, duration)])
        } else {
            task
        }
    }

    fn process_message(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(screen) => {
                self.confirm_delete = None;
                if let Screen::GameDetail(id) = &screen {
                    if let Some(idx) = self.find_game_index(*id) {
                        self.achievements.clear();
                        // Load achievements: Steam by appid, Xbox by xbox_id
                        let load_key: Option<AchievementKey> =
                            if let Some(appid) = self.games[idx].steam_appid {
                                Some(AchievementKey::Steam(appid))
                            } else {
                                self.games[idx]
                                    .xbox_id
                                    .as_ref()
                                    .map(|xbox_id| AchievementKey::Platform(xbox_id.clone()))
                            };
                        if let Some(key) = load_key {
                            self.achievements_loading = true;
                            self.screen = screen;
                            return Task::perform(
                                async move {
                                    (match db::open() {
                                        Ok(conn) => match key {
                                            AchievementKey::Steam(appid) => {
                                                db::load_achievements(&conn, appid)
                                            }
                                            AchievementKey::Platform(id) => {
                                                db::load_achievements_by_platform(&conn, &id)
                                            }
                                        },
                                        Err(e) => Err(e),
                                    })
                                    .map_err(Into::into)
                                },
                                Message::AchievementsLoaded,
                            );
                        }
                    }
                }
                if let Screen::AddGame = &screen {
                    self.new_game_title.clear();
                    self.new_game_genre.clear();
                    self.new_game_platform = self.settings.default_platform;
                    self.new_game_status = self.settings.default_status;
                }
                self.screen = screen;
            }
            Message::SearchChanged(query) => {
                self.search_query = query;
                self.invalidate_filter_cache();
            }
            Message::OpenUrl(url) => {
                if let Err(e) = crate::api_client::open_browser(&url) {
                    self.show_error(format!("Failed to open URL: {}", e));
                }
            }
            Message::ClearAllFilters => {
                self.search_query.clear();
                self.filter_status = None;
                self.filter_platform = None;
                self.invalidate_filter_cache();
            }
            Message::FilterStatus(status) => {
                self.filter_status = status;
                self.invalidate_filter_cache();
                if self.settings.remember_filters {
                    self.settings.last_filter_status = status;
                    return self.persist_settings();
                }
            }
            Message::FilterPlatform(platform) => {
                self.filter_platform = platform;
                self.invalidate_filter_cache();
                if self.settings.remember_filters {
                    self.settings.last_filter_platform = platform;
                    return self.persist_settings();
                }
            }
            Message::SetSortOrder(order) => {
                self.sort_order = order;
                self.settings.default_sort_order = order;
                self.invalidate_filter_cache();
                return self.persist_settings();
            }
            Message::SetGameStatus(id, status) => {
                if let Some(idx) = self.find_game_index(id) {
                    self.games[idx].status = status;
                    self.games[idx].last_played =
                        chrono::Local::now().format("%Y-%m-%d").to_string();
                    self.invalidate_filter_cache();
                    return self.persist_single_game(id);
                }
            }
            Message::SetGameRating(id, rating) => {
                if let Some(idx) = self.find_game_index(id) {
                    self.games[idx].rating = rating.map(|r| r.min(10));
                    self.invalidate_filter_cache();
                    return self.persist_single_game(id);
                }
            }
            Message::SetGameNotes(id, notes) => {
                if let Some(idx) = self.find_game_index(id) {
                    self.games[idx].notes = truncate_input(notes, 2000);
                    return self.persist_single_game(id);
                }
            }
            // Delete with confirmation
            Message::ConfirmDeleteGame(id) => {
                if self.settings.confirm_before_delete {
                    self.confirm_delete = Some(id);
                } else {
                    return self.process_message(Message::DeleteGame(id));
                }
            }
            Message::CancelDelete => {
                if self.confirm_delete.is_some() || self.confirm_bulk_delete {
                    self.confirm_delete = None;
                    self.confirm_bulk_delete = false;
                } else {
                    self.screen = Screen::Library;
                }
            }
            Message::DeleteGame(id) => {
                self.confirm_delete = None;
                if let Some(idx) = self.find_game_index(id) {
                    let game = self.games.remove(idx);
                    self.recently_deleted.push((game, id));
                    self.screen = Screen::Library;
                    self.invalidate_filter_cache();
                    self.show_success("Game deleted — click Undo to restore");
                    let ids = vec![id];
                    return delay_task(8000, move || Message::PermanentDelete(ids));
                }
            }
            Message::UndoDelete => {
                if !self.recently_deleted.is_empty() {
                    let restored: Vec<(Game, i64)> = std::mem::take(&mut self.recently_deleted);
                    let count = restored.len();
                    for (game, _id) in restored {
                        self.games.push(game);
                    }
                    self.invalidate_filter_cache();
                    if count == 1 {
                        self.show_success("Game restored");
                    } else {
                        self.show_success(format!("{} games restored", count));
                    }
                    return self.persist_games();
                }
            }
            Message::PermanentDelete(ids) => {
                let ids_to_delete: Vec<i64> = ids
                    .into_iter()
                    .filter(|id| self.recently_deleted.iter().any(|(_, did)| did == id))
                    .collect();
                if !ids_to_delete.is_empty() {
                    self.recently_deleted
                        .retain(|(_, did)| !ids_to_delete.contains(did));
                    return spawn_task(
                        move || {
                            let conn = db::open()?;
                            for del_id in ids_to_delete {
                                db::delete_game(&conn, del_id)?;
                            }
                            Ok(())
                        },
                        Message::DataSaved,
                    );
                }
            }
            // Profile
            Message::UpdateUsername(name) => {
                self.profile.username = truncate_input(name, 50);
            }
            Message::UpdateSteamApiKey(key) => {
                self.profile.steam_api_key = key.trim().to_string();
                return self.persist_profile();
            }
            Message::UpdateXboxApiKey(key) => {
                self.profile.xbox_api_key = key.trim().to_string();
                return self.persist_profile();
            }
            Message::UpdateXboxGamertag(tag) => {
                self.profile.xbox_gamertag = tag.trim().to_string();
                return self.persist_profile();
            }
            Message::UpdatePsnNpsso(token) => {
                self.profile.psn_npsso = token.trim().to_string();
                return self.persist_profile();
            }
            Message::SaveProfile => {
                return self.save_profile_task();
            }
            Message::ProfileSaved(result) => match result {
                Ok(()) => {
                    self.profile_just_saved = true;
                    self.show_success("Profile saved successfully!");
                    return delay_task(3000, || Message::ProfileSaveShown);
                }
                Err(e) => {
                    self.show_error(format!("Profile save error: {}", e));
                }
            },
            Message::ProfileAutoSaved(result) => {
                if let Err(e) = result {
                    self.show_error(format!("Profile save error: {}", e));
                }
            }
            // ── Auth handlers (delegated to handlers/auth.rs) ──
            Message::Auth(auth_msg) => {
                return self.handle_auth(auth_msg);
            }
            // ── Import handlers (delegated to handlers/import.rs) ──
            Message::ImportSteam => return self.handle_import_steam(),
            Message::ImportGog => return self.handle_import_gog(),
            Message::ImportEpic => return self.handle_import_epic(),
            Message::ImportXbox => return self.handle_import_xbox(),
            Message::ImportPlayStation => return self.handle_import_playstation(),
            Message::ImportJson => return self.handle_import_json(),
            Message::SteamImportComplete(result) => {
                return self.handle_import_result("Steam", result);
            }
            Message::GogImportComplete(result) => {
                return self.handle_gog_import_complete(result);
            }
            Message::EpicImportComplete(result) => {
                return self.handle_epic_import_complete(result);
            }
            Message::XboxImportComplete(result) => {
                return self.handle_xbox_import_complete(result);
            }
            Message::PlayStationImportComplete(result) => {
                return self.handle_import_result("PlayStation", result);
            }
            Message::ImportJsonComplete(result) => {
                return self.handle_import_result("JSON", result);
            }
            // ── Data handlers (delegated to handlers/data.rs) ──
            Message::DataLoaded(result) => {
                return self.handle_data_loaded(result);
            }
            Message::DataSaved(result) => {
                if let Err(e) = result {
                    self.show_error(format!("Save error: {}", e));
                }
            }
            Message::GameIdsMapped(result) => {
                match result {
                    Ok(id_map) => {
                        for (title, plat_str, db_id) in &id_map {
                            // Patch games in the main list
                            for g in &mut self.games {
                                if g.id.is_none()
                                    && g.title == *title
                                    && g.platform.to_string() == *plat_str
                                {
                                    g.id = Some(*db_id);
                                    break;
                                }
                            }
                            // Patch games still in the cascade queue
                            for g in self.import_queue.iter_mut() {
                                if g.id.is_none()
                                    && g.title == *title
                                    && g.platform.to_string() == *plat_str
                                {
                                    g.id = Some(*db_id);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        self.show_error(format!("Save error: {}", e));
                    }
                }
            }
            Message::DownloadCovers => {
                return self.handle_download_covers();
            }
            Message::CoversDownloaded(_result) => {
                self.refresh_cover_cache();
            }
            Message::ExportData => return self.handle_export_json(),
            Message::ExportCsv => return self.handle_export_csv(),
            Message::ExportComplete(result) => match result {
                Ok(path) => {
                    self.show_success(format!("Exported to {}", path));
                }
                Err(e) => {
                    self.show_error(format!("Export error: {}", e));
                }
            },
            Message::AchievementsLoaded(result) => {
                return self.handle_achievements_loaded(result);
            }
            Message::AchievementIconsDownloaded(_) => {}
            // Add Game form
            Message::AddGameTitleChanged(title) => {
                self.new_game_title = truncate_input(title, 200);
            }
            Message::AddGameGenreChanged(genre) => {
                self.new_game_genre = truncate_input(genre, 100);
            }
            Message::AddGamePlatformChanged(platform) => {
                self.new_game_platform = platform;
            }
            Message::AddGameStatusChanged(status) => {
                self.new_game_status = status;
            }
            Message::SaveNewGame => {
                let title = self.new_game_title.trim().to_string();
                if title.is_empty() {
                    self.show_error("Game title is required");
                    return Task::none();
                }
                let title_lower = title.to_lowercase();
                if self
                    .games
                    .iter()
                    .any(|g| g.title.to_lowercase() == title_lower)
                {
                    self.show_error(format!("'{}' already exists in your library", title));
                    return Task::none();
                }
                let game = Game {
                    id: None,
                    title,
                    platform: self.new_game_platform,
                    playtime_minutes: 0,
                    achievements_unlocked: 0,
                    achievements_total: 0,
                    status: self.new_game_status,
                    rating: None,
                    genre: self.new_game_genre.trim().to_string(),
                    last_played: String::new(),
                    cover_url: String::new(),
                    steam_appid: None,
                    gog_id: None,
                    epic_id: None,
                    xbox_id: None,
                    psn_id: None,
                    notes: String::new(),
                    description: String::new(),
                    release_date: String::new(),
                    review_percent: None,
                    tags: String::new(),
                };
                self.games.push(game);
                self.invalidate_filter_cache();
                self.show_success(format!("'{}' added to library", self.new_game_title.trim()));
                self.new_game_title.clear();
                self.new_game_genre.clear();
                self.new_game_platform = self.settings.default_platform;
                self.new_game_status = self.settings.default_status;
                self.screen = Screen::Library;
                return self.persist_games();
            }
            // Settings (delegated to handlers/settings.rs)
            Message::Settings(settings_msg) => {
                return self.handle_settings(settings_msg);
            }
            Message::ToggleFavorite(id) => {
                if let Some(pos) = self.settings.favorites.iter().position(|&fid| fid == id) {
                    self.settings.favorites.remove(pos);
                } else {
                    self.settings.favorites.push(id);
                }
                self.invalidate_filter_cache();
                return self.persist_settings();
            }
            Message::ToggleBulkMode => {
                self.bulk_mode = !self.bulk_mode;
                if !self.bulk_mode {
                    self.bulk_selected.clear();
                }
            }
            Message::ToggleBulkSelect(id) => {
                if self.bulk_selected.contains(&id) {
                    self.bulk_selected.remove(&id);
                } else {
                    self.bulk_selected.insert(id);
                }
            }
            Message::BulkSetStatus(status) => {
                let ids: Vec<i64> = std::mem::take(&mut self.bulk_selected)
                    .into_iter()
                    .collect();
                for &id in &ids {
                    if let Some(idx) = self.find_game_index(id) {
                        self.games[idx].status = status;
                    }
                }
                let count = ids.len();
                self.bulk_mode = false;
                self.invalidate_filter_cache();
                self.show_success(format!("{} games updated to {}", count, status));
                return self.persist_games();
            }
            Message::BulkDelete => {
                if self.settings.confirm_before_delete && !self.confirm_bulk_delete {
                    self.confirm_bulk_delete = true;
                    return Task::none();
                }
                self.confirm_bulk_delete = false;
                let ids = std::mem::take(&mut self.bulk_selected);
                let count = ids.len();
                let mut to_delete: Vec<usize> = self
                    .games
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| g.id.is_some_and(|id| ids.contains(&id)))
                    .map(|(idx, _)| idx)
                    .collect();
                to_delete.sort_unstable();
                let mut deleted = Vec::with_capacity(to_delete.len());
                for &idx in to_delete.iter().rev() {
                    let game = self.games.remove(idx);
                    let id = game.id.unwrap();
                    deleted.push((game, id));
                }
                let delete_ids: Vec<i64> = deleted.iter().map(|(_, id)| *id).collect();
                self.recently_deleted.extend(deleted);
                self.bulk_mode = false;
                self.invalidate_filter_cache();
                self.screen = Screen::Library;
                self.show_success(format!("{} games deleted — click Undo to restore", count));
                return delay_task(8000, move || Message::PermanentDelete(delete_ids));
            }
            Message::ConfirmBulkDelete => {
                return self.process_message(Message::BulkDelete);
            }
            Message::BulkSelectAll => {
                self.bulk_mode = true;
                self.ensure_filter_cache();
                let ids: Vec<i64> = self
                    .filtered_cache
                    .iter()
                    .filter_map(|&i| self.games.get(i).and_then(|g| g.id))
                    .collect();
                for id in ids {
                    self.bulk_selected.insert(id);
                }
            }
            Message::BulkDeselectAll => {
                self.bulk_selected.clear();
            }
            Message::DismissOnboarding => {
                self.first_launch = false;
            }
            Message::ShowMoreCards => {
                self.visible_card_limit += CARD_PAGE_SIZE;
            }
            Message::FocusSearch => {
                self.screen = Screen::Library;
                return iced::widget::operation::focus(iced::widget::Id::new("library_search"));
            }
            Message::ProfileSaveShown => {
                self.profile_just_saved = false;
            }
            // Feedback
            Message::ToastTick(id) => {
                if self.toast_id == id {
                    self.success_message = None;
                    self.error_message = None;
                }
            }
            // Cascade import
            Message::DrainImportQueue => {
                return self.handle_drain_import_queue();
            }
            Message::NoOp => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        use iced::widget::{button, column, container, row, stack, text};
        use iced::{Alignment, Background, Border, Color, Length, Padding};

        let sidebar = views::sidebar::view(self);

        // Loading indicator before data is ready
        let content: Element<'_, Message> = if !self.data_loaded {
            let vt = crate::theme::ViewTheme::from_settings(&self.settings);
            container(
                column![
                    text("Loading your library...")
                        .size(18)
                        .color(vt.text_light),
                    text("Please wait").size(12).color(vt.text_muted),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else {
            match &self.screen {
                Screen::Library => views::library::view(self),
                Screen::GameDetail(id) => views::detail::view(self, *id),
                Screen::Statistics => views::stats::view(self),
                Screen::Profile => views::profile::view(self),
                Screen::Import => views::import::view(self),
                Screen::AddGame => views::add_game::view(self),
                Screen::Settings => views::settings::view(self),
            }
        };

        let main_layout: Element<'_, Message> = iced::widget::row![sidebar, content].into();

        let toast_msg = self
            .success_message
            .as_ref()
            .or(self.error_message.as_ref());
        let has_modal = self.confirm_delete.is_some() || self.confirm_bulk_delete;
        let has_toast = toast_msg.is_some();

        if !has_modal && !has_toast {
            return main_layout;
        }

        let mut layers: Vec<Element<'_, Message>> = vec![main_layout];

        // Delete confirmation modal overlay
        if let Some(delete_id) = self.confirm_delete {
            let game_title = self
                .games
                .iter()
                .find(|g| g.id == Some(delete_id))
                .map(|g| g.title.clone())
                .unwrap_or_else(|| "this game".to_string());

            let backdrop = button(
                container(text("").size(1))
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_press(Message::CancelDelete)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_: &Theme, _| button::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                ..button::Style::default()
            });
            layers.push(backdrop.into());

            let modal_bg = self.settings.theme_mode.bg_card();
            let modal_card = container(
                column![
                    text(format!("{} Delete Game", crate::theme::icons::DELETE))
                        .size(20)
                        .font(crate::theme::FONT_BOLD)
                        .color(crate::theme::DANGER),
                    text(format!(
                        "Are you sure you want to delete \"{}\"?",
                        game_title
                    ))
                    .size(14)
                    .color(crate::theme::TEXT_LIGHT),
                    text("You'll have 8 seconds to undo after deletion.")
                        .size(12)
                        .color(crate::theme::TEXT_MUTED),
                    row![
                        button(text("Delete").size(14).color(Color::WHITE))
                            .on_press(Message::DeleteGame(delete_id))
                            .padding([10, 24])
                            .style(|_: &Theme, status| {
                                let hover = matches!(
                                    status,
                                    button::Status::Hovered | button::Status::Pressed
                                );
                                let bg = if hover {
                                    Color::from_rgb(0.7, 0.12, 0.12)
                                } else {
                                    Color::from_rgb(0.6, 0.1, 0.1)
                                };
                                button::Style {
                                    background: Some(Background::Color(bg)),
                                    text_color: Color::WHITE,
                                    border: Border {
                                        radius: 8.0.into(),
                                        color: Color::from_rgb(0.8, 0.2, 0.2),
                                        width: 1.0,
                                    },
                                    ..button::Style::default()
                                }
                            }),
                        button(text("Cancel").size(14).color(crate::theme::TEXT_SECONDARY))
                            .on_press(Message::CancelDelete)
                            .padding([10, 24])
                            .style(crate::theme::outline_btn_style(
                                crate::theme::TEXT_SECONDARY,
                                modal_bg,
                            )),
                    ]
                    .spacing(12),
                ]
                .spacing(12),
            )
            .padding(24)
            .max_width(450)
            .style(move |_: &Theme| container::Style {
                background: Some(Background::Color(modal_bg)),
                border: Border {
                    radius: 12.0.into(),
                    color: Color::from_rgb(0.6, 0.15, 0.15),
                    width: 2.0,
                },
                ..container::Style::default()
            });

            let modal_positioned = container(modal_card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill);
            layers.push(modal_positioned.into());
        } else if self.confirm_bulk_delete {
            let count = self.bulk_selected.len();

            let backdrop = button(
                container(text("").size(1))
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_press(Message::CancelDelete)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_: &Theme, _| button::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                ..button::Style::default()
            });
            layers.push(backdrop.into());

            let modal_bg = self.settings.theme_mode.bg_card();
            let modal_card = container(
                column![
                    text(format!("{} Bulk Delete", crate::theme::icons::DELETE))
                        .size(20)
                        .font(crate::theme::FONT_BOLD)
                        .color(crate::theme::DANGER),
                    text(format!("Are you sure you want to delete {} games?", count))
                        .size(14)
                        .color(crate::theme::TEXT_LIGHT),
                    text("You'll have 8 seconds to undo after deletion.")
                        .size(12)
                        .color(crate::theme::TEXT_MUTED),
                    row![
                        button(
                            text(format!("Delete {}", count))
                                .size(14)
                                .color(Color::WHITE)
                        )
                        .on_press(Message::ConfirmBulkDelete)
                        .padding([10, 24])
                        .style(|_: &Theme, status| {
                            let hover =
                                matches!(status, button::Status::Hovered | button::Status::Pressed);
                            let bg = if hover {
                                Color::from_rgb(0.7, 0.12, 0.12)
                            } else {
                                Color::from_rgb(0.6, 0.1, 0.1)
                            };
                            button::Style {
                                background: Some(Background::Color(bg)),
                                text_color: Color::WHITE,
                                border: Border {
                                    radius: 8.0.into(),
                                    color: Color::from_rgb(0.8, 0.2, 0.2),
                                    width: 1.0,
                                },
                                ..button::Style::default()
                            }
                        }),
                        button(text("Cancel").size(14).color(crate::theme::TEXT_SECONDARY))
                            .on_press(Message::CancelDelete)
                            .padding([10, 24])
                            .style(crate::theme::outline_btn_style(
                                crate::theme::TEXT_SECONDARY,
                                modal_bg,
                            )),
                    ]
                    .spacing(12),
                ]
                .spacing(12),
            )
            .padding(24)
            .max_width(450)
            .style(move |_: &Theme| container::Style {
                background: Some(Background::Color(modal_bg)),
                border: Border {
                    radius: 12.0.into(),
                    color: Color::from_rgb(0.6, 0.15, 0.15),
                    width: 2.0,
                },
                ..container::Style::default()
            });

            let modal_positioned = container(modal_card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill);
            layers.push(modal_positioned.into());
        }

        // Toast notification
        if let Some(msg) = toast_msg {
            let is_error = self.error_message.is_some() && self.success_message.is_none();
            let accent = if is_error {
                crate::theme::DANGER
            } else {
                crate::theme::SUCCESS
            };

            let has_undo = !self.recently_deleted.is_empty();
            let toast_bg = self.settings.theme_mode.bg_card();

            let toast_content: Element<'_, Message> = if has_undo && !is_error {
                let undo_btn = button(text("Undo").size(13).color(crate::theme::ACCENT_BLUE))
                    .on_press(Message::UndoDelete)
                    .padding([4, 12])
                    .style(crate::theme::outline_btn_style(
                        crate::theme::ACCENT_BLUE,
                        toast_bg,
                    ));
                row![text(msg.clone()).size(13).color(accent), undo_btn,]
                    .spacing(12)
                    .align_y(iced::Alignment::Center)
                    .into()
            } else {
                text(msg.clone()).size(13).color(accent).into()
            };

            let toast = container(toast_content)
                .padding([10, 16])
                .style(move |_: &Theme| container::Style {
                    background: Some(Background::Color(toast_bg)),
                    border: Border {
                        radius: 10.0.into(),
                        color: Color {
                            r: accent.r * 0.4,
                            g: accent.g * 0.4,
                            b: accent.b * 0.4,
                            a: 1.0,
                        },
                        width: 1.0,
                    },
                    ..container::Style::default()
                });

            let sidebar_w = self.settings.sidebar_width.clamp(160, 400) as f32 + 20.0;
            let toast_positioned = container(toast)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_bottom(Length::Fill)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 20.0,
                    left: sidebar_w,
                });
            layers.push(toast_positioned.into());
        }

        stack(layers).into()
    }

    pub fn theme(&self) -> Theme {
        match self.settings.theme_mode {
            ThemeMode::Dark => Theme::Dark,
            ThemeMode::Darker => Theme::Moonfly,
            ThemeMode::Midnight => Theme::Oxocarbon,
        }
    }

    pub fn filtered_games(&self) -> Vec<(usize, &Game)> {
        let query = self.search_query.to_lowercase();

        let mut games: Vec<(usize, &Game)> = self
            .games
            .iter()
            .enumerate()
            .filter(|(_, game)| {
                let matches_status = self.filter_status.is_none_or(|s| game.status == s);
                if !matches_status {
                    return false;
                }
                let matches_platform = self.filter_platform.is_none_or(|p| game.platform == p);
                if !matches_platform {
                    return false;
                }
                query.is_empty()
                    || contains_ci(&game.title, &query)
                    || contains_ci(&game.genre, &query)
                    || contains_ci(&game.tags, &query)
                    || contains_ci(&game.notes, &query)
                    || contains_ci(&game.description, &query)
            })
            .collect();

        match self.sort_order {
            SortOrder::TitleAsc => {
                games.sort_by_cached_key(|(_, g)| g.title.to_lowercase());
            }
            SortOrder::TitleDesc => {
                games.sort_by_cached_key(|(_, g)| std::cmp::Reverse(g.title.to_lowercase()));
            }
            SortOrder::PlaytimeDesc => {
                games.sort_by(|a, b| b.1.playtime_minutes.cmp(&a.1.playtime_minutes));
            }
            SortOrder::RatingDesc => {
                games.sort_by(|a, b| b.1.rating.unwrap_or(0).cmp(&a.1.rating.unwrap_or(0)));
            }
            SortOrder::LastPlayedDesc => {
                games.sort_by(|a, b| b.1.last_played.cmp(&a.1.last_played));
            }
            SortOrder::FavoritesFirst => {
                games.sort_by_cached_key(|(_, g)| g.title.to_lowercase());
            }
        }

        if self.sort_order == SortOrder::FavoritesFirst && !self.settings.favorites.is_empty() {
            let favorites = &self.settings.favorites;
            games.sort_by(|a, b| {
                let a_fav = a.1.id.is_some_and(|id| favorites.contains(&id));
                let b_fav = b.1.id.is_some_and(|id| favorites.contains(&id));
                b_fav.cmp(&a_fav)
            });
        }

        games
    }

    /// Keyboard shortcut subscription.
    fn subscription(&self) -> iced::Subscription<Message> {
        iced::keyboard::listen().map(|event| {
            use iced::keyboard::{key::Named, Event, Key};
            match event {
                Event::KeyPressed { key, modifiers, .. } => match key.as_ref() {
                    Key::Named(Named::Escape) => Message::CancelDelete,
                    Key::Character("f") if modifiers.command() => Message::FocusSearch,
                    Key::Character("a") if modifiers.command() => Message::BulkSelectAll,
                    _ => Message::NoOp,
                },
                _ => Message::NoOp,
            }
        })
    }
}

// contains_ci lives in models.rs (imported via `use crate::models::*`).

/// Truncate a string input to a maximum character limit.
fn truncate_input(s: String, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s
    } else {
        s.chars().take(max_chars).collect()
    }
}

/// Run a blocking closure on a background thread, returning a Task.
pub(crate) fn spawn_task<T, F, M>(
    f: F,
    msg: impl FnOnce(Result<T, String>) -> M + Send + 'static,
) -> Task<M>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
    M: Send + 'static,
{
    Task::perform(
        async move {
            match std::thread::spawn(f).join() {
                Ok(result) => result,
                Err(_) => Err("Background task panicked".into()),
            }
        },
        msg,
    )
}

/// Schedule a Message to be sent after a delay (in milliseconds).
pub(crate) fn delay_task<M, F>(ms: u64, make_msg: F) -> Task<M>
where
    M: Send + 'static,
    F: FnOnce() -> M + Send + 'static,
{
    let cell = std::sync::Mutex::new(Some(make_msg));
    Task::perform(
        async move {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(ms));
                let _ = tx.send(());
            });
            let _ = rx.recv();
        },
        move |_| {
            let f = cell.lock().ok().and_then(|mut opt| opt.take());
            f.expect("delay_task callback called more than once")()
        },
    )
}

fn load_data() -> Result<LoadResult, String> {
    let conn = db::open()?;
    let games = db::load_games(&conn)?;
    let mut profile = db::load_profile(&conn)?;
    // Overlay credentials from the OS keyring (takes precedence over DB values).
    crate::keyring::load_profile_secrets(&mut profile);
    let playtime = db::get_daily_playtime(&conn, 30)?;
    let settings = db::load_settings(&conn)?;
    Ok((games, profile, playtime, settings))
}

pub fn run() -> iced::Result {
    iced::application(boot, Spotter::update, Spotter::view)
        .title("Spotter — Game Tracker")
        .theme(Spotter::theme)
        .subscription(Spotter::subscription)
        .font(include_bytes!("../assets/fonts/JetBrainsMonoNerdFont-Regular.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/JetBrainsMonoNerdFont-Bold.ttf").as_slice())
        .default_font(iced::Font::with_name("JetBrainsMono NF"))
        .scale_factor(|app: &Spotter| app.settings.ui_scale.factor())
        .run()
}

fn boot() -> (Spotter, Task<Message>) {
    let load_task = spawn_task(load_data, |r| Message::DataLoaded(Box::new(r)));
    (Spotter::default(), load_task)
}
