//! Import message handlers — Steam, GOG, Epic, Xbox, PlayStation, JSON.
//!
//! Each platform follows the same dispatch pattern:
//!
//! 1. **Trigger** — `Message::Import{Platform}` from the UI starts the import.
//! 2. **Guard** — the handler checks auth prerequisites and dedup (`importing` set).
//! 3. **Spawn** — a background `spawn_task` calls the platform module's
//!    `full_import(auth_params...)` and maps the result to a `*Complete` message.
//! 4. **Complete** — the `*Complete` handler calls `handle_import_result()` which
//!    merges games, persists, starts the cascade animation, and triggers cover
//!    downloads.
//!
//! Auth is separate (`handlers/auth.rs` + `messages::AuthMessage`) and handles
//! login flows, code exchange, and proactive token refresh at startup.
//!
//! No shared `PlatformImporter` trait — each platform has different auth params
//! and flows, so the dispatch is kept explicit here.

use std::collections::HashMap;

use iced::Task;

use crate::app::{Message, Spotter};
use crate::db;
use crate::messages::AuthMessage;

impl Spotter {
    // ── Steam ──

    pub(crate) fn handle_steam_login(&mut self) -> Task<Message> {
        if self.steam_login_active {
            return Task::none();
        }
        self.steam_login_active = true;
        self.import_status = "Opening Steam login in browser...".into();
        super::super::app::spawn_task(crate::steam::auth::steam_login, |r| {
            Message::Auth(AuthMessage::SteamLoginComplete(r))
        })
    }

    pub(crate) fn handle_steam_login_complete(
        &mut self,
        result: Result<String, String>,
    ) -> Task<Message> {
        self.steam_login_active = false;
        match result {
            Ok(steam_id) => {
                self.profile.steam_id = steam_id.clone();
                self.import_status = String::new();
                self.show_success(format!("Steam ID detected: {}", steam_id));
                self.save_profile_task()
            }
            Err(e) => {
                self.import_status = String::new();
                self.show_error(format!("Steam login failed: {}", e));
                Task::none()
            }
        }
    }

    pub(crate) fn handle_import_steam(&mut self) -> Task<Message> {
        if self.profile.steam_api_key.is_empty() || self.profile.steam_id.is_empty() {
            self.show_error("Steam API key and Steam ID are required. Use 'Login with Steam' to get your ID, then enter your API key.");
            return Task::none();
        }
        if self.importing.contains("Steam") {
            return Task::none();
        }
        self.importing.insert("Steam".into());
        self.import_status = format!(
            "Importing from Steam... ({} existing games)",
            self.games.len()
        );
        eprintln!(
            "[app] Starting Steam import (id={})",
            &self.profile.steam_id
        );
        let api_key = self.profile.steam_api_key.clone();
        let steam_id = self.profile.steam_id.clone();
        // Build map of already-enriched games so full_import can skip them
        let mut existing_map: HashMap<u32, (bool, bool)> = HashMap::new();
        for g in &self.games {
            if let Some(appid) = g.steam_appid {
                let has_store =
                    !g.genre.is_empty() && !g.description.is_empty() && !g.release_date.is_empty();
                let ach_done =
                    g.achievements_total > 0 && g.achievements_unlocked == g.achievements_total;
                existing_map.insert(appid, (has_store, ach_done));
            }
        }
        super::super::app::spawn_task(
            move || crate::steam::full_import(&api_key, &steam_id, existing_map),
            Message::SteamImportComplete,
        )
    }

    // ── GOG ──

    pub(crate) fn handle_gog_login(&mut self) -> Task<Message> {
        self.gog_login_active = true;
        self.gog_code_input.clear();
        if let Err(e) = crate::gog::auth::open_login_page() {
            self.show_error(format!("Failed to open browser: {}", e));
            self.gog_login_active = false;
        }
        Task::none()
    }

    pub(crate) fn handle_gog_submit_code(&mut self) -> Task<Message> {
        let input = self.gog_code_input.clone();
        let code = match crate::gog::auth::extract_code(&input) {
            Some(c) => c,
            None => {
                self.show_error("Please paste the URL or code from the browser");
                return Task::none();
            }
        };
        self.import_status = "Exchanging GOG authorization code...".into();
        super::super::app::spawn_task(
            move || crate::gog::auth::exchange_code(&code),
            |r| Message::Auth(AuthMessage::GogLoginComplete(r)),
        )
    }

    pub(crate) fn handle_gog_login_complete(
        &mut self,
        result: Result<(String, String), String>,
    ) -> Task<Message> {
        self.gog_login_active = false;
        self.gog_code_input.clear();
        match result {
            Ok((access_token, refresh_token)) => {
                self.profile.gog_token = access_token;
                self.profile.gog_refresh_token = refresh_token;
                self.import_status = String::new();
                self.show_success("GOG login successful!");
                self.save_profile_task()
            }
            Err(e) => {
                self.import_status = String::new();
                self.show_error(format!("GOG login failed: {}", e));
                Task::none()
            }
        }
    }

    pub(crate) fn handle_import_gog(&mut self) -> Task<Message> {
        if self.profile.gog_token.is_empty() {
            self.show_error("Please login with GOG first");
            return Task::none();
        }
        if self.importing.contains("GOG") {
            return Task::none();
        }
        self.importing.insert("GOG".into());
        self.import_status = "Importing from GOG...".into();
        let token = self.profile.gog_token.clone();
        let refresh = self.profile.gog_refresh_token.clone();
        super::super::app::spawn_task(
            move || match crate::gog::full_import(&token) {
                Ok(games) => Ok((games, None)),
                Err(e) if !refresh.is_empty() => {
                    eprintln!("[gog] Import failed, trying token refresh: {}", e);
                    let (new_token, new_refresh) = crate::gog::auth::refresh_token(&refresh)?;
                    let games = crate::gog::full_import(&new_token)?;
                    Ok((games, Some((new_token, new_refresh))))
                }
                Err(e) => Err(e),
            },
            Message::GogImportComplete,
        )
    }

    pub(crate) fn handle_gog_import_complete(
        &mut self,
        result: Result<crate::app::GogImportResult, String>,
    ) -> Task<Message> {
        match result {
            Ok((games, refreshed_tokens)) => {
                // If token was refreshed, update profile
                let profile_save = if let Some((new_token, new_refresh)) = refreshed_tokens {
                    self.profile.gog_token = new_token;
                    self.profile.gog_refresh_token = new_refresh;
                    self.save_profile_task()
                } else {
                    Task::none()
                };
                let import = self.handle_import_result("GOG", Ok(games));
                Task::batch([import, profile_save])
            }
            Err(e) => self.handle_import_result("GOG", Err(e)),
        }
    }

    // ── Epic ──

    pub(crate) fn handle_epic_login(&mut self) -> Task<Message> {
        self.epic_login_active = true;
        self.epic_code_input.clear();
        if let Err(e) = crate::epic::auth::open_login_page() {
            self.show_error(format!("Failed to open browser: {}", e));
            self.epic_login_active = false;
        }
        Task::none()
    }

    pub(crate) fn handle_epic_submit_code(&mut self) -> Task<Message> {
        let input = self.epic_code_input.clone();
        let code = match crate::epic::auth::extract_code(&input) {
            Some(c) => c,
            None => {
                self.show_error(
                    "Please paste the JSON response or authorization code from the browser",
                );
                return Task::none();
            }
        };
        self.import_status = "Exchanging Epic authorization code...".into();
        super::super::app::spawn_task(
            move || crate::epic::auth::exchange_code(&code),
            |r| Message::Auth(AuthMessage::EpicLoginComplete(r)),
        )
    }

    pub(crate) fn handle_epic_login_complete(
        &mut self,
        result: Result<crate::epic::auth::EpicLoginResult, String>,
    ) -> Task<Message> {
        self.epic_login_active = false;
        self.epic_code_input.clear();
        match result {
            Ok((access_token, refresh_token, account_id, display_name)) => {
                self.profile.epic_token = access_token;
                self.profile.epic_refresh_token = refresh_token;
                self.profile.epic_account_id = account_id;
                if !display_name.is_empty() {
                    self.profile.epic_display_name = display_name.clone();
                }
                self.import_status = String::new();
                let name = if display_name.is_empty() {
                    "account linked".to_string()
                } else {
                    display_name
                };
                self.show_success(format!("Epic Games login successful: {}", name));
                self.save_profile_task()
            }
            Err(e) => {
                self.import_status = String::new();
                self.show_error(format!("Epic login failed: {}", e));
                Task::none()
            }
        }
    }

    pub(crate) fn handle_import_epic(&mut self) -> Task<Message> {
        if self.importing.contains("Epic") {
            return Task::none();
        }
        self.importing.insert("Epic".into());

        // If user is logged in with Epic account, use online import
        if !self.profile.epic_token.is_empty() && !self.profile.epic_account_id.is_empty() {
            self.import_status = "Importing from Epic Games account...".into();
            let token = self.profile.epic_token.clone();
            let account_id = self.profile.epic_account_id.clone();
            let refresh = self.profile.epic_refresh_token.clone();
            super::super::app::spawn_task(
                move || crate::epic::full_import_online(&token, &account_id, &refresh),
                Message::EpicImportComplete,
            )
        } else {
            // Fallback to local scanning
            self.import_status = "Scanning Epic Games Launcher...".into();
            super::super::app::spawn_task(
                || crate::epic::full_import_local().map(|games| (games, None)),
                Message::EpicImportComplete,
            )
        }
    }

    pub(crate) fn handle_epic_import_complete(
        &mut self,
        result: Result<crate::app::EpicImportResult, String>,
    ) -> Task<Message> {
        match result {
            Ok((games, refreshed_tokens)) => {
                let profile_save =
                    if let Some((new_token, new_refresh, new_account_id, new_display_name)) =
                        refreshed_tokens
                    {
                        self.profile.epic_token = new_token;
                        self.profile.epic_refresh_token = new_refresh;
                        self.profile.epic_account_id = new_account_id;
                        if !new_display_name.is_empty() {
                            self.profile.epic_display_name = new_display_name;
                        }
                        self.persist_profile()
                    } else {
                        Task::none()
                    };

                // Clean up old non-game Epic entries from previous imports
                // (UE engine items, promos, hex IDs, free tokens, etc.)
                let cleanup = self.cleanup_epic_junk();

                let import = self.handle_import_result("Epic", Ok(games));
                Task::batch([import, profile_save, cleanup])
            }
            Err(e) => self.handle_import_result("Epic", Err(e)),
        }
    }

    /// Remove previously imported Epic entries that are clearly not games
    /// (UE engine items, promos, hex IDs, etc.) from the in-memory list and the DB.
    fn cleanup_epic_junk(&mut self) -> Task<Message> {
        use crate::models::Platform;

        let mut junk_ids: Vec<i64> = Vec::new();
        self.games.retain(|g| {
            if g.platform != Platform::Epic {
                return true; // keep non-Epic games
            }
            if crate::epic::is_epic_junk_title(&g.title) {
                eprintln!("[epic] Cleanup: removing junk entry \"{}\"", g.title);
                if let Some(id) = g.id {
                    junk_ids.push(id);
                }
                return false; // remove
            }
            true // keep
        });

        if junk_ids.is_empty() {
            return Task::none();
        }

        let count = junk_ids.len();
        eprintln!(
            "[epic] Cleanup: removing {} old non-game entries from DB",
            count
        );
        self.invalidate_filter_cache();

        crate::app::spawn_task(
            move || {
                let conn = db::open()?;
                for id in &junk_ids {
                    db::delete_game(&conn, *id)?;
                }
                eprintln!("[epic] Cleanup: deleted {} entries from DB", junk_ids.len());
                Ok(())
            },
            Message::DataSaved,
        )
    }

    // ── Xbox ──

    pub(crate) fn handle_import_xbox(&mut self) -> Task<Message> {
        if self.profile.xbox_api_key.is_empty() {
            self.show_error(
                "Xbox API key is required. Get one at xbl.io and configure it in Profile.",
            );
            return Task::none();
        }
        if self.importing.contains("Xbox") {
            return Task::none();
        }
        self.importing.insert("Xbox".into());
        self.import_status = "Importing from Xbox Live...".into();
        let api_key = self.profile.xbox_api_key.clone();
        super::super::app::spawn_task(
            move || crate::xbox::full_import(&api_key),
            Message::XboxImportComplete,
        )
    }

    pub(crate) fn handle_xbox_import_complete(
        &mut self,
        result: Result<crate::app::XboxImportResult, String>,
    ) -> Task<Message> {
        match result {
            Ok((games, xuid)) => {
                let profile_save = if let Some(new_xuid) = xuid {
                    if self.profile.xbox_xuid != new_xuid {
                        self.profile.xbox_xuid = new_xuid;
                        self.persist_profile()
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                };
                let import = self.handle_import_result("Xbox", Ok(games));
                Task::batch([import, profile_save])
            }
            Err(e) => self.handle_import_result("Xbox", Err(e)),
        }
    }

    // ── PlayStation ──

    pub(crate) fn handle_import_playstation(&mut self) -> Task<Message> {
        if self.profile.psn_npsso.is_empty() {
            self.show_error("PSN NPSSO token is required. Get it from the PlayStation website.");
            return Task::none();
        }
        if self.importing.contains("PlayStation") {
            return Task::none();
        }
        self.importing.insert("PlayStation".into());
        self.import_status = "Importing from PlayStation Network...".into();
        let npsso = self.profile.psn_npsso.clone();
        super::super::app::spawn_task(
            move || crate::playstation::full_import(&npsso),
            Message::PlayStationImportComplete,
        )
    }

    // ── JSON ──

    pub(crate) fn handle_import_json(&mut self) -> Task<Message> {
        if self.importing.contains("JSON") {
            return Task::none();
        }
        self.importing.insert("JSON".into());
        self.import_status = "Importing from JSON backup...".into();
        super::super::app::spawn_task(
            || db::import_games_json().map_err(Into::into),
            Message::ImportJsonComplete,
        )
    }
}
