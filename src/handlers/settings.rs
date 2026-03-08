use iced::Task;

use crate::app::{delay_task, Message, Spotter};
use crate::messages::SettingsMessage;

impl Spotter {
    pub(crate) fn handle_settings(&mut self, msg: SettingsMessage) -> Task<Message> {
        match msg {
            SettingsMessage::ChangeSection(section) => {
                self.settings_section = section;
            }
            SettingsMessage::DefaultStatus(status) => {
                self.settings.default_status = status;
                return self.persist_settings();
            }
            SettingsMessage::DefaultPlatform(platform) => {
                self.settings.default_platform = platform;
                return self.persist_settings();
            }
            SettingsMessage::ConfirmDelete => {
                self.settings.confirm_before_delete = !self.settings.confirm_before_delete;
                return self.persist_settings();
            }
            SettingsMessage::StartScreen(screen) => {
                self.settings.start_screen = screen;
                return self.persist_settings();
            }
            SettingsMessage::ThemeMode(mode) => {
                self.settings.theme_mode = mode;
                return self.persist_settings();
            }
            SettingsMessage::AccentColor(color) => {
                self.settings.accent_color = color;
                return self.persist_settings();
            }
            SettingsMessage::CompactList => {
                self.settings.compact_list = !self.settings.compact_list;
                return self.persist_settings();
            }
            SettingsMessage::ShowCovers => {
                self.settings.show_covers_in_list = !self.settings.show_covers_in_list;
                return self.persist_settings();
            }
            SettingsMessage::UiScale(scale) => {
                self.settings.ui_scale = scale;
                return self.persist_settings();
            }
            SettingsMessage::HighContrast => {
                self.settings.high_contrast = !self.settings.high_contrast;
                return self.persist_settings();
            }
            SettingsMessage::ShowStatusLabels => {
                self.settings.show_status_labels = !self.settings.show_status_labels;
                return self.persist_settings();
            }
            SettingsMessage::LargeTargets => {
                self.settings.large_click_targets = !self.settings.large_click_targets;
                return self.persist_settings();
            }
            SettingsMessage::SidebarWidth(w) => {
                self.settings.sidebar_width = w.clamp(160, 400);
                return self.persist_settings();
            }
            SettingsMessage::Notifications => {
                self.settings.notifications_enabled = !self.settings.notifications_enabled;
                return self.persist_settings();
            }
            SettingsMessage::ToastDuration(d) => {
                self.settings.toast_duration = d;
                return self.persist_settings();
            }
            SettingsMessage::DateFormat(f) => {
                self.settings.date_format = f;
                return self.persist_settings();
            }
            SettingsMessage::DefaultSortOrder(order) => {
                self.settings.default_sort_order = order;
                self.sort_order = order;
                self.invalidate_filter_cache();
                return self.persist_settings();
            }
            SettingsMessage::RememberFilters => {
                self.settings.remember_filters = !self.settings.remember_filters;
                return self.persist_settings();
            }
            SettingsMessage::ShowDescriptions => {
                self.settings.show_game_descriptions = !self.settings.show_game_descriptions;
                return self.persist_settings();
            }
            SettingsMessage::AchievementsDisplay(d) => {
                self.settings.achievements_display = d;
                return self.persist_settings();
            }
            SettingsMessage::DownloadCoversAuto => {
                self.settings.download_covers_auto = !self.settings.download_covers_auto;
                return self.persist_settings();
            }
            SettingsMessage::Saved(result) => {
                if let Err(e) = result {
                    self.show_error(format!("Settings save error: {}", e));
                } else {
                    self.settings_just_saved = true;
                    return delay_task(1500, || Message::Settings(SettingsMessage::SaveShown));
                }
            }
            SettingsMessage::SaveShown => {
                self.settings_just_saved = false;
            }
        }
        Task::none()
    }
}
