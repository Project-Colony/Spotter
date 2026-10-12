use colony_ui::theme::Palette;
use colony_ui::{sz, ui_font};
use iced::widget::{button, column, container, row, rule, space, text_input};
use iced::{Background, Border, Color, Element, Length, Theme};

use super::helpers::{platform_dot, separator, status_badge};
use crate::app::{Message, Spotter};
use crate::i18n::{t, tf, tn};
use crate::messages::AuthMessage;
use crate::models::Platform;
use crate::theme::{self, ui_text};

/// Where Steam hands out Web API keys.
const STEAM_KEY_PAGE: &str = "steamcommunity.com/dev/apikey";

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let accent = Palette::ACCENT();

    let header = column![
        ui_text(
            format!("{} {}", theme::icons::IMPORT, t("import_title")),
            24
        )
        .font(colony_ui::ui_font_bold())
        .color(Palette::TEXT_PRIMARY()),
        ui_text(t("import_subtitle"), 12).color(Palette::TEXT_MUTED()),
    ]
    .spacing(4);

    // ── Import status banner ──
    let status_section = import_status_banner(app);

    // ── Row 1: Steam | GOG ──
    let steam_card = steam_setup_card(app);
    let gog_card = gog_setup_card(app);

    // ── Row 2: Epic | Xbox ──
    let epic_card = epic_setup_card(app);

    let xbox_card = xbox_setup_card(app);

    // ── Row 3: PlayStation | Nintendo ──
    let psn_card = psn_setup_card(app);

    let nintendo_card = nintendo_info_card();

    // ── Help section ──
    let help_card = help_section(accent);

    // Import history log
    let history_section = import_history_card(app);

    let content = column![
        header,
        rule::horizontal(1),
        status_section,
        row![steam_card, gog_card]
            .spacing(12)
            .align_y(iced::Alignment::Start),
        row![epic_card, xbox_card]
            .spacing(12)
            .align_y(iced::Alignment::Start),
        row![psn_card, nintendo_card]
            .spacing(12)
            .align_y(iced::Alignment::Start),
        history_section,
        help_card,
    ]
    .spacing(16)
    .padding(24)
    .width(Length::Fill);

    iced::widget::scrollable(container(content).width(Length::Fill).height(Length::Fill)).into()
}

// ── Status banner ──

fn import_status_banner<'a>(app: &'a Spotter) -> Element<'a, Message> {
    let any_importing = !app.importing.is_empty();

    let status = match &app.import_status {
        Ok(text) if text.is_empty() => None,
        Ok(text) => Some((text, false)),
        Err(text) => Some((text, true)),
    };
    if !any_importing && status.is_none() {
        return column![].into();
    }

    let mut content = column![].spacing(6);

    if any_importing {
        let platforms: Vec<&String> = app.importing.iter().collect();
        let label = tf(
            "import_from",
            &[(
                "platform",
                &platforms
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            )],
        );
        // The dots move between frames: an animation, so still without effects.
        let phase = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            / 500;
        let dots = match phase % 4 {
            _ if !colony_ui::effects_enabled() => "...",
            0 => ".",
            1 => "..",
            2 => "...",
            _ => "",
        };
        let spinner = ui_text(dots.to_string(), 14).color(Palette::WARNING());
        let msg = ui_text(label, 14).color(Palette::WARNING());
        content = content.push(
            row![spinner, msg]
                .spacing(10)
                .align_y(iced::Alignment::Center),
        );
    }

    if let Some((text, is_error)) = status.filter(|_| !any_importing) {
        let (icon, color) = if is_error {
            (theme::icons::TIMES, Palette::ERROR())
        } else {
            (theme::icons::CHECK, Palette::SUCCESS())
        };
        let badge = container(ui_text(icon, 11).color(Palette::TEXT_PRIMARY()))
            .padding([2, 8])
            .style(theme::pill_style(
                color.scale_alpha(0.2),
                color.scale_alpha(0.4),
            ));
        content = content.push(
            row![badge, ui_text(text.as_str(), 13).color(color)]
                .spacing(10)
                .align_y(iced::Alignment::Center),
        );
    }

    let (banner_bg, banner_border) = if any_importing {
        (Palette::WARNING_BG(), Palette::WARNING().scale_alpha(0.4))
    } else {
        (Palette::BG_CARD(), Palette::BORDER_SUBTLE())
    };

    container(content)
        .padding([12, 16])
        .width(Length::Fill)
        .style(theme::card_style(banner_bg, banner_border, 10.0))
        .into()
}

// ── Steam setup card ──

fn steam_setup_card(app: &Spotter) -> Element<'_, Message> {
    let color = Platform::Steam.color();
    let has_steam_id = !app.profile.steam_id.is_empty();
    let has_api_key = !app.profile.steam_api_key.is_empty();
    let ready = has_steam_id && has_api_key;
    let steam_importing = app.importing.contains("Steam");

    // Header row: platform dot + title + status pill
    let status_pill = if ready {
        status_badge(t("import_ready"), Palette::SUCCESS())
    } else if has_steam_id {
        status_badge(t("import_key_needed"), Palette::WARNING())
    } else {
        status_badge(t("import_setup_needed"), Palette::ERROR())
    };

    let header_row = row![
        platform_dot(color),
        ui_text(Platform::Steam.store_name(), 18).color(color),
        space::horizontal(),
        status_pill,
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // Step 1: Login
    let step1_label = step_label("1", t("import_step_login"), color);
    let login_status: Element<'_, Message> = if has_steam_id {
        row![
            ui_text(t("import_steam_id"), 12).color(Palette::TEXT_MUTED()),
            ui_text(&app.profile.steam_id, 12).color(Palette::SUCCESS()),
        ]
        .spacing(4)
        .into()
    } else {
        ui_text(t("import_not_logged_in"), 12)
            .color(Palette::TEXT_DIM())
            .into()
    };

    let login_btn = if app.steam_login_active {
        button(ui_text(t("import_waiting"), 12).color(Palette::WARNING()))
            .padding([6, 14])
            .style(theme::loading_btn_style())
    } else {
        button(ui_text(t("import_login"), 12))
            .on_press(AuthMessage::SteamLogin.into())
            .padding([6, 14])
            .style(theme::primary_btn_style(color))
    };

    // Step 2: API Key
    let step2_label = step_label("2", t("import_step_api_key"), color);
    let api_key_input = column![
        text_input(
            t("import_steam_key_placeholder"),
            &app.profile.steam_api_key
        )
        .on_input(Message::UpdateSteamApiKey)
        .padding(8)
        .size(sz(12))
        .font(ui_font()),
        ui_text(STEAM_KEY_PAGE, 10).color(Palette::TEXT_DIM()),
    ]
    .spacing(4);

    // Import button
    let import_btn = if ready && !steam_importing {
        button(ui_text(import_label(Platform::Steam), 13))
            .on_press(Message::ImportSteam)
            .padding([8, 18])
            .style(theme::primary_btn_style(Palette::BTN_SUCCESS()))
    } else if steam_importing {
        button(ui_text(t("import_importing"), 13).color(Palette::WARNING()))
            .padding([8, 18])
            .style(theme::loading_btn_style())
    } else {
        button(ui_text(import_label(Platform::Steam), 13).color(Palette::TEXT_DIMMER()))
            .padding([8, 18])
            .style(theme::disabled_btn_style())
    };

    let card_content = column![
        header_row,
        separator(color),
        step1_label,
        row![login_btn, login_status]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        separator(color),
        step2_label,
        api_key_input,
        separator(color),
        import_btn,
    ]
    .spacing(8);

    container(card_content)
        .padding([14, 16])
        .width(Length::Fill)
        .style(theme::accent_card_style(color))
        .into()
}

// ── GOG setup card ──

fn gog_setup_card(app: &Spotter) -> Element<'_, Message> {
    let color = Platform::Gog.color();
    let has_token = !app.profile.gog_token.is_empty();
    let gog_importing = app.importing.contains("GOG");

    let status_pill = if has_token {
        status_badge(t("import_ready"), Palette::SUCCESS())
    } else if app.gog_login_active {
        status_badge(t("import_logging_in"), Palette::WARNING())
    } else {
        status_badge(t("import_setup_needed"), Palette::ERROR())
    };

    let header_row = row![
        platform_dot(color),
        ui_text(Platform::Gog.store_name(), 18).color(color),
        space::horizontal(),
        status_pill,
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // Step 1: Login
    let step1_label = step_label("1", t("import_step_login"), color);
    let login_status: Element<'_, Message> = if has_token {
        ui_text(t("import_logged_in"), 12)
            .color(Palette::SUCCESS())
            .into()
    } else if app.gog_login_active {
        ui_text(t("import_waiting_login"), 12)
            .color(Palette::WARNING())
            .into()
    } else {
        ui_text(t("import_not_logged_in"), 12)
            .color(Palette::TEXT_DIM())
            .into()
    };

    let login_btn = if app.gog_login_active {
        button(ui_text(t("import_waiting"), 12).color(Palette::WARNING()))
            .padding([6, 14])
            .style(theme::loading_btn_style())
    } else {
        button(ui_text(t("import_login"), 12))
            .on_press(AuthMessage::GogLogin.into())
            .padding([6, 14])
            .style(theme::primary_btn_style(color))
    };

    // Step 2: Paste code
    let code_section: Element<'_, Message> = if app.gog_login_active || !has_token {
        let step2_label = step_label("2", t("import_step_paste_address"), color);
        let submit_btn = if !app.gog_code_input.is_empty() {
            button(ui_text(t("import_finish_login"), 12))
                .on_press(AuthMessage::GogSubmitCode.into())
                .padding([6, 14])
                .style(theme::primary_btn_style(Palette::BTN_SUCCESS()))
        } else {
            button(ui_text(t("import_finish_login"), 12).color(Palette::TEXT_DIMMER()))
                .padding([6, 14])
                .style(theme::disabled_btn_style())
        };

        column![
            separator(color),
            step2_label,
            ui_text(t("import_gog_hint"), 10).color(Palette::TEXT_DIM()),
            text_input(t("import_gog_placeholder"), &app.gog_code_input,)
                .on_input(|s| AuthMessage::GogCodeChanged(s).into())
                .padding(8)
                .size(sz(12))
                .font(ui_font()),
            submit_btn,
        ]
        .spacing(6)
        .into()
    } else {
        column![].into()
    };

    // Import button
    let import_btn = if has_token && !gog_importing {
        button(ui_text(import_label(Platform::Gog), 13))
            .on_press(Message::ImportGog)
            .padding([8, 18])
            .style(theme::primary_btn_style(Palette::BTN_SUCCESS()))
    } else if gog_importing {
        button(ui_text(t("import_importing"), 13).color(Palette::WARNING()))
            .padding([8, 18])
            .style(theme::loading_btn_style())
    } else {
        button(ui_text(import_label(Platform::Gog), 13).color(Palette::TEXT_DIMMER()))
            .padding([8, 18])
            .style(theme::disabled_btn_style())
    };

    let card_content = column![
        header_row,
        separator(color),
        step1_label,
        row![login_btn, login_status]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        code_section,
        separator(color),
        import_btn,
    ]
    .spacing(8);

    container(card_content)
        .padding([14, 16])
        .width(Length::Fill)
        .style(theme::accent_card_style(color))
        .into()
}

// ── Epic Games setup card ──

fn epic_setup_card(app: &Spotter) -> Element<'_, Message> {
    let color = Platform::Epic.color();
    let has_token = !app.profile.epic_token.is_empty();
    let has_account = !app.profile.epic_account_id.is_empty();
    let logged_in = has_token && has_account;
    let epic_importing = app.importing.contains("Epic");

    let status_pill = if logged_in {
        status_badge(t("import_connected"), Palette::SUCCESS())
    } else if app.epic_login_active {
        status_badge(t("import_logging_in"), Palette::WARNING())
    } else {
        status_badge(t("import_local_only"), Palette::WARNING())
    };

    let header_row = row![
        platform_dot(color),
        ui_text(Platform::Epic.store_name(), 18).color(color),
        space::horizontal(),
        status_pill,
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // Step 1: Login
    let step1_label = step_label("1", t("import_step_login"), color);
    let login_status: Element<'_, Message> = if logged_in {
        let name = if app.profile.epic_display_name.is_empty() {
            app.profile
                .epic_account_id
                .chars()
                .take(12)
                .collect::<String>()
                + "..."
        } else {
            app.profile.epic_display_name.clone()
        };
        row![
            ui_text(t("import_account"), 12).color(Palette::TEXT_MUTED()),
            ui_text(name, 12).color(Palette::SUCCESS()),
        ]
        .spacing(4)
        .into()
    } else if app.epic_login_active {
        ui_text(t("import_waiting_login"), 12)
            .color(Palette::WARNING())
            .into()
    } else {
        ui_text(t("import_epic_optional"), 12)
            .color(Palette::TEXT_DIM())
            .into()
    };

    let login_btn = if app.epic_login_active {
        button(ui_text(t("import_waiting"), 12).color(Palette::WARNING()))
            .padding([6, 14])
            .style(theme::loading_btn_style())
    } else if logged_in {
        button(ui_text(t("import_login_again"), 12))
            .on_press(AuthMessage::EpicLogin.into())
            .padding([6, 14])
            .style(theme::primary_btn_style(color))
    } else {
        button(ui_text(t("import_login"), 12))
            .on_press(AuthMessage::EpicLogin.into())
            .padding([6, 14])
            .style(theme::primary_btn_style(color))
    };

    // Step 2: Paste code (only shown when login is active or not logged in)
    let code_section: Element<'_, Message> = if app.epic_login_active || !logged_in {
        let step2_label = step_label("2", t("import_step_paste_code"), color);
        let submit_btn = if !app.epic_code_input.is_empty() {
            button(ui_text(t("import_finish_login"), 12))
                .on_press(AuthMessage::EpicSubmitCode.into())
                .padding([6, 14])
                .style(theme::primary_btn_style(Palette::BTN_SUCCESS()))
        } else {
            button(ui_text(t("import_finish_login"), 12).color(Palette::TEXT_DIMMER()))
                .padding([6, 14])
                .style(theme::disabled_btn_style())
        };

        column![
            separator(color),
            step2_label,
            ui_text(t("import_epic_hint"), 10).color(Palette::TEXT_DIM()),
            text_input(t("import_epic_placeholder"), &app.epic_code_input)
                .on_input(|s| AuthMessage::EpicCodeChanged(s).into())
                .padding(8)
                .size(sz(12))
                .font(ui_font()),
            submit_btn,
        ]
        .spacing(6)
        .into()
    } else {
        column![].into()
    };

    // Import button
    let import_label = t(if logged_in {
        "import_epic_full"
    } else {
        "import_epic_local"
    });

    let import_btn = if !epic_importing {
        button(ui_text(import_label, 13))
            .on_press(Message::ImportEpic)
            .padding([8, 18])
            .style(theme::primary_btn_style(if logged_in {
                Palette::BTN_SUCCESS()
            } else {
                color
            }))
    } else {
        button(ui_text(t("import_importing"), 13).color(Palette::WARNING()))
            .padding([8, 18])
            .style(theme::loading_btn_style())
    };

    let import_note = ui_text(
        t(if logged_in {
            "import_epic_full_note"
        } else {
            "import_epic_local_note"
        }),
        10,
    )
    .color(Palette::TEXT_DIM());

    let card_content = column![
        header_row,
        separator(color),
        step1_label,
        row![login_btn, login_status]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        code_section,
        separator(color),
        import_btn,
        import_note,
    ]
    .spacing(8);

    container(card_content)
        .padding([14, 16])
        .width(Length::Fill)
        .style(theme::accent_card_style(color))
        .into()
}

// ── Xbox setup card ──

fn xbox_setup_card(app: &Spotter) -> Element<'_, Message> {
    let color = Platform::Xbox.color();
    let has_key = !app.profile.xbox_api_key.is_empty();
    let has_tag = !app.profile.xbox_gamertag.is_empty();
    let xbox_importing = app.importing.contains("Xbox");

    let status_pill = if xbox_importing {
        status_badge(t("import_importing"), Palette::WARNING())
    } else if has_key {
        status_badge(t("import_ready"), Palette::SUCCESS())
    } else {
        status_badge(t("import_setup_needed"), Palette::ERROR())
    };

    let header_row = row![
        platform_dot(color),
        ui_text(Platform::Xbox.store_name(), 18).color(color),
        space::horizontal(),
        status_pill,
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // Step 1: API Key
    let step1_label = step_label("1", t("import_step_api_key"), color);
    let api_key_input = column![
        text_input(t("import_xbox_key_placeholder"), &app.profile.xbox_api_key)
            .on_input(Message::UpdateXboxApiKey)
            .padding(8)
            .size(sz(12))
            .font(ui_font()),
        button(ui_text(t("import_xbox_key_link"), 10).color(Palette::ACCENT()),)
            .on_press(Message::OpenUrl("https://xbl.io".to_string()))
            .padding(0)
            .style(theme::transparent_btn_style()),
    ]
    .spacing(4);

    // Step 2: Gamertag (shown when API key is entered)
    let gamertag_section: Element<'_, Message> = if has_key || has_tag {
        let step2_label = step_label("2", t("import_step_gamertag"), color);
        column![
            separator(color),
            step2_label,
            text_input(
                t("import_xbox_gamertag_placeholder"),
                &app.profile.xbox_gamertag
            )
            .on_input(Message::UpdateXboxGamertag)
            .padding(8)
            .size(sz(12))
            .font(ui_font()),
        ]
        .spacing(6)
        .into()
    } else {
        column![].into()
    };

    // Import button
    let import_btn = if has_key && !xbox_importing {
        button(ui_text(import_label(Platform::Xbox), 13))
            .on_press(Message::ImportXbox)
            .padding([8, 18])
            .style(theme::primary_btn_style(Palette::BTN_SUCCESS()))
    } else if xbox_importing {
        button(ui_text(t("import_importing"), 13).color(Palette::WARNING()))
            .padding([8, 18])
            .style(theme::loading_btn_style())
    } else {
        button(ui_text(import_label(Platform::Xbox), 13).color(Palette::TEXT_DIMMER()))
            .padding([8, 18])
            .style(theme::disabled_btn_style())
    };

    let card_content = column![
        header_row,
        separator(color),
        step1_label,
        api_key_input,
        gamertag_section,
        separator(color),
        import_btn,
    ]
    .spacing(8);

    container(card_content)
        .padding([14, 16])
        .width(Length::Fill)
        .style(theme::accent_card_style(color))
        .into()
}

// ── PlayStation setup card ──

fn psn_setup_card(app: &Spotter) -> Element<'_, Message> {
    let color = Platform::PlayStation.color();
    let has_token = !app.profile.psn_npsso.is_empty();
    let psn_importing = app.importing.contains("PlayStation");

    let status_pill = if psn_importing {
        status_badge(t("import_importing"), Palette::WARNING())
    } else if has_token {
        status_badge(t("import_ready"), Palette::SUCCESS())
    } else {
        status_badge(t("import_setup_needed"), Palette::ERROR())
    };

    let header_row = row![
        platform_dot(color),
        ui_text(Platform::PlayStation.store_name(), 18).color(color),
        space::horizontal(),
        status_pill,
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // Step 1: NPSSO Token
    let step1_label = step_label("1", t("import_step_npsso"), color);
    let npsso_input = column![
        text_input(t("import_psn_placeholder"), &app.profile.psn_npsso)
            .on_input(Message::UpdatePsnNpsso)
            .padding(8)
            .size(sz(12))
            .font(ui_font()),
        button(ui_text(t("import_psn_link"), 10).color(Palette::ACCENT()),)
            .on_press(Message::OpenUrl(
                "https://ca.account.sony.com/api/v1/ssocookie".to_string(),
            ))
            .padding(0)
            .style(theme::transparent_btn_style()),
    ]
    .spacing(4);

    // Import button
    let import_btn = if has_token && !psn_importing {
        button(ui_text(import_label(Platform::PlayStation), 13))
            .on_press(Message::ImportPlayStation)
            .padding([8, 18])
            .style(theme::primary_btn_style(Palette::BTN_SUCCESS()))
    } else if psn_importing {
        button(ui_text(t("import_importing"), 13).color(Palette::WARNING()))
            .padding([8, 18])
            .style(theme::loading_btn_style())
    } else {
        button(ui_text(import_label(Platform::PlayStation), 13).color(Palette::TEXT_DIMMER()))
            .padding([8, 18])
            .style(theme::disabled_btn_style())
    };

    let card_content = column![
        header_row,
        separator(color),
        step1_label,
        npsso_input,
        separator(color),
        import_btn,
    ]
    .spacing(8);

    container(card_content)
        .padding([14, 16])
        .width(Length::Fill)
        .style(theme::accent_card_style(color))
        .into()
}

// ── Nintendo info card (no importer available) ──

fn nintendo_info_card() -> Element<'static, Message> {
    let color = Platform::Nintendo.color();

    let header_row = row![
        platform_dot(color),
        ui_text(Platform::Nintendo.store_name(), 18).color(color),
        space::horizontal(),
        status_badge(t("import_manual_only"), Palette::TEXT_MUTED()),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let card_content = column![
        header_row,
        ui_text(t("import_nintendo_no_api"), 12).color(Palette::TEXT_MUTED()),
        ui_text(t("import_nintendo_manual"), 12).color(Palette::TEXT_MUTED()),
    ]
    .spacing(10);

    container(card_content)
        .padding([14, 18])
        .width(Length::Fill)
        .style(theme::accent_card_style(color))
        .into()
}

// ── Import history card ──

fn import_history_card<'a>(app: &'a Spotter) -> Element<'a, Message> {
    if app.import_history.is_empty() {
        return column![].into();
    }

    let accent = Palette::ACCENT();
    let mut log_col = column![].spacing(4);

    for entry in app.import_history.iter().rev().take(10) {
        let (text, color) = match entry {
            Ok(text) => (text, Palette::SUCCESS()),
            Err(text) => (text, Palette::ERROR()),
        };
        log_col = log_col.push(ui_text(text.as_str(), 12).color(color));
    }

    let card_content = column![
        row![
            ui_text(t("import_log"), 16).color(accent),
            space::horizontal(),
            ui_text(tn("import_log_entries", app.import_history.len(), &[]), 11)
                .color(Palette::TEXT_DIM()),
        ]
        .align_y(iced::Alignment::Center),
        separator(accent),
        log_col,
    ]
    .spacing(8);

    container(card_content)
        .padding([14, 18])
        .width(Length::Fill)
        .style(theme::accent_card_style(accent))
        .into()
}

// ── Help section ──

fn help_section<'a>(accent: Color) -> Element<'a, Message> {
    let tips = column(
        [
            (Platform::Steam, "import_tip_steam"),
            (Platform::Gog, "import_tip_gog"),
            (Platform::Epic, "import_tip_epic"),
            (Platform::Xbox, "import_tip_xbox"),
            (Platform::PlayStation, "import_tip_playstation"),
            (Platform::Nintendo, "import_tip_nintendo"),
        ]
        .map(|(platform, tip)| tip_row(platform, t(tip))),
    )
    .spacing(8);

    let notes = column![
        note_row(t("import_note_reimport")),
        note_row(t("import_note_covers")),
        note_row(t("import_note_backups")),
    ]
    .spacing(4);

    let card_content = column![
        row![
            ui_text(t("import_guide"), 16).color(accent),
            space::horizontal(),
            ui_text(t("import_guide_subtitle"), 11).color(Palette::TEXT_DIM()),
        ]
        .align_y(iced::Alignment::Center),
        rule::horizontal(1),
        tips,
        rule::horizontal(1),
        notes,
    ]
    .spacing(12);

    let bg = Palette::BG_CARD();
    let border = Palette::BORDER_SUBTLE();
    container(card_content)
        .padding([16, 20])
        .width(Length::Fill)
        .style(theme::card_style(bg, border, 10.0))
        .into()
}

// ── Shared helpers ──

/// "Import Steam", on the button that starts an import.
fn import_label(platform: Platform) -> String {
    tf("import_platform", &[("platform", &platform)])
}

fn step_label<'a>(num: &'a str, label: &'a str, color: Color) -> Element<'a, Message> {
    let num_badge = container(ui_text(num, 11).color(color))
        .width(22)
        .height(22)
        .center_x(22)
        .center_y(22)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(color.scale_alpha(0.15))),
            border: Border {
                radius: 11.0.into(),
                ..Border::default()
            },
            ..container::Style::default()
        });

    row![num_badge, ui_text(label, 13).color(color.scale_alpha(0.7)),]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into()
}

fn tip_row<'a>(platform: Platform, tip: &'a str) -> Element<'a, Message> {
    row![
        container(ui_text(platform.to_string(), 11).color(platform.color()))
            .width(colony_ui::sz(80))
            .padding([2, 0]),
        ui_text(tip, 12).color(Palette::TEXT_SECONDARY()),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Start)
    .into()
}

fn note_row<'a>(note: &'a str) -> Element<'a, Message> {
    row![
        ui_text("*", 11).color(Palette::TEXT_DIM()).width(14),
        ui_text(note, 11).color(Palette::TEXT_DIM()),
    ]
    .spacing(4)
    .into()
}
