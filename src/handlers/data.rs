//! Data persistence, export, achievements, covers, and cascade import handlers.
//!
//! Extracted from `app.rs` to keep the main module focused on state and dispatch.

use std::collections::HashMap;

use iced::Task;

use crate::app::{Message, Spotter};
use crate::db;
use crate::models::*;

impl Spotter {
    // ── Data loading ──

    pub(crate) fn handle_data_loaded(
        &mut self,
        result: Box<Result<crate::app::LoadResult, String>>,
    ) -> Task<Message> {
        self.data_loaded = true;
        match *result {
            Ok((games, profile, playtime, settings)) => {
                if !games.is_empty() {
                    self.games = games;
                } else {
                    self.first_launch = true;
                }
                self.profile = profile;
                self.playtime_data = playtime;
                self.settings = settings;
                // Apply start screen setting
                self.screen = match self.settings.start_screen {
                    StartScreen::Statistics => crate::app::Screen::Statistics,
                    StartScreen::Import => crate::app::Screen::Import,
                    StartScreen::Profile => crate::app::Screen::Profile,
                    StartScreen::Library => crate::app::Screen::Library,
                };
                // Restore sort order from settings
                self.sort_order = self.settings.default_sort_order;
                // Restore filters if remember_filters is enabled
                if self.settings.remember_filters {
                    self.filter_status = self.settings.last_filter_status;
                    self.filter_platform = self.settings.last_filter_platform;
                }
                // Apply default platform/status from settings for add-game form
                self.new_game_platform = self.settings.default_platform;
                self.new_game_status = self.settings.default_status;
                // Build caches once at startup
                self.refresh_cover_cache();
                self.invalidate_filter_cache();
                self.ensure_filter_cache();

                // Proactively refresh tokens at startup
                let mut startup_tasks = Vec::new();

                if !self.profile.gog_refresh_token.is_empty() && !self.profile.gog_token.is_empty()
                {
                    let refresh = self.profile.gog_refresh_token.clone();
                    startup_tasks.push(crate::app::spawn_task(
                        move || crate::gog::auth::refresh_token(&refresh),
                        |r| Message::Auth(crate::messages::AuthMessage::GogTokenRefreshed(r)),
                    ));
                }

                if !self.profile.epic_refresh_token.is_empty()
                    && !self.profile.epic_token.is_empty()
                {
                    let refresh = self.profile.epic_refresh_token.clone();
                    startup_tasks.push(crate::app::spawn_task(
                        move || crate::epic::auth::refresh_token(&refresh),
                        |r| Message::Auth(crate::messages::AuthMessage::EpicTokenRefreshed(r)),
                    ));
                }

                if !startup_tasks.is_empty() {
                    return Task::batch(startup_tasks);
                }
            }
            Err(e) => {
                self.show_error(format!("Load error: {}", e));
            }
        }
        Task::none()
    }

    // ── Export ──

    pub(crate) fn handle_export_json(&mut self) -> Task<Message> {
        match db::export_games_json_from_slice(&self.games) {
            Ok(json) => crate::app::spawn_task(
                move || {
                    let export_path = db::exports_dir().join("spotter_export.json");
                    std::fs::write(&export_path, &json)
                        .map_err(|e| format!("Write error: {}", e))?;
                    Ok(export_path.to_string_lossy().to_string())
                },
                Message::ExportComplete,
            ),
            Err(e) => {
                self.show_error(format!("Export error: {}", e));
                Task::none()
            }
        }
    }

    pub(crate) fn handle_export_csv(&mut self) -> Task<Message> {
        match db::export_games_csv_from_slice(&self.games) {
            Ok(csv) => crate::app::spawn_task(
                move || {
                    let export_path = db::exports_dir().join("spotter_export.csv");
                    std::fs::write(&export_path, &csv)
                        .map_err(|e| format!("Write error: {}", e))?;
                    Ok(export_path.to_string_lossy().to_string())
                },
                Message::ExportComplete,
            ),
            Err(e) => {
                self.show_error(format!("Export error: {}", e));
                Task::none()
            }
        }
    }

    // ── Covers ──

    pub(crate) fn handle_download_covers(&self) -> Task<Message> {
        let games: Vec<(usize, String, String)> = self
            .games
            .iter()
            .enumerate()
            .filter(|(_, g)| !g.cover_url.is_empty() && !self.cover_cache.contains(&g.title))
            .map(|(i, g)| (i, g.cover_url.clone(), g.title.clone()))
            .collect();

        if games.is_empty() {
            return Task::none();
        }

        crate::app::spawn_task(
            move || {
                let mut results = Vec::with_capacity(games.len());
                for (idx, url, title) in &games {
                    if let Ok(path) = crate::images::download_cover(url, title) {
                        results.push((*idx, path.to_string_lossy().to_string()));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Ok(results)
            },
            Message::CoversDownloaded,
        )
    }

    // ── Achievements ──

    pub(crate) fn handle_achievements_loaded(
        &mut self,
        result: Result<Vec<Achievement>, String>,
    ) -> Task<Message> {
        self.achievements_loading = false;
        match result {
            Ok(list) => {
                self.achievements = list;
                // Download missing achievement icons in the background
                if !self.achievements.is_empty() {
                    if let crate::app::Screen::GameDetail(id) = &self.screen {
                        if let Some(idx) = self.find_game_index(*id) {
                            if let Some(appid) = self.games[idx].steam_appid {
                                let icon_data: Vec<(String, String, String)> = self
                                    .achievements
                                    .iter()
                                    .map(|a| {
                                        (
                                            a.api_name.clone(),
                                            a.icon_url.clone(),
                                            a.icon_gray_url.clone(),
                                        )
                                    })
                                    .collect();
                                return Task::perform(
                                    async move {
                                        let _ = std::thread::spawn(move || {
                                            crate::images::download_achievement_icons_minimal(
                                                appid, &icon_data,
                                            );
                                        })
                                        .join();
                                    },
                                    Message::AchievementIconsDownloaded,
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[app] Failed to load achievements: {}", e);
            }
        }
        Task::none()
    }

    // ── Import result handling ──

    /// Handle a completed import (any platform). Merges games, persists, downloads covers.
    pub(crate) fn handle_import_result(
        &mut self,
        platform: &str,
        result: Result<Vec<Game>, String>,
    ) -> Task<Message> {
        self.importing.remove(platform);
        match result {
            Ok(new_games) => {
                let count = new_games.len();
                eprintln!("[app] {} import returned {} games", platform, count);
                let (added, updated) = self.merge_imported_games(new_games);
                self.invalidate_filter_cache();
                let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                if self.import_history.len() >= 50 {
                    self.import_history.pop_front();
                }
                self.import_history.push_back(format!(
                    "[{}] {}: {} found, {} new, {} updated",
                    timestamp, platform, count, added, updated
                ));

                // Persist ALL games (including new ones) to DB immediately,
                // then move new games to the cascade queue for visual effect.
                // Use the ID-returning variant so that newly inserted games get
                // their DB ids patched back into memory (fixes "..." display).
                let save = self.persist_games_returning_ids();

                // Auto-backup after import
                let backup_task = match db::export_games_json_from_slice(&self.games) {
                    Ok(json) => crate::app::spawn_task(
                        move || {
                            let path = db::exports_dir().join("spotter_auto_backup.json");
                            std::fs::write(&path, &json)
                                .map_err(|e| format!("Auto-backup write error: {}", e))?;
                            eprintln!("[app] Auto-backup saved to {:?}", path);
                            Ok(())
                        },
                        Message::DataSaved,
                    ),
                    Err(e) => {
                        eprintln!("[app] Auto-backup serialization failed: {}", e);
                        Task::none()
                    }
                };

                if added > 0 {
                    // Move newly added games to cascade queue
                    let start = self.games.len() - added;
                    let queue_start = self.import_queue.len();
                    self.import_queue.extend(self.games.drain(start..));
                    self.import_queue[queue_start..].reverse();
                    self.import_queue_total += added;

                    self.import_status = format!("{}: importing {} games...", platform, added);

                    let drain = Self::cascade_delay(30);
                    Task::batch([save, drain, backup_task])
                } else {
                    self.import_status =
                        format!("{}: {} found, {} updated", platform, count, updated);
                    self.show_success(format!("{}: {} updated", platform, updated));
                    let covers = if self.settings.download_covers_auto {
                        Task::perform(async {}, |_| Message::DownloadCovers)
                    } else {
                        Task::none()
                    };
                    Task::batch([save, covers, backup_task])
                }
            }
            Err(e) => {
                self.import_status = format!("{} error: {}", platform, e);
                let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                if self.import_history.len() >= 50 {
                    self.import_history.pop_front();
                }
                self.import_history
                    .push_back(format!("[{}] {} ERROR: {}", timestamp, platform, e));
                self.show_error(e);
                Task::none()
            }
        }
    }

    // ── Cascade import ──

    pub(crate) fn handle_drain_import_queue(&mut self) -> Task<Message> {
        if self.import_queue.is_empty() {
            return Task::none();
        }

        let remaining_before = self.import_queue.len();

        // Adaptive batch size: larger batches for big queues so 1 600-game imports
        // finish in seconds instead of minutes.
        let batch = if remaining_before > 500 {
            50
        } else if remaining_before > 200 {
            25
        } else if remaining_before > 100 {
            15
        } else if remaining_before > 50 {
            8
        } else if remaining_before > 20 {
            4
        } else {
            2
        };

        for _ in 0..batch.min(remaining_before) {
            if let Some(game) = self.import_queue.pop() {
                self.games.push(game);
            }
        }

        // One filter invalidation per batch instead of per game.
        self.invalidate_filter_cache();

        let remaining = self.import_queue.len();
        let done = self.import_queue_total - remaining;

        if remaining > 0 {
            self.import_status = format!("Importing... {}/{}", done, self.import_queue_total);
            // Adaptive delay: slow down when few remain for a nicer visual finish.
            let delay = if remaining > 200 {
                30
            } else if remaining > 100 {
                40
            } else if remaining > 50 {
                50
            } else {
                80
            };
            return Self::cascade_delay(delay);
        } else {
            // Cascade complete
            self.import_status =
                format!("Import complete: {} games added", self.import_queue_total);
            self.show_success(format!("{} games imported!", self.import_queue_total));
            self.import_queue_total = 0;
            // Rebuild cover cache: re-imported games may already have covers on disk.
            self.refresh_cover_cache();
            if self.settings.download_covers_auto {
                return Task::perform(async {}, |_| Message::DownloadCovers);
            }
        }
        Task::none()
    }

    // ── Merge / dedup ──

    /// Merge imported games into the library.
    /// Deduplicates by platform ID first, then title+platform, then title-only.
    /// Returns (added_count, updated_count).
    pub(crate) fn merge_imported_games(&mut self, new_games: Vec<Game>) -> (usize, usize) {
        let capacity = self.games.len() + self.import_queue.len();

        // Use separate indexes per ID type. String IDs are cloned once during indexing
        // but lookups use &str (HashMap<String, _>::get accepts &str via Borrow).
        let mut steam_idx: HashMap<u32, usize> = HashMap::with_capacity(capacity);
        let mut gog_idx: HashMap<String, usize> = HashMap::with_capacity(capacity);
        let mut epic_idx: HashMap<String, usize> = HashMap::with_capacity(capacity);
        let mut xbox_idx: HashMap<String, usize> = HashMap::with_capacity(capacity);
        let mut psn_idx: HashMap<String, usize> = HashMap::with_capacity(capacity);
        let mut title_plat_idx: HashMap<(String, Platform), usize> =
            HashMap::with_capacity(capacity);
        let mut title_idx: HashMap<String, usize> = HashMap::with_capacity(capacity);

        // Index games in self.games
        for (i, g) in self.games.iter().enumerate() {
            if let Some(id) = g.steam_appid {
                steam_idx.insert(id, i);
            }
            if let Some(ref id) = g.gog_id {
                gog_idx.insert(id.clone(), i);
            }
            if let Some(ref id) = g.epic_id {
                epic_idx.insert(id.clone(), i);
            }
            if let Some(ref id) = g.xbox_id {
                xbox_idx.insert(id.clone(), i);
            }
            if let Some(ref id) = g.psn_id {
                psn_idx.insert(id.clone(), i);
            }
            if !g.title.is_empty() {
                let lower = g.title.to_lowercase();
                title_plat_idx.insert((lower.clone(), g.platform), i);
                title_idx.insert(lower, i);
            }
        }
        // Also index games waiting in the cascade queue
        let sentinel = usize::MAX;
        for g in &self.import_queue {
            if let Some(id) = g.steam_appid {
                steam_idx.entry(id).or_insert(sentinel);
            }
            if let Some(ref id) = g.gog_id {
                gog_idx.entry(id.clone()).or_insert(sentinel);
            }
            if let Some(ref id) = g.epic_id {
                epic_idx.entry(id.clone()).or_insert(sentinel);
            }
            if let Some(ref id) = g.xbox_id {
                xbox_idx.entry(id.clone()).or_insert(sentinel);
            }
            if let Some(ref id) = g.psn_id {
                psn_idx.entry(id.clone()).or_insert(sentinel);
            }
            if !g.title.is_empty() {
                let lower = g.title.to_lowercase();
                title_plat_idx
                    .entry((lower.clone(), g.platform))
                    .or_insert(sentinel);
                title_idx.entry(lower).or_insert(sentinel);
            }
        }

        let mut added = 0;
        let mut updated = 0;

        for game in new_games {
            // Lookup by platform ID — HashMap<String>::get accepts &str via Borrow
            let idx_opt = game
                .steam_appid
                .and_then(|id| steam_idx.get(&id))
                .or_else(|| game.gog_id.as_deref().and_then(|id| gog_idx.get(id)))
                .or_else(|| game.epic_id.as_deref().and_then(|id| epic_idx.get(id)))
                .or_else(|| game.xbox_id.as_deref().and_then(|id| xbox_idx.get(id)))
                .or_else(|| game.psn_id.as_deref().and_then(|id| psn_idx.get(id)))
                .or_else(|| {
                    let title_lower = game.title.to_lowercase();
                    title_plat_idx
                        .get(&(title_lower.clone(), game.platform))
                        .or_else(|| title_idx.get(&title_lower))
                });

            if let Some(&idx) = idx_opt {
                if idx == usize::MAX {
                    updated += 1;
                    continue;
                }
                let g = &mut self.games[idx];
                if g.title.is_empty() && !game.title.is_empty() {
                    g.title = game.title;
                }
                if !game.genre.is_empty() {
                    g.genre = game.genre;
                }
                if !game.description.is_empty() {
                    g.description = game.description;
                }
                if !game.release_date.is_empty() {
                    g.release_date = game.release_date;
                }
                if !game.tags.is_empty() {
                    g.tags = game.tags;
                }
                if !game.cover_url.is_empty() {
                    g.cover_url = game.cover_url;
                }
                if game.review_percent.is_some() {
                    g.review_percent = game.review_percent;
                }
                if game.achievements_total > 0 {
                    g.achievements_unlocked = game.achievements_unlocked;
                    g.achievements_total = game.achievements_total;
                }
                if game.playtime_minutes > g.playtime_minutes {
                    g.playtime_minutes = game.playtime_minutes;
                }
                if game.steam_appid.is_some() && g.steam_appid.is_none() {
                    g.steam_appid = game.steam_appid;
                }
                if game.gog_id.is_some() && g.gog_id.is_none() {
                    g.gog_id = game.gog_id;
                }
                if game.epic_id.is_some() && g.epic_id.is_none() {
                    g.epic_id = game.epic_id;
                }
                if game.xbox_id.is_some() && g.xbox_id.is_none() {
                    g.xbox_id = game.xbox_id;
                }
                if game.psn_id.is_some() && g.psn_id.is_none() {
                    g.psn_id = game.psn_id;
                }
                updated += 1;
            } else {
                self.games.push(game);
                added += 1;
            }
        }

        self.games.retain(|g| !g.title.is_empty());

        (added, updated)
    }

    // ── Persistence helpers ──

    /// Save profile to database and OS keyring in a background thread.
    pub(crate) fn save_profile_task(&self) -> Task<Message> {
        let profile = self.profile.clone();
        crate::app::spawn_task(
            move || {
                let conn = db::open()?;
                db::save_profile(&conn, &profile)?;
                crate::keyring::store_profile_secrets(&profile);
                Ok(())
            },
            Message::ProfileSaved,
        )
    }

    /// Silently persist the profile to the database and OS keyring (no success toast).
    pub(crate) fn persist_profile(&self) -> Task<Message> {
        let profile = self.profile.clone();
        crate::app::spawn_task(
            move || {
                let conn = db::open()?;
                db::save_profile(&conn, &profile)?;
                crate::keyring::store_profile_secrets(&profile);
                Ok(())
            },
            Message::ProfileAutoSaved,
        )
    }

    /// Persist settings to the database.
    pub(crate) fn persist_settings(&self) -> Task<Message> {
        let settings = self.settings.clone();
        crate::app::spawn_task(
            move || {
                let conn = db::open()?;
                db::save_settings(&conn, &settings)?;
                Ok(())
            },
            |r| Message::Settings(crate::messages::SettingsMessage::Saved(r)),
        )
    }

    /// Persist all games to the database.
    /// Serializes to JSON on the main thread (cheap borrow) to avoid cloning
    /// the entire Vec<Game>. The background thread deserializes and saves.
    pub(crate) fn persist_games(&self) -> Task<Message> {
        let json = match serde_json::to_vec(&self.games) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("[app] Failed to serialize games for persist: {}", e);
                return Task::none();
            }
        };
        crate::app::spawn_task(
            move || {
                let games: Vec<Game> =
                    serde_json::from_slice(&json).map_err(|e| format!("Deserialize: {}", e))?;
                let conn = db::open()?;
                db::save_all_games_ref(&conn, &games)?;
                Ok(())
            },
            Message::DataSaved,
        )
    }

    /// Like `persist_games`, but uses the mutable `save_all_games` so that
    /// newly-inserted rows get their DB ids written back.  Returns those ids
    /// via `Message::GameIdsMapped` so the main thread can patch in-memory
    /// copies (both `self.games` and `self.import_queue`).
    pub(crate) fn persist_games_returning_ids(&self) -> Task<Message> {
        let json = match serde_json::to_vec(&self.games) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("[app] Failed to serialize games for persist: {}", e);
                return Task::none();
            }
        };
        crate::app::spawn_task(
            move || {
                let mut games: Vec<Game> =
                    serde_json::from_slice(&json).map_err(|e| format!("Deserialize: {}", e))?;
                // Remember which games have no ID yet
                let no_id: Vec<usize> = games
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| g.id.is_none())
                    .map(|(i, _)| i)
                    .collect();
                let conn = db::open()?;
                db::save_all_games(&conn, &mut games)?;
                // Collect newly assigned IDs
                let new_ids: Vec<(String, String, i64)> = no_id
                    .into_iter()
                    .filter_map(|i| {
                        let g = &games[i];
                        g.id.map(|id| (g.title.clone(), g.platform.to_string(), id))
                    })
                    .collect();
                Ok(new_ids)
            },
            Message::GameIdsMapped,
        )
    }

    /// Persist a single game to the database by its ID.
    pub(crate) fn persist_single_game(&self, game_id: i64) -> Task<Message> {
        if let Some(game) = self.games.iter().find(|g| g.id == Some(game_id)).cloned() {
            crate::app::spawn_task(
                move || {
                    let conn = db::open()?;
                    db::save_game(&conn, &game)?;
                    Ok(())
                },
                Message::DataSaved,
            )
        } else {
            Task::none()
        }
    }

    /// Schedule a delayed DrainImportQueue message.
    pub(crate) fn cascade_delay(ms: u64) -> Task<Message> {
        crate::app::delay_task(ms, || Message::DrainImportQueue)
    }
}
