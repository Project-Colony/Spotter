//! The Preferences page, built from the shared Colony kit: General,
//! Appearance and Accessibility first, then Spotter's own Library and Imports.
//! It replaces the content area; the sidebar stays.

use colony_ui::widgets::{
    category_heading, category_list, collapsible_section, dropdown, functional_toggle,
    preferences_page, section_description, setting_row, PreferencesView, STANDARD_CATEGORIES,
};
use colony_ui::Typography;
use iced::widget::{column, Column};
use iced::{Element, Length};

use crate::app::{Message, Spotter};
use crate::i18n::{t, LANGUAGES};
use crate::messages::PreferencesMessage;
use crate::models::*;

/// Spotter's own categories, after the three every Colony program opens with.
const OWN_CATEGORIES: [&str; 2] = ["preferences_cat_library", "preferences_cat_imports"];

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let typo = Typography::current();
    let labels = STANDARD_CATEGORIES
        .iter()
        .chain(&OWN_CATEGORIES)
        .map(|key| t(key));
    let categories = category_list(&typo, labels, app.preferences_category, |index| {
        PreferencesMessage::Category(index).into()
    });

    let kit = PreferencesView::new(
        typo,
        &app.preferences.standard,
        &app.preference_sections,
        |change| PreferencesMessage::Standard(change).into(),
        |key| PreferencesMessage::ToggleSection(key).into(),
    );
    let page = Page { app, typo };
    let content: Element<'_, Message> = match app.preferences_category {
        0 => kit
            .general()
            .push(page.language())
            .push(page.startup())
            .push(page.notifications())
            .push(page.dates())
            .into(),
        1 => category_heading(
            &typo,
            t("settings_cat_appearance"),
            t("settings_appearance_desc"),
        )
        .push(kit.theme_section())
        .push(kit.colors_section())
        .push(kit.typography_section())
        .push(page.layout())
        .push(kit.effects_section())
        .push(kit.preview_section())
        .into(),
        2 => kit
            .accessibility_heading()
            .push(kit.vision_section())
            .push(page.colour_cues())
            .push(kit.motion_section())
            .push(page.navigation())
            .push(kit.reading_section())
            .into(),
        3 => category_heading(
            &typo,
            t("preferences_cat_library"),
            t("preferences_library_desc"),
        )
        .push(page.new_games())
        .push(page.sorting())
        .push(page.game_details())
        .push(page.deleting())
        .into(),
        _ => category_heading(
            &typo,
            t("preferences_cat_imports"),
            t("preferences_imports_desc"),
        )
        .push(page.covers())
        .into(),
    };

    preferences_page(
        &typo,
        categories,
        content,
        PreferencesMessage::Toggle.into(),
    )
}

/// Spotter's own sections, drawn like the kit's.
struct Page<'a> {
    app: &'a Spotter,
    typo: Typography,
}

impl<'a> Page<'a> {
    fn prefs(&self) -> &'a Preferences {
        &self.app.preferences
    }

    // ── General ──

    fn language(&self) -> Element<'a, Message> {
        let options = std::iter::once((None, t("preferences_language_system").to_owned())).chain(
            LANGUAGES
                .iter()
                .map(|(tag, name)| (Some((*tag).to_owned()), (*name).to_owned())),
        );
        self.section(
            "language",
            "preferences_section_language",
            "preferences_language_desc",
            vec![self.choice(
                "preferences_language",
                "preferences_language_choice_desc",
                options,
                &self.prefs().language,
                PreferencesMessage::Language,
            )],
        )
    }

    fn startup(&self) -> Element<'a, Message> {
        self.section(
            "startup",
            "preferences_section_startup",
            "preferences_startup_desc",
            vec![self.choice(
                "preferences_start_screen",
                "preferences_start_screen_desc",
                StartScreen::ALL.map(|s| (s, s.label().to_owned())),
                &self.prefs().start_screen,
                PreferencesMessage::StartScreen,
            )],
        )
    }

    fn notifications(&self) -> Element<'a, Message> {
        let prefs = self.prefs();
        self.section(
            "notifications",
            "preferences_section_notifications",
            "preferences_notifications_desc",
            vec![
                self.toggle(
                    "preferences_show_notifications",
                    "preferences_show_notifications_desc",
                    prefs.notifications_enabled,
                    PreferencesMessage::Notifications,
                ),
                self.choice(
                    "preferences_toast_duration",
                    "preferences_toast_duration_desc",
                    ToastDuration::ALL.map(|d| (d, d.label().to_owned())),
                    &prefs.toast_duration,
                    PreferencesMessage::ToastDuration,
                ),
            ],
        )
    }

    fn dates(&self) -> Element<'a, Message> {
        self.section(
            "dates",
            "preferences_section_dates",
            "preferences_dates_desc",
            vec![self.choice(
                "preferences_date_format",
                "preferences_date_format_desc",
                DateFormat::ALL.map(|f| (f, f.label().to_owned())),
                &self.prefs().date_format,
                PreferencesMessage::DateFormat,
            )],
        )
    }

    // ── Appearance ──

    fn layout(&self) -> Element<'a, Message> {
        let prefs = self.prefs();
        let widths = [
            (180, t("preferences_sidebar_narrow").to_owned()),
            (220, t("preferences_sidebar_normal").to_owned()),
            (280, t("preferences_sidebar_wide").to_owned()),
        ];
        self.section(
            "layout",
            "preferences_section_layout",
            "preferences_layout_desc",
            vec![
                self.choice(
                    "preferences_sidebar_width",
                    "preferences_sidebar_width_desc",
                    widths,
                    &prefs.sidebar_width,
                    PreferencesMessage::SidebarWidth,
                ),
                self.toggle(
                    "preferences_compact_list",
                    "preferences_compact_list_desc",
                    prefs.compact_list,
                    PreferencesMessage::CompactList,
                ),
                self.toggle(
                    "preferences_show_covers",
                    "preferences_show_covers_desc",
                    prefs.show_covers_in_list,
                    PreferencesMessage::ShowCovers,
                ),
            ],
        )
    }

    // ── Accessibility ──

    fn colour_cues(&self) -> Element<'a, Message> {
        self.section(
            "colour_cues",
            "preferences_section_colour_cues",
            "preferences_colour_cues_desc",
            vec![self.toggle(
                "preferences_status_labels",
                "preferences_status_labels_desc",
                self.prefs().show_status_labels,
                PreferencesMessage::ShowStatusLabels,
            )],
        )
    }

    fn navigation(&self) -> Element<'a, Message> {
        self.section(
            "navigation",
            "preferences_section_navigation",
            "preferences_navigation_desc",
            vec![self.toggle(
                "preferences_large_targets",
                "preferences_large_targets_desc",
                self.prefs().large_click_targets,
                PreferencesMessage::LargeTargets,
            )],
        )
    }

    // ── Library ──

    fn new_games(&self) -> Element<'a, Message> {
        let prefs = self.prefs();
        self.section(
            "new_games",
            "preferences_section_new_games",
            "preferences_new_games_desc",
            vec![
                self.choice(
                    "preferences_default_status",
                    "preferences_preselected_desc",
                    GameStatus::ALL.map(|s| (s, s.label().to_owned())),
                    &prefs.default_status,
                    PreferencesMessage::DefaultStatus,
                ),
                self.choice(
                    "preferences_default_platform",
                    "preferences_preselected_desc",
                    Platform::ALL.map(|p| (p, p.to_string())),
                    &prefs.default_platform,
                    PreferencesMessage::DefaultPlatform,
                ),
            ],
        )
    }

    fn sorting(&self) -> Element<'a, Message> {
        let prefs = self.prefs();
        self.section(
            "sorting",
            "preferences_section_sorting",
            "preferences_sorting_desc",
            vec![
                self.choice(
                    "preferences_sort_order",
                    "preferences_sort_order_desc",
                    SortOrder::ALL.map(|s| (s, s.label().to_owned())),
                    &prefs.default_sort_order,
                    PreferencesMessage::DefaultSortOrder,
                ),
                self.toggle(
                    "preferences_remember_filters",
                    "preferences_remember_filters_desc",
                    prefs.remember_filters,
                    PreferencesMessage::RememberFilters,
                ),
            ],
        )
    }

    fn game_details(&self) -> Element<'a, Message> {
        let prefs = self.prefs();
        self.section(
            "game_details",
            "preferences_section_game_details",
            "preferences_game_details_desc",
            vec![
                self.toggle(
                    "preferences_show_descriptions",
                    "preferences_show_descriptions_desc",
                    prefs.show_game_descriptions,
                    PreferencesMessage::ShowDescriptions,
                ),
                self.choice(
                    "preferences_achievements_display",
                    "preferences_achievements_display_desc",
                    AchievementsDisplay::ALL.map(|d| (d, d.label().to_owned())),
                    &prefs.achievements_display,
                    PreferencesMessage::AchievementsDisplay,
                ),
            ],
        )
    }

    fn deleting(&self) -> Element<'a, Message> {
        self.section(
            "deleting",
            "preferences_section_deleting",
            "preferences_deleting_desc",
            vec![self.toggle(
                "preferences_confirm_delete",
                "preferences_confirm_delete_desc",
                self.prefs().confirm_before_delete,
                PreferencesMessage::ConfirmDelete,
            )],
        )
    }

    // ── Imports ──

    fn covers(&self) -> Element<'a, Message> {
        self.section(
            "covers",
            "preferences_section_covers",
            "preferences_covers_desc",
            vec![self.toggle(
                "preferences_auto_covers",
                "preferences_auto_covers_desc",
                self.prefs().download_covers_auto,
                PreferencesMessage::DownloadCoversAuto,
            )],
        )
    }

    // ── Building blocks ──

    /// A collapsible section: its title, what its settings do, then them.
    fn section(
        &self,
        key: &'static str,
        title: &str,
        description: &str,
        controls: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        collapsible_section(
            &self.typo,
            t(title),
            self.app.preference_sections.contains(key),
            PreferencesMessage::ToggleSection(key).into(),
            column![
                section_description(&self.typo, t(description)),
                Column::with_children(controls).width(Length::Fill),
            ]
            .into(),
        )
    }

    fn toggle(
        &self,
        title: &str,
        description: &str,
        on: bool,
        msg: PreferencesMessage,
    ) -> Element<'a, Message> {
        functional_toggle(&self.typo, t(title), t(description), on, msg.into())
    }

    /// A setting picked from a drop-down list.
    fn choice<T: Clone + PartialEq + 'a>(
        &self,
        title: &str,
        description: &str,
        options: impl IntoIterator<Item = (T, String)>,
        selected: &T,
        msg: impl Fn(T) -> PreferencesMessage + 'a,
    ) -> Element<'a, Message> {
        setting_row(
            &self.typo,
            t(title),
            t(description),
            dropdown(&self.typo, options, selected, move |value| {
                msg(value).into()
            }),
        )
    }
}
