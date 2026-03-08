use iced::widget::{button, column, container, row, rule, text, toggler};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::{Message, Spotter};
use crate::messages::SettingsMessage;
use crate::models::*;
use crate::theme::{self, ViewTheme};

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let vt = ViewTheme::from_settings(&app.settings);

    let saved_indicator: Element<'_, Message> = if app.settings_just_saved {
        text("Saved!").size(12).color(theme::SUCCESS).into()
    } else {
        text("").size(1).into()
    };
    let header = column![
        row![
            text(format!("{} Settings", theme::icons::SETTINGS))
                .size(24)
                .font(theme::FONT_BOLD)
                .color(Color::WHITE),
            saved_indicator,
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
        text("Customize your Spotter experience")
            .size(12)
            .color(vt.text_muted),
    ]
    .spacing(4);

    let accent = app.settings.accent_color.color();

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
        SettingsSection::General => general_section(app, vt),
        SettingsSection::Appearances => appearances_section(app, vt),
        SettingsSection::Accessibility => accessibility_section(app, vt),
    };

    let bg_card = vt.bg_card;
    let border = vt.border;
    let card = container(column![tab_bar, rule::horizontal(1), section_content,].spacing(16))
        .padding(20)
        .width(Length::Fill)
        .style(theme::card_style(bg_card, border, 8.0));

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
    let is_active = current == target;
    let color = if is_active { accent } else { theme::TEXT_MUTED };

    button(text(label).size(14).color(color))
        .on_press(SettingsMessage::ChangeSection(target).into())
        .padding([8, 16])
        .style(move |_: &Theme, _| {
            let bg = if is_active {
                Color::from_rgb(0.15, 0.15, 0.22)
            } else {
                Color::TRANSPARENT
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: color,
                border: Border {
                    radius: 8.0.into(),
                    color: if is_active {
                        Color { a: 0.4, ..color }
                    } else {
                        Color::TRANSPARENT
                    },
                    width: 1.0,
                },
                ..button::Style::default()
            }
        })
        .into()
}

// ───── General ─────

fn general_section(app: &Spotter, vt: ViewTheme) -> Element<'_, Message> {
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
        vt,
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
        vt,
    );

    // Confirm before delete
    let confirm_delete_row = setting_toggle(
        "Confirm before deleting",
        "Show a confirmation dialog before removing a game",
        settings.confirm_before_delete,
        SettingsMessage::ConfirmDelete.into(),
        vt,
    );

    // Start screen
    let start_screen_row = setting_row(
        "Start screen",
        "Which screen to show when Spotter launches",
        row![
            option_chip(
                "Library",
                settings.start_screen == StartScreen::Library,
                theme::ACCENT_BLUE,
                SettingsMessage::StartScreen(StartScreen::Library)
            ),
            option_chip(
                "Statistics",
                settings.start_screen == StartScreen::Statistics,
                theme::ACCENT_BLUE,
                SettingsMessage::StartScreen(StartScreen::Statistics)
            ),
            option_chip(
                "Import",
                settings.start_screen == StartScreen::Import,
                theme::ACCENT_BLUE,
                SettingsMessage::StartScreen(StartScreen::Import)
            ),
            option_chip(
                "Profile",
                settings.start_screen == StartScreen::Profile,
                theme::ACCENT_BLUE,
                SettingsMessage::StartScreen(StartScreen::Profile)
            ),
        ]
        .spacing(4),
        vt,
    );

    // Notifications
    let notifications_row = setting_toggle(
        "Show notifications",
        "Display success toasts after actions (errors always show)",
        settings.notifications_enabled,
        SettingsMessage::Notifications.into(),
        vt,
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
                theme::ACCENT_BLUE,
                SettingsMessage::ToastDuration(d),
            )
        }))
        .spacing(4),
        vt,
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
                theme::ACCENT_BLUE,
                SettingsMessage::DateFormat(f),
            )
        }))
        .spacing(4),
        vt,
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
                theme::ACCENT_BLUE,
                SettingsMessage::DefaultSortOrder(s),
            )
        }))
        .spacing(4)
        .wrap(),
        vt,
    );

    // Remember filters
    let remember_filters_row = setting_toggle(
        "Remember filters",
        "Persist status and platform filters across sessions",
        settings.remember_filters,
        SettingsMessage::RememberFilters.into(),
        vt,
    );

    // Show game descriptions
    let show_descriptions_row = setting_toggle(
        "Show descriptions in library",
        "Display game descriptions in library card view",
        settings.show_game_descriptions,
        SettingsMessage::ShowDescriptions.into(),
        vt,
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
                theme::ACCENT_BLUE,
                SettingsMessage::AchievementsDisplay(d),
            )
        }))
        .spacing(4),
        vt,
    );

    // Download covers auto
    let download_covers_row = setting_toggle(
        "Auto-download covers on import",
        "Automatically download game cover images after importing",
        settings.download_covers_auto,
        SettingsMessage::DownloadCoversAuto.into(),
        vt,
    );

    column![
        text("General").size(18).color(theme::TEXT_LIGHT),
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

fn appearances_section(app: &Spotter, vt: ViewTheme) -> Element<'_, Message> {
    let settings = &app.settings;

    // Theme mode
    let theme_row = setting_row(
        "Theme",
        "Controls the overall darkness level of the interface",
        row(ThemeMode::all().iter().copied().map(|t| {
            let active = settings.theme_mode == t;
            option_chip(
                &t.to_string(),
                active,
                theme::ACCENT_BLUE,
                SettingsMessage::ThemeMode(t),
            )
        }))
        .spacing(4),
        vt,
    );

    // Accent color
    let accent_row = setting_row(
        "Accent color",
        "Highlight color used throughout the interface",
        row(AccentColor::all().iter().copied().map(|c| {
            let active = settings.accent_color == c;
            color_swatch(
                &c.to_string(),
                c.color(),
                active,
                SettingsMessage::AccentColor(c),
            )
        }))
        .spacing(4),
        vt,
    );

    // UI scale
    let scale_row = setting_row(
        "UI scale",
        "Adjust text and element sizes",
        row(UiScale::all().iter().copied().map(|s| {
            let active = settings.ui_scale == s;
            let label = match s {
                UiScale::Small => "Small (85%)",
                UiScale::Normal => "Normal (100%)",
                UiScale::Large => "Large (120%)",
            };
            option_chip(
                label,
                active,
                theme::ACCENT_BLUE,
                SettingsMessage::UiScale(s),
            )
        }))
        .spacing(4),
        vt,
    );

    // Compact list
    let compact_row = setting_toggle(
        "Compact library list",
        "Reduce spacing between games in the library view",
        settings.compact_list,
        SettingsMessage::CompactList.into(),
        vt,
    );

    // Show covers
    let covers_row = setting_toggle(
        "Show cover images in library",
        "Display game cover thumbnails alongside titles",
        settings.show_covers_in_list,
        SettingsMessage::ShowCovers.into(),
        vt,
    );

    // Sidebar width
    let sidebar_row = setting_row(
        "Sidebar width",
        "Adjust the width of the navigation sidebar",
        row![
            option_chip(
                "Narrow (180)",
                settings.sidebar_width == 180,
                theme::ACCENT_BLUE,
                SettingsMessage::SidebarWidth(180)
            ),
            option_chip(
                "Normal (220)",
                settings.sidebar_width == 220,
                theme::ACCENT_BLUE,
                SettingsMessage::SidebarWidth(220)
            ),
            option_chip(
                "Wide (280)",
                settings.sidebar_width == 280,
                theme::ACCENT_BLUE,
                SettingsMessage::SidebarWidth(280)
            ),
        ]
        .spacing(4),
        vt,
    );

    column![
        text("Appearances").size(18).color(theme::TEXT_LIGHT),
        row![theme_row, accent_row].spacing(12),
        rule::horizontal(1),
        row![scale_row, sidebar_row].spacing(12),
        rule::horizontal(1),
        row![compact_row, covers_row].spacing(12),
    ]
    .spacing(16)
    .into()
}

// ───── Accessibility ─────

fn accessibility_section(app: &Spotter, vt: ViewTheme) -> Element<'_, Message> {
    let settings = &app.settings;

    let high_contrast_row = setting_toggle(
        "High contrast mode",
        "Increase contrast for text and borders to improve readability",
        settings.high_contrast,
        SettingsMessage::HighContrast.into(),
        vt,
    );

    let status_labels_row = setting_toggle(
        "Show status labels",
        "Display text labels next to status indicators instead of color only",
        settings.show_status_labels,
        SettingsMessage::ShowStatusLabels.into(),
        vt,
    );

    let large_targets_row = setting_toggle(
        "Large click targets",
        "Increase button and clickable area sizes for easier interaction",
        settings.large_click_targets,
        SettingsMessage::LargeTargets.into(),
        vt,
    );

    column![
        text("Accessibility").size(18).color(theme::TEXT_LIGHT),
        row![high_contrast_row, status_labels_row].spacing(12),
        large_targets_row,
    ]
    .spacing(16)
    .into()
}

// ───── Helpers ─────

fn setting_row<'a>(
    title: &'a str,
    description: &'a str,
    control: impl Into<Element<'a, Message>>,
    vt: ViewTheme,
) -> Element<'a, Message> {
    let label_col = column![
        text(title).size(14).color(Color::WHITE),
        text(description).size(11).color(vt.text_muted),
    ]
    .spacing(2)
    .width(Length::Fill);

    let bg = vt.bg_row;
    let border = vt.border;
    container(column![label_col, control.into(),].spacing(8))
        .padding([10, 12])
        .width(Length::Fill)
        .style(theme::card_style(bg, border, 6.0))
        .into()
}

fn setting_toggle<'a>(
    title: &'a str,
    description: &'a str,
    value: bool,
    msg: Message,
    vt: ViewTheme,
) -> Element<'a, Message> {
    let label_col = column![
        text(title).size(14).color(Color::WHITE),
        text(description).size(11).color(vt.text_muted),
    ]
    .spacing(2)
    .width(Length::Fill);

    let toggle = toggler(value).on_toggle(move |_| msg.clone()).size(20.0);

    let bg = vt.bg_row;
    let border = vt.border;
    container(
        row![label_col, toggle]
            .spacing(12)
            .align_y(iced::Alignment::Center),
    )
    .padding([10, 12])
    .width(Length::Fill)
    .style(theme::card_style(bg, border, 6.0))
    .into()
}

fn option_chip(
    label: &str,
    active: bool,
    accent: Color,
    msg: impl Into<Message>,
) -> Element<'static, Message> {
    let color = if active { accent } else { theme::TEXT_MUTED };
    let msg: Message = msg.into();

    button(text(label.to_owned()).size(12).color(color))
        .on_press(msg)
        .padding([5, 10])
        .style(theme::chip_style(active, color, accent, 6.0))
        .into()
}

fn color_swatch(
    label: &str,
    color: Color,
    active: bool,
    msg: impl Into<Message>,
) -> Element<'static, Message> {
    let msg: Message = msg.into();
    let dot = container(text("").size(1))
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: 8.0.into(),
                ..Border::default()
            },
            ..container::Style::default()
        });

    let lbl = text(label.to_owned())
        .size(12)
        .color(if active { color } else { theme::TEXT_MUTED });

    button(row![dot, lbl].spacing(6).align_y(iced::Alignment::Center))
        .on_press(msg)
        .padding([5, 10])
        .style(move |_: &Theme, _| {
            let bg = if active {
                Color::from_rgb(0.14, 0.14, 0.20)
            } else {
                theme::BG_BUTTON
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: color,
                border: Border {
                    radius: 6.0.into(),
                    color: if active {
                        Color { a: 0.4, ..color }
                    } else {
                        Color::TRANSPARENT
                    },
                    width: 1.0,
                },
                ..button::Style::default()
            }
        })
        .into()
}
