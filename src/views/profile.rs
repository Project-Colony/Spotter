use colony_ui::theme::Palette;
use iced::widget::{button, column, container, row, rule, space, text, text_input};
use iced::{Background, Border, Color, Element, Length, Theme};

use super::helpers::{platform_dot, separator};
use crate::app::{Message, Spotter};
use crate::models::{GameStatus, Platform};
use crate::theme;

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let header = column![
        text(format!("{} User Profile", theme::icons::PROFILE))
            .size(24)
            .font(colony_ui::ui_font_bold())
            .color(Palette::TEXT_PRIMARY()),
        text("Manage your account and data")
            .size(12)
            .color(Palette::TEXT_MUTED()),
    ]
    .spacing(4);

    // ── Profile overview card ──
    let profile_card = profile_overview_card(app);

    // ── Platform note cards (credentials are managed on Import page) ──
    let steam_note = platform_note_card("Steam", Platform::Steam.color());
    let gog_note = platform_note_card("GOG Galaxy", Platform::Gog.color());
    let epic_note = platform_note_card("Epic Games", Platform::Epic.color());
    let xbox_note = platform_note_card("Xbox", Platform::Xbox.color());
    let psn_note = platform_note_card("PlayStation", Platform::PlayStation.color());

    // ── Data & export card ──
    let data_card = data_storage_card(app);

    let content = column![
        header,
        rule::horizontal(1),
        profile_card,
        row![steam_note, gog_note, epic_note].spacing(12),
        row![xbox_note, psn_note].spacing(12),
        crate::keyring::is_unavailable().then(|| {
            text("No system keyring available: platform tokens are stored unencrypted in Spotter's config folder.")
                .size(12)
                .color(Palette::WARNING())
        }),
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

    let avatar = container(text(initial).size(38).color(accent))
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
        text(&app.profile.username)
            .size(24)
            .color(Palette::TEXT_PRIMARY()),
        text(format!(
            "Member since {}",
            app.settings
                .date_format
                .format_date(&app.profile.member_since)
        ))
        .size(12)
        .color(Palette::TEXT_MUTED()),
    ]
    .spacing(4);

    // Stats row
    let stats = row![
        stat_badge("Games", format!("{}", app.games.len()), Palette::ACCENT()),
        stat_badge(
            "Playtime",
            format_playtime(total_playtime),
            Palette::SUCCESS()
        ),
        stat_badge(
            "Completed",
            format!("{}", completed),
            GameStatus::Completed.color()
        ),
        stat_badge(
            "Avg Rating",
            format!("{:.1}", avg_rating),
            Palette::WARNING()
        ),
    ]
    .spacing(8);

    // Username edit
    let username_input = column![
        text("Username").size(12).color(Palette::TEXT_MUTED()),
        text_input("Enter your username", &app.profile.username)
            .on_input(Message::UpdateUsername)
            .padding(10)
            .size(13),
    ]
    .spacing(4);

    let save_btn = if app.profile_just_saved {
        button(text("Saved!").size(14))
            .padding([10, 24])
            .style(theme::primary_btn_style(Palette::BTN_SUCCESS()))
    } else {
        button(text("Save Profile").size(14))
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

fn platform_note_card(name: &str, color: Color) -> Element<'static, Message> {
    let card_content = row![
        platform_dot(color),
        text(format!("{} — managed on Import page", name))
            .size(12)
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
        text("Data & Export").size(18).color(accent),
        space::horizontal(),
        text(format!("{} games stored", app.games.len()))
            .size(12)
            .color(Palette::TEXT_MUTED()),
    ]
    .align_y(iced::Alignment::Center);

    // The paths in use, resolved at startup (see `paths::layout`).
    let layout = crate::paths::layout();
    let paths_section: Element<'a, Message> = match &layout {
        Ok(layout) => column![
            data_row("Preferences", &layout.preferences.to_string_lossy()),
            data_row("Database", &layout.db.to_string_lossy()),
            data_row("Exports", &layout.exports.to_string_lossy()),
            data_row("Covers", &layout.covers.to_string_lossy()),
        ]
        .spacing(4)
        .into(),
        Err(e) => text(e.clone()).size(11).color(Palette::TEXT_DIM()).into(),
    };

    // Export/import buttons
    let export_json_btn = button(
        row![
            text("JSON").size(12).color(Palette::ACCENT()),
            text(format!("{} Export", theme::icons::EXPORT))
                .size(12)
                .color(Palette::TEXT_PRIMARY()),
        ]
        .spacing(6),
    )
    .on_press(Message::ExportData)
    .padding([8, 16])
    .style(theme::outline_btn_style(Palette::ACCENT()));

    let export_csv_btn = button(
        row![
            text("CSV").size(12).color(Palette::WARNING()),
            text(format!("{} Export", theme::icons::EXPORT))
                .size(12)
                .color(Palette::TEXT_PRIMARY()),
        ]
        .spacing(6),
    )
    .on_press(Message::ExportCsv)
    .padding([8, 16])
    .style(theme::outline_btn_style(Palette::WARNING()));

    let import_btn = button(
        row![
            text("JSON").size(12).color(Palette::SUCCESS()),
            text(format!("{} Import", theme::icons::IMPORT))
                .size(12)
                .color(Palette::TEXT_PRIMARY()),
        ]
        .spacing(6),
    )
    .on_press(Message::ImportJson)
    .padding([8, 16])
    .style(theme::outline_btn_style(Palette::SUCCESS()));

    let backup_found = layout.is_ok_and(|l| l.exports.join("spotter_export.json").exists());
    let backup_note = if backup_found {
        "Backup file found. Import will merge with your current library."
    } else {
        "No backup file found. Export first to create one."
    };

    let card_content = column![
        header_row,
        separator(accent),
        paths_section,
        separator(accent),
        row![export_json_btn, export_csv_btn, import_btn].spacing(8),
        text(backup_note).size(11).color(Palette::TEXT_DIM()),
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
            text(value).size(15).color(color),
            text(label).size(10).color(Palette::TEXT_DIM()),
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
        text(label).size(12).color(Palette::TEXT_MUTED()).width(100),
        text(value.to_string()).size(11).color(Palette::TEXT_DIM()),
    ]
    .spacing(8)
    .into()
}

fn format_playtime(minutes: u32) -> String {
    crate::models::format_playtime(minutes)
}
