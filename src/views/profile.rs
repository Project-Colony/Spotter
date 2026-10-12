use colony_ui::theme::Palette;
use colony_ui::{sz, ui_font};
use iced::widget::{button, column, container, row, rule, space, text_input};
use iced::{Background, Border, Color, Element, Length, Theme};

use super::helpers::{platform_dot, separator};
use crate::app::{Message, Spotter};
use crate::i18n::{decimal, t, tf, tn};
use crate::models::{GameStatus, Platform};

/// The backup formats, by name.
const JSON: &str = "JSON";
const CSV: &str = "CSV";
use crate::theme::{self, ui_text};

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let header = column![
        ui_text(
            format!("{} {}", theme::icons::PROFILE, t("nav_profile")),
            24
        )
        .font(colony_ui::ui_font_bold())
        .color(Palette::TEXT_PRIMARY()),
        ui_text(t("profile_subtitle"), 12).color(Palette::TEXT_MUTED()),
    ]
    .spacing(4);

    // ── Profile overview card ──
    let profile_card = profile_overview_card(app);

    // ── Platform note cards (credentials are managed on Import page) ──
    let steam_note = platform_note_card(Platform::Steam);
    let gog_note = platform_note_card(Platform::Gog);
    let epic_note = platform_note_card(Platform::Epic);
    let xbox_note = platform_note_card(Platform::Xbox);
    let psn_note = platform_note_card(Platform::PlayStation);

    // ── Data & export card ──
    let data_card = data_storage_card(app);

    let content = column![
        header,
        rule::horizontal(1),
        profile_card,
        row![steam_note, gog_note, epic_note].spacing(12),
        row![xbox_note, psn_note].spacing(12),
        crate::keyring::is_unavailable()
            .then(|| { ui_text(t("profile_no_keyring"), 12).color(Palette::WARNING()) }),
        data_card,
    ]
    .spacing(16)
    .padding(24)
    .width(Length::Fill);

    iced::widget::scrollable(container(content).width(Length::Fill).height(Length::Fill)).into()
}

// ── Profile overview card ──

fn profile_overview_card<'a>(app: &'a Spotter) -> Element<'a, Message> {
    let accent = Palette::ACCENT();

    let total_playtime: u32 = app.games.iter().map(|g| g.playtime_minutes).sum();
    let completed = app
        .games
        .iter()
        .filter(|g| g.status == GameStatus::Completed)
        .count();
    let avg_rating: f32 = {
        let (count, sum) = app
            .games
            .iter()
            .filter_map(|g| g.rating)
            .fold((0u32, 0u32), |(c, s), r| (c + 1, s + r as u32));
        if count == 0 {
            0.0
        } else {
            sum as f32 / count as f32
        }
    };

    // Avatar
    let initial = app
        .profile
        .username
        .chars()
        .next()
        .unwrap_or('P')
        .to_uppercase()
        .to_string();

    let avatar = container(ui_text(initial, 38).color(accent))
        .width(80)
        .height(80)
        .center_x(80)
        .center_y(80)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(accent.scale_alpha(0.12))),
            border: Border {
                radius: 40.0.into(),
                color: accent.scale_alpha(0.3),
                width: 2.0,
            },
            ..container::Style::default()
        });

    // Username input inline
    let username_section = column![
        ui_text(&app.profile.username, 24).color(Palette::TEXT_PRIMARY()),
        ui_text(
            tf(
                "profile_member_since",
                &[(
                    "date",
                    &app.preferences
                        .date_format
                        .format_date(&app.profile.member_since)
                )]
            ),
            12
        )
        .color(Palette::TEXT_MUTED()),
    ]
    .spacing(4);

    // Stats row
    let stats = row![
        stat_badge(
            t("stats_games"),
            format!("{}", app.games.len()),
            Palette::ACCENT()
        ),
        stat_badge(
            t("detail_playtime"),
            format_playtime(total_playtime),
            Palette::SUCCESS()
        ),
        stat_badge(
            t("profile_completed"),
            format!("{}", completed),
            GameStatus::Completed.color()
        ),
        stat_badge(
            t("stats_average_rating"),
            decimal(avg_rating, 1),
            Palette::WARNING()
        ),
    ]
    .spacing(8);

    // Username edit
    let username_input = column![
        ui_text(t("profile_username"), 12).color(Palette::TEXT_MUTED()),
        text_input(t("profile_username_placeholder"), &app.profile.username)
            .on_input(Message::UpdateUsername)
            .padding(10)
            .size(sz(13))
            .font(ui_font()),
    ]
    .spacing(4);

    let save_btn = if app.profile_just_saved {
        button(ui_text(t("profile_saved"), 14))
            .padding([10, 24])
            .style(theme::primary_btn_style(Palette::BTN_SUCCESS()))
    } else {
        button(ui_text(t("profile_save"), 14))
            .on_press(Message::SaveProfile)
            .padding([10, 24])
            .style(theme::primary_btn_style(accent))
    };

    let card_content = column![
        row![avatar, column![username_section, stats].spacing(10),]
            .spacing(16)
            .align_y(iced::Alignment::Center),
        separator(accent),
        username_input,
        save_btn,
    ]
    .spacing(12);

    container(card_content)
        .padding([16, 20])
        .width(Length::Fill)
        .style(theme::accent_card_style(accent))
        .into()
}

// ── Platform note card (credentials managed on Import page) ──

fn platform_note_card(platform: Platform) -> Element<'static, Message> {
    let card_content = row![
        platform_dot(platform.color()),
        ui_text(
            tf(
                "profile_platform_note",
                &[("platform", &platform.store_name())]
            ),
            12
        )
        .color(Palette::TEXT_MUTED()),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    container(card_content)
        .padding([10, 16])
        .width(Length::Fill)
        .style(theme::card_style(
            Palette::BG_CARD(),
            Palette::BORDER_SUBTLE(),
            10.0,
        ))
        .into()
}

// ── Data storage card ──

fn data_storage_card<'a>(app: &'a Spotter) -> Element<'a, Message> {
    let accent = Palette::ACCENT();

    let header_row = row![
        ui_text(t("profile_data"), 18).color(accent),
        space::horizontal(),
        ui_text(tn("profile_games_stored", app.games.len(), &[]), 12).color(Palette::TEXT_MUTED()),
    ]
    .align_y(iced::Alignment::Center);

    // The paths in use, resolved at startup (see `paths::layout`).
    let layout = crate::paths::layout();
    let paths_section: Element<'a, Message> = match &layout {
        Ok(layout) => column![
            data_row(t("settings_title"), &layout.preferences.to_string_lossy()),
            data_row(t("profile_database"), &layout.db.to_string_lossy()),
            data_row(t("profile_exports"), &layout.exports.to_string_lossy()),
            data_row(t("profile_covers"), &layout.covers.to_string_lossy()),
        ]
        .spacing(4)
        .into(),
        Err(e) => ui_text(e.clone(), 11).color(Palette::TEXT_DIM()).into(),
    };

    // Export/import buttons
    let export_json_btn = button(
        row![
            ui_text(JSON, 12).color(Palette::ACCENT()),
            ui_text(
                format!("{} {}", theme::icons::EXPORT, t("action_export")),
                12
            )
            .color(Palette::TEXT_PRIMARY()),
        ]
        .spacing(6),
    )
    .on_press(Message::ExportData)
    .padding([8, 16])
    .style(theme::outline_btn_style(Palette::ACCENT()));

    let export_csv_btn = button(
        row![
            ui_text(CSV, 12).color(Palette::WARNING()),
            ui_text(
                format!("{} {}", theme::icons::EXPORT, t("action_export")),
                12
            )
            .color(Palette::TEXT_PRIMARY()),
        ]
        .spacing(6),
    )
    .on_press(Message::ExportCsv)
    .padding([8, 16])
    .style(theme::outline_btn_style(Palette::WARNING()));

    let import_btn = button(
        row![
            ui_text(JSON, 12).color(Palette::SUCCESS()),
            ui_text(
                format!("{} {}", theme::icons::IMPORT, t("action_import")),
                12
            )
            .color(Palette::TEXT_PRIMARY()),
        ]
        .spacing(6),
    )
    .on_press(Message::ImportJson)
    .padding([8, 16])
    .style(theme::outline_btn_style(Palette::SUCCESS()));

    let backup_found = layout.is_ok_and(|l| l.exports.join("spotter_export.json").exists());
    let backup_note = t(if backup_found {
        "profile_backup_found"
    } else {
        "profile_backup_missing"
    });

    let card_content = column![
        header_row,
        separator(accent),
        paths_section,
        separator(accent),
        row![export_json_btn, export_csv_btn, import_btn].spacing(8),
        ui_text(backup_note, 11).color(Palette::TEXT_DIM()),
    ]
    .spacing(10);

    container(card_content)
        .padding([16, 20])
        .width(Length::Fill)
        .style(theme::accent_card_style(accent))
        .into()
}

// ── Shared helpers ──

fn stat_badge<'a>(label: &'a str, value: String, color: Color) -> Element<'a, Message> {
    let bg = color.scale_alpha(0.1);
    let border_color = color.scale_alpha(0.2);
    container(
        column![
            ui_text(value, 15).color(color),
            ui_text(label, 10).color(Palette::TEXT_DIM()),
        ]
        .spacing(2)
        .align_x(iced::Alignment::Center),
    )
    .padding([8, 14])
    .style(theme::pill_style(bg, border_color))
    .into()
}

fn data_row<'a>(label: &'a str, value: &str) -> Element<'a, Message> {
    row![
        ui_text(label, 12)
            .color(Palette::TEXT_MUTED())
            .width(colony_ui::sz(130)),
        ui_text(value.to_string(), 11).color(Palette::TEXT_DIM()),
    ]
    .spacing(8)
    .into()
}

fn format_playtime(minutes: u32) -> String {
    crate::models::format_playtime(minutes)
}
