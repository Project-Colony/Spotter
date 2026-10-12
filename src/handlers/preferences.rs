use colony_ui::preferences::Change;
use iced::Task;

use crate::app::{Message, Screen, Spotter};
use crate::i18n::{t, tf};
use crate::messages::PreferencesMessage;

impl Spotter {
    pub(crate) fn handle_preferences(&mut self, msg: PreferencesMessage) -> Task<Message> {
        let prefs = &mut self.preferences;
        match msg {
            PreferencesMessage::Toggle => {
                if self.screen == Screen::Preferences {
                    self.screen =
                        std::mem::replace(&mut self.screen_before_preferences, Screen::Library);
                } else {
                    self.screen_before_preferences =
                        std::mem::replace(&mut self.screen, Screen::Preferences);
                    // The page opens on General, every section closed.
                    self.preferences_category = 0;
                    self.preference_sections.clear();
                }
                return Task::none();
            }
            PreferencesMessage::Category(index) => {
                self.preferences_category = index;
                return Task::none();
            }
            PreferencesMessage::ToggleSection(key) => {
                if !self.preference_sections.remove(key) {
                    self.preference_sections.insert(key.to_string());
                }
                return Task::none();
            }
            PreferencesMessage::Saved(result) => {
                if let Err(e) = result {
                    self.show_error(tf("error_preferences_save", &[("error", &e)]));
                }
                return Task::none();
            }
            PreferencesMessage::Standard(change) => {
                prefs.standard.update(change);
                if let Change::Theme { .. } = change {
                    self.show_success(t("theme_applied"));
                }
            }
            PreferencesMessage::Language(language) => {
                prefs.language = language;
                crate::i18n::set_language(prefs.language.as_deref());
            }
            PreferencesMessage::DefaultStatus(status) => prefs.default_status = status,
            PreferencesMessage::DefaultPlatform(platform) => prefs.default_platform = platform,
            PreferencesMessage::ConfirmDelete => {
                prefs.confirm_before_delete = !prefs.confirm_before_delete;
            }
            PreferencesMessage::StartScreen(screen) => prefs.start_screen = screen,
            PreferencesMessage::CompactList => prefs.compact_list = !prefs.compact_list,
            PreferencesMessage::ShowCovers => {
                prefs.show_covers_in_list = !prefs.show_covers_in_list;
            }
            PreferencesMessage::ShowStatusLabels => {
                prefs.show_status_labels = !prefs.show_status_labels;
            }
            PreferencesMessage::LargeTargets => {
                prefs.large_click_targets = !prefs.large_click_targets;
            }
            PreferencesMessage::SidebarWidth(width) => prefs.sidebar_width = width.clamp(160, 400),
            PreferencesMessage::Notifications => {
                prefs.notifications_enabled = !prefs.notifications_enabled;
            }
            PreferencesMessage::ToastDuration(duration) => prefs.toast_duration = duration,
            PreferencesMessage::DateFormat(format) => prefs.date_format = format,
            PreferencesMessage::DefaultSortOrder(order) => {
                prefs.default_sort_order = order;
                self.sort_order = order;
                self.invalidate_filter_cache();
            }
            PreferencesMessage::RememberFilters => {
                prefs.remember_filters = !prefs.remember_filters;
            }
            PreferencesMessage::ShowDescriptions => {
                prefs.show_game_descriptions = !prefs.show_game_descriptions;
            }
            PreferencesMessage::AchievementsDisplay(display) => {
                prefs.achievements_display = display;
            }
            PreferencesMessage::DownloadCoversAuto => {
                prefs.download_covers_auto = !prefs.download_covers_auto;
            }
        }
        self.persist_preferences()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toggle(app: &mut Spotter) {
        let _ = app.handle_preferences(PreferencesMessage::Toggle);
    }

    #[test]
    fn the_toggle_returns_to_the_screen_it_was_opened_from() {
        let mut app = Spotter::default();
        app.screen = Screen::GameDetail(7);
        app.preferences_category = 3;
        app.preference_sections.insert("theme".into());

        toggle(&mut app);
        assert_eq!(app.screen, Screen::Preferences);
        assert_eq!(app.preferences_category, 0, "opens on General");
        assert!(app.preference_sections.is_empty(), "every section closed");

        toggle(&mut app);
        assert_eq!(app.screen, Screen::GameDetail(7));
    }

    #[test]
    fn escape_closes_the_page_too() {
        let mut app = Spotter::default();
        app.screen = Screen::Statistics;
        toggle(&mut app);
        let _ = app.update(Message::CancelDelete);
        assert_eq!(app.screen, Screen::Statistics);
    }
}
