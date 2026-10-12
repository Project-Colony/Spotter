use colony_ui::preferences::Change;
use colony_ui::sz;
use colony_ui::theme::Palette;
use colony_ui::widgets::PreferencesView;
use colony_ui::Typography;
use iced::widget::{button, column, container, row, rule, space, toggler};
use iced::{Color, Element, Length};

use crate::app::{Message, Spotter};
use crate::messages::SettingsMessage;
use crate::models::*;
use crate::theme::{self, ui_text};

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let saved_indicator: Element<'_, Message> = if app.settings_just_saved {
        ui_text("Saved!", 12).color(Palette::SUCCESS()).into()
    } else {
        space().into()
    };
    let header = column![
        row![
            ui_text(format!("{} Settings", theme::icons::SETTINGS), 24)
                .font(colony_ui::ui_font_bold())
                .color(Palette::TEXT_PRIMARY()),
            saved_indicator,
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
        ui_text("Customize your Spotter experience", 12).color(Palette::TEXT_MUTED()),
    ]
    .spacing(4);

    let accent = Palette::ACCENT();

    // Section tabs
    let tab_bar = row![
        section_tab(
            "General",
            SettingsSection::General,
            app.settings_section,
            accent
        ),
        section_tab(
            "Appearances",
            SettingsSection::Appearances,
            app.settings_section,
            accent
        ),
        section_tab(
            "Accessibility",
            SettingsSection::Accessibility,
            app.settings_section,
            accent
        ),
    ]
    .spacing(4);

    let section_content = match app.settings_section {
        SettingsSection::General => general_section(app),
        SettingsSection::Appearances => appearances_section(app),
        SettingsSection::Accessibility => accessibility_section(app),
    };

    let card = container(column![tab_bar, rule::horizontal(1), section_content,].spacing(16))
        .padding(20)
        .width(Length::Fill)
        .style(theme::card_style(
            Palette::BG_CARD(),
            Palette::BORDER_SUBTLE(),
            8.0,
        ));

    let content = column![header, rule::horizontal(1), card]
        .spacing(16)
        .padding(24)
        .width(Length::Fill);

    iced::widget::scrollable(container(content).width(Length::Fill).height(Length::Fill)).into()
}

fn section_tab<'a>(
    label: &'a str,
    target: SettingsSection,
    current: SettingsSection,
    accent: Color,
) -> Element<'a, Message> {
    button(ui_text(label, 14))
        .on_press(SettingsMessage::ChangeSection(target).into())
        .padding([8, 16])
        .style(theme::chip_style(current == target, accent, 8.0))
        .into()
}

// ───── General ─────

fn general_section(app: &Spotter) -> Element<'_, Message> {
    let settings = &app.settings;

    // Default status for new games
    let default_status_row = setting_row(
        "Default status for new games",
        "When manually adding a game, this status will be pre-selected",
        row(GameStatus::all().iter().copied().map(|s| {
            option_chip(
                &s.to_string(),
                settings.default_status == s,
                s.color(),
                SettingsMessage::DefaultStatus(s),
            )
        }))
        .spacing(4),
    );

    // Default platform for new games
    let default_platform_row = setting_row(
        "Default platform",
        "Pre-selected platform when adding a game manually",
        row(Platform::all().iter().copied().map(|p| {
            option_chip(
                &p.to_string(),
                settings.default_platform == p,
                p.color(),
                SettingsMessage::DefaultPlatform(p),
            )
        }))
        .spacing(4),
    );

    // Confirm before delete
    let confirm_delete_row = setting_toggle(
        "Confirm before deleting",
        "Show a confirmation dialog before removing a game",
        settings.confirm_before_delete,
        SettingsMessage::ConfirmDelete.into(),
    );

    // Start screen
    let start_screen_row = setting_row(
        "Start screen",
        "Which screen to show when Spotter launches",
        row![
            option_chip(
                "Library",
                settings.start_screen == StartScreen::Library,
                Palette::ACCENT(),
                SettingsMessage::StartScreen(StartScreen::Library)
            ),
            option_chip(
                "Statistics",
                settings.start_screen == StartScreen::Statistics,
                Palette::ACCENT(),
                SettingsMessage::StartScreen(StartScreen::Statistics)
            ),
            option_chip(
                "Import",
                settings.start_screen == StartScreen::Import,
                Palette::ACCENT(),
                SettingsMessage::StartScreen(StartScreen::Import)
            ),
            option_chip(
                "Profile",
                settings.start_screen == StartScreen::Profile,
                Palette::ACCENT(),
                SettingsMessage::StartScreen(StartScreen::Profile)
            ),
        ]
        .spacing(4),
    );

    // Notifications
    let notifications_row = setting_toggle(
        "Show notifications",
        "Display success toasts after actions (errors always show)",
        settings.notifications_enabled,
        SettingsMessage::Notifications.into(),
    );

    // Toast duration
    let toast_duration_row = setting_row(
        "Toast duration",
        "How long notification toasts stay on screen",
        row(ToastDuration::all().iter().copied().map(|d| {
            let active = settings.toast_duration == d;
            option_chip(
                &d.to_string(),
                active,
                Palette::ACCENT(),
                SettingsMessage::ToastDuration(d),
            )
        }))
        .spacing(4),
    );

    // Date format
    let date_format_row = setting_row(
        "Date format",
        "Format used for dates throughout the app",
        row(DateFormat::all().iter().copied().map(|f| {
            let active = settings.date_format == f;
            option_chip(
                &f.to_string(),
                active,
                Palette::ACCENT(),
                SettingsMessage::DateFormat(f),
            )
        }))
        .spacing(4),
    );

    // Default sort order
    let sort_order_row = setting_row(
        "Default sort order",
        "Sort order is remembered across sessions",
        row(SortOrder::all().iter().copied().map(|s| {
            let active = settings.default_sort_order == s;
            option_chip(
                &s.to_string(),
                active,
                Palette::ACCENT(),
                SettingsMessage::DefaultSortOrder(s),
            )
        }))
        .spacing(4)
        .wrap(),
    );

    // Remember filters
    let remember_filters_row = setting_toggle(
        "Remember filters",
        "Persist status and platform filters across sessions",
        settings.remember_filters,
        SettingsMessage::RememberFilters.into(),
    );

    // Show game descriptions
    let show_descriptions_row = setting_toggle(
        "Show descriptions in library",
        "Display game descriptions in library card view",
        settings.show_game_descriptions,
        SettingsMessage::ShowDescriptions.into(),
    );

    // Achievements display
    let achievements_display_row = setting_row(
        "Achievements display",
        "How achievements are shown on the game detail page",
        row(AchievementsDisplay::all().iter().copied().map(|d| {
            let active = settings.achievements_display == d;
            option_chip(
                &d.to_string(),
                active,
                Palette::ACCENT(),
                SettingsMessage::AchievementsDisplay(d),
            )
        }))
        .spacing(4),
    );

    // Download covers auto
    let download_covers_row = setting_toggle(
        "Auto-download covers on import",
        "Automatically download game cover images after importing",
        settings.download_covers_auto,
        SettingsMessage::DownloadCoversAuto.into(),
    );

    column![
        ui_text("General", 18).color(Palette::TEXT_PRIMARY()),
        row![default_status_row, default_platform_row].spacing(12),
        rule::horizontal(1),
        row![confirm_delete_row, start_screen_row].spacing(12),
        rule::horizontal(1),
        row![sort_order_row, remember_filters_row].spacing(12),
        rule::horizontal(1),
        row![show_descriptions_row, achievements_display_row].spacing(12),
        download_covers_row,
        rule::horizontal(1),
        row![notifications_row, toast_duration_row].spacing(12),
        date_format_row,
    ]
    .spacing(16)
    .into()
}

// ───── Appearances ─────

fn appearances_section(app: &Spotter) -> Element<'_, Message> {
    let settings = &app.settings;

    // Compact list
    let compact_row = setting_toggle(
        "Compact library list",
        "Reduce spacing between games in the library view",
        settings.compact_list,
        SettingsMessage::CompactList.into(),
    );

    // Show covers
    let covers_row = setting_toggle(
        "Show cover images in library",
        "Display game cover thumbnails alongside titles",
        settings.show_covers_in_list,
        SettingsMessage::ShowCovers.into(),
    );

    // Sidebar width
    let sidebar_row = setting_row(
        "Sidebar width",
        "Adjust the width of the navigation sidebar",
        row![
            option_chip(
                "Narrow (180)",
                settings.sidebar_width == 180,
                Palette::ACCENT(),
                SettingsMessage::SidebarWidth(180)
            ),
            option_chip(
                "Normal (220)",
                settings.sidebar_width == 220,
                Palette::ACCENT(),
                SettingsMessage::SidebarWidth(220)
            ),
            option_chip(
                "Wide (280)",
                settings.sidebar_width == 280,
                Palette::ACCENT(),
                SettingsMessage::SidebarWidth(280)
            ),
        ]
        .spacing(4),
    );

    let kit = shared_preferences(app);
    column![
        ui_text("Appearances", 18).color(Palette::TEXT_PRIMARY()),
        kit.theme_section(),
        kit.colors_section(),
        kit.typography_section(),
        kit.effects_section(),
        rule::horizontal(1),
        sidebar_row,
        row![compact_row, covers_row].spacing(12),
    ]
    .spacing(16)
    .into()
}

// ───── Accessibility ─────

fn accessibility_section(app: &Spotter) -> Element<'_, Message> {
    let settings = &app.settings;

    let status_labels_row = setting_toggle(
        "Show status labels",
        "Display text labels next to status indicators instead of color only",
        settings.show_status_labels,
        SettingsMessage::ShowStatusLabels.into(),
    );

    let large_targets_row = setting_toggle(
        "Large click targets",
        "Increase button and clickable area sizes for easier interaction",
        settings.large_click_targets,
        SettingsMessage::LargeTargets.into(),
    );

    let kit = shared_preferences(app);
    column![
        ui_text("Accessibility", 18).color(Palette::TEXT_PRIMARY()),
        kit.vision_section(),
        kit.motion_section(),
        kit.reading_section(),
        rule::horizontal(1),
        row![status_labels_row, large_targets_row].spacing(12),
    ]
    .spacing(16)
    .into()
}

// ───── Helpers ─────

/// The shared Colony preferences sections, bound to Spotter's messages.
fn shared_preferences(
    app: &Spotter,
) -> PreferencesView<'_, fn(Change) -> Message, fn(&'static str) -> Message> {
    PreferencesView::new(
        Typography::current(),
        &app.settings.standard,
        &app.preference_sections,
        Message::Preference,
        Message::TogglePreferenceSection,
    )
}

fn setting_row<'a>(
    title: &'a str,
    description: &'a str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let label_col = column![
        ui_text(title, 14).color(Palette::TEXT_PRIMARY()),
        ui_text(description, 11).color(Palette::TEXT_MUTED()),
    ]
    .spacing(2)
    .width(Length::Fill);

    container(column![label_col, control.into(),].spacing(8))
        .padding([10, 12])
        .width(Length::Fill)
        .style(theme::card_style(
            Palette::BG_PRIMARY(),
            Palette::BORDER_SUBTLE(),
            6.0,
        ))
        .into()
}

fn setting_toggle<'a>(
    title: &'a str,
    description: &'a str,
    value: bool,
    msg: Message,
) -> Element<'a, Message> {
    let label_col = column![
        ui_text(title, 14).color(Palette::TEXT_PRIMARY()),
        ui_text(description, 11).color(Palette::TEXT_MUTED()),
    ]
    .spacing(2)
    .width(Length::Fill);

    let toggle = toggler(value).on_toggle(move |_| msg.clone()).size(sz(20));

    container(
        row![label_col, toggle]
            .spacing(12)
            .align_y(iced::Alignment::Center),
    )
    .padding([10, 12])
    .width(Length::Fill)
    .style(theme::card_style(
        Palette::BG_PRIMARY(),
        Palette::BORDER_SUBTLE(),
        6.0,
    ))
    .into()
}

fn option_chip(
    label: &str,
    active: bool,
    accent: Color,
    msg: impl Into<Message>,
) -> Element<'static, Message> {
    button(ui_text(label.to_owned(), 12))
        .on_press(msg.into())
        .padding([5, 10])
        .style(theme::chip_style(active, accent, 6.0))
        .into()
}
