use iced::widget::{button, column, container, row, rule, space, text, text_input};
use iced::{Background, Border, Color, Element, Length, Theme};

use super::helpers::{platform_dot, separator, status_badge};
use crate::app::{Message, Spotter};
use crate::messages::AuthMessage;
use crate::theme::{self, ViewTheme};

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let vt = ViewTheme::from_settings(&app.settings);
    let accent = app.settings.accent_color.color();

    let header = column![
        text(format!("{} Import Games", theme::icons::IMPORT))
            .size(24)
            .font(theme::FONT_BOLD)
            .color(Color::WHITE),
        text("Import your game library from Steam, GOG, and more")
            .size(12)
            .color(vt.text_muted),
    ]
    .spacing(4);

    // ── Import status banner ──
    let status_section = import_status_banner(app, vt);

    // ── Row 1: Steam | GOG ──
    let steam_card = steam_setup_card(app, vt);
    let gog_card = gog_setup_card(app, vt);

    // ── Row 2: Epic | Xbox ──
    let epic_card = epic_setup_card(app, vt);

    let xbox_card = xbox_setup_card(app, vt);

    // ── Row 3: PlayStation | Nintendo ──
    let psn_card = psn_setup_card(app, vt);

    let nintendo_card = nintendo_info_card(vt);

    // ── Help section ──
    let help_card = help_section(accent, vt);

    // Import history log
    let history_section = import_history_card(app, vt);

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

fn import_status_banner<'a>(app: &'a Spotter, vt: ViewTheme) -> Element<'a, Message> {
    let any_importing = !app.importing.is_empty();

    if !any_importing && app.import_status.is_empty() {
        return column![].into();
    }

    let mut content = column![].spacing(6);

    if any_importing {
        let platforms: Vec<&String> = app.importing.iter().collect();
        let label = format!(
            "Importing from {}...",
            platforms
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        let dots = match (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            / 500)
            % 4
        {
            0 => ".",
            1 => "..",
            2 => "...",
            _ => "",
        };
        let spinner = text(dots.to_string()).size(14).color(theme::ACCENT_GOLD);
        let msg = text(label).size(14).color(theme::ACCENT_GOLD);
        content = content.push(
            row![spinner, msg]
                .spacing(10)
                .align_y(iced::Alignment::Center),
        );
    }

    if !app.import_status.is_empty() && !any_importing {
        let is_error = app.import_status.contains("error") || app.import_status.contains("Error");
        let (icon, color) = if is_error {
            ("!", Color::from_rgb(1.0, 0.4, 0.4))
        } else {
            ("OK", theme::SUCCESS)
        };
        let badge = container(text(icon).size(11).color(Color::WHITE))
            .padding([2, 8])
            .style(theme::pill_style(
                Color { a: 0.2, ..color },
                Color { a: 0.4, ..color },
            ));
        content = content.push(
            row![badge, text(&app.import_status).size(13).color(color)]
                .spacing(10)
                .align_y(iced::Alignment::Center),
        );
    }

    let (banner_bg, banner_border) = if any_importing {
        (
            Color::from_rgb(0.12, 0.11, 0.06),
            Color::from_rgb(0.25, 0.22, 0.08),
        )
    } else {
        (vt.bg_card, vt.border)
    };

    container(content)
        .padding([12, 16])
        .width(Length::Fill)
        .style(theme::card_style(banner_bg, banner_border, 10.0))
        .into()
}

// ── Steam setup card ──

fn steam_setup_card(app: &Spotter, vt: ViewTheme) -> Element<'_, Message> {
    let color = Color::from_rgb(0.4, 0.6, 0.9);
    let has_steam_id = !app.profile.steam_id.is_empty();
    let has_api_key = !app.profile.steam_api_key.is_empty();
    let ready = has_steam_id && has_api_key;
    let steam_importing = app.importing.contains("Steam");

    // Header row: platform dot + title + status pill
    let status_pill = if ready {
        status_badge("Ready", theme::SUCCESS)
    } else if has_steam_id {
        status_badge("API key needed", theme::ACCENT_GOLD)
    } else {
        status_badge("Setup required", Color::from_rgb(0.8, 0.4, 0.3))
    };

    let header_row = row![
        platform_dot(color),
        text("Steam").size(18).color(color),
        space::horizontal(),
        status_pill,
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // Step 1: Login
    let step1_label = step_label("1", "Login", color);
    let login_status: Element<'_, Message> = if has_steam_id {
        row![
            text("ID: ").size(12).color(theme::TEXT_MUTED),
            text(&app.profile.steam_id).size(12).color(theme::SUCCESS),
        ]
        .spacing(4)
        .into()
    } else {
        text("Not logged in")
            .size(12)
            .color(Color::from_rgb(0.6, 0.4, 0.35))
            .into()
    };

    let login_btn = if app.steam_login_active {
        button(text("Waiting...").size(12).color(theme::ACCENT_GOLD))
            .padding([6, 14])
            .style(theme::loading_btn_style())
    } else {
        button(text("Login").size(12).color(theme::BG_DARKER))
            .on_press(AuthMessage::SteamLogin.into())
            .padding([6, 14])
            .style(theme::primary_btn_style(color))
    };

    // Step 2: API Key
    let step2_label = step_label("2", "API Key", color);
    let api_key_input = column![
        text_input("Steam Web API key", &app.profile.steam_api_key)
            .on_input(Message::UpdateSteamApiKey)
            .padding(8)
            .size(12),
        text("steamcommunity.com/dev/apikey")
            .size(10)
            .color(theme::TEXT_DIM),
    ]
    .spacing(4);

    // Import button
    let import_btn = if ready && !steam_importing {
        button(text("Import Steam").size(13).color(theme::BG_DARKER))
            .on_press(Message::ImportSteam)
            .padding([8, 18])
            .style(theme::primary_btn_style(theme::SUCCESS))
    } else if steam_importing {
        button(text("Importing...").size(13).color(theme::ACCENT_GOLD))
            .padding([8, 18])
            .style(theme::loading_btn_style())
    } else {
        button(text("Import Steam").size(13).color(theme::TEXT_HEADING))
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
        .style(theme::accent_card_style(vt.bg_card, color))
        .into()
}

// ── GOG setup card ──

fn gog_setup_card(app: &Spotter, vt: ViewTheme) -> Element<'_, Message> {
    let color = Color::from_rgb(0.7, 0.5, 0.9);
    let has_token = !app.profile.gog_token.is_empty();
    let gog_importing = app.importing.contains("GOG");

    let status_pill = if has_token {
        status_badge("Ready", theme::SUCCESS)
    } else if app.gog_login_active {
        status_badge("Logging in...", theme::ACCENT_GOLD)
    } else {
        status_badge("Setup required", Color::from_rgb(0.8, 0.4, 0.3))
    };

    let header_row = row![
        platform_dot(color),
        text("GOG Galaxy").size(18).color(color),
        space::horizontal(),
        status_pill,
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // Step 1: Login
    let step1_label = step_label("1", "Login", color);
    let login_status: Element<'_, Message> = if has_token {
        text("Logged in").size(12).color(theme::SUCCESS).into()
    } else if app.gog_login_active {
        text("Waiting for login...")
            .size(12)
            .color(theme::ACCENT_GOLD)
            .into()
    } else {
        text("Not logged in")
            .size(12)
            .color(Color::from_rgb(0.6, 0.4, 0.35))
            .into()
    };

    let login_btn = if app.gog_login_active {
        button(text("Waiting...").size(12).color(theme::ACCENT_GOLD))
            .padding([6, 14])
            .style(theme::loading_btn_style())
    } else {
        button(text("Login").size(12).color(theme::BG_DARKER))
            .on_press(AuthMessage::GogLogin.into())
            .padding([6, 14])
            .style(theme::primary_btn_style(color))
    };

    // Step 2: Paste code
    let code_section: Element<'_, Message> = if app.gog_login_active || !has_token {
        let step2_label = step_label("2", "Paste URL", color);
        let submit_btn = if !app.gog_code_input.is_empty() {
            button(text("Complete").size(12).color(theme::BG_DARKER))
                .on_press(AuthMessage::GogSubmitCode.into())
                .padding([6, 14])
                .style(theme::primary_btn_style(theme::SUCCESS))
        } else {
            button(text("Complete").size(12).color(theme::TEXT_HEADING))
                .padding([6, 14])
                .style(theme::disabled_btn_style())
        };

        column![
            separator(color),
            step2_label,
            text("Copy the redirect URL after signing in")
                .size(10)
                .color(theme::TEXT_DIM),
            text_input("Paste redirect URL here...", &app.gog_code_input,)
                .on_input(|s| AuthMessage::GogCodeChanged(s).into())
                .padding(8)
                .size(12),
            submit_btn,
        ]
        .spacing(6)
        .into()
    } else {
        column![].into()
    };

    // Import button
    let import_btn = if has_token && !gog_importing {
        button(text("Import GOG").size(13).color(theme::BG_DARKER))
            .on_press(Message::ImportGog)
            .padding([8, 18])
            .style(theme::primary_btn_style(theme::SUCCESS))
    } else if gog_importing {
        button(text("Importing...").size(13).color(theme::ACCENT_GOLD))
            .padding([8, 18])
            .style(theme::loading_btn_style())
    } else {
        button(text("Import GOG").size(13).color(theme::TEXT_HEADING))
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
        .style(theme::accent_card_style(vt.bg_card, color))
        .into()
}

// ── Epic Games setup card ──

fn epic_setup_card(app: &Spotter, vt: ViewTheme) -> Element<'_, Message> {
    let color = Color::from_rgb(0.9, 0.9, 0.3);
    let has_token = !app.profile.epic_token.is_empty();
    let has_account = !app.profile.epic_account_id.is_empty();
    let logged_in = has_token && has_account;
    let epic_importing = app.importing.contains("Epic");

    let status_pill = if logged_in {
        status_badge("Connected", theme::SUCCESS)
    } else if app.epic_login_active {
        status_badge("Logging in...", theme::ACCENT_GOLD)
    } else {
        status_badge("Local only", Color::from_rgb(0.7, 0.6, 0.3))
    };

    let header_row = row![
        platform_dot(color),
        text("Epic Games").size(18).color(color),
        space::horizontal(),
        status_pill,
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // Step 1: Login
    let step1_label = step_label("1", "Login", color);
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
            text("Account: ").size(12).color(theme::TEXT_MUTED),
            text(name).size(12).color(theme::SUCCESS),
        ]
        .spacing(4)
        .into()
    } else if app.epic_login_active {
        text("Waiting for login...")
            .size(12)
            .color(theme::ACCENT_GOLD)
            .into()
    } else {
        text("Optional — without login, only installed games are detected")
            .size(12)
            .color(Color::from_rgb(0.6, 0.5, 0.35))
            .into()
    };

    let login_btn = if app.epic_login_active {
        button(text("Waiting...").size(12).color(theme::ACCENT_GOLD))
            .padding([6, 14])
            .style(theme::loading_btn_style())
    } else if logged_in {
        button(text("Re-login").size(12).color(theme::BG_DARKER))
            .on_press(AuthMessage::EpicLogin.into())
            .padding([6, 14])
            .style(theme::primary_btn_style(color))
    } else {
        button(text("Login").size(12).color(theme::BG_DARKER))
            .on_press(AuthMessage::EpicLogin.into())
            .padding([6, 14])
            .style(theme::primary_btn_style(color))
    };

    // Step 2: Paste code (only shown when login is active or not logged in)
    let code_section: Element<'_, Message> = if app.epic_login_active || !logged_in {
        let step2_label = step_label("2", "Paste Code", color);
        let submit_btn = if !app.epic_code_input.is_empty() {
            button(text("Complete").size(12).color(theme::BG_DARKER))
                .on_press(AuthMessage::EpicSubmitCode.into())
                .padding([6, 14])
                .style(theme::primary_btn_style(theme::SUCCESS))
        } else {
            button(text("Complete").size(12).color(theme::TEXT_HEADING))
                .padding([6, 14])
                .style(theme::disabled_btn_style())
        };

        column![
            separator(color),
            step2_label,
            text("Copy the JSON response (or authorization code) from the browser page")
                .size(10)
                .color(theme::TEXT_DIM),
            text_input(
                "Paste JSON or authorization code here...",
                &app.epic_code_input
            )
            .on_input(|s| AuthMessage::EpicCodeChanged(s).into())
            .padding(8)
            .size(12),
            submit_btn,
        ]
        .spacing(6)
        .into()
    } else {
        column![].into()
    };

    // Import button
    let import_label = if logged_in {
        "Import Full Library"
    } else {
        "Import Local Games"
    };

    let import_btn = if !epic_importing {
        button(text(import_label).size(13).color(theme::BG_DARKER))
            .on_press(Message::ImportEpic)
            .padding([8, 18])
            .style(theme::primary_btn_style(if logged_in {
                theme::SUCCESS
            } else {
                color
            }))
    } else {
        button(text("Importing...").size(13).color(theme::ACCENT_GOLD))
            .padding([8, 18])
            .style(theme::loading_btn_style())
    };

    let import_note: Element<'_, Message> = if logged_in {
        text("Full library import via Epic account (all owned games)")
            .size(10)
            .color(theme::TEXT_DIM)
            .into()
    } else {
        text("Local scan only (installed games). Login for full library.")
            .size(10)
            .color(theme::TEXT_DIM)
            .into()
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
        import_note,
    ]
    .spacing(8);

    container(card_content)
        .padding([14, 16])
        .width(Length::Fill)
        .style(theme::accent_card_style(vt.bg_card, color))
        .into()
}

// ── Xbox setup card ──

fn xbox_setup_card(app: &Spotter, vt: ViewTheme) -> Element<'_, Message> {
    let color = Color::from_rgb(0.2, 0.8, 0.2);
    let has_key = !app.profile.xbox_api_key.is_empty();
    let has_tag = !app.profile.xbox_gamertag.is_empty();
    let xbox_importing = app.importing.contains("Xbox");

    let status_pill = if xbox_importing {
        status_badge("Importing...", theme::ACCENT_GOLD)
    } else if has_key {
        status_badge("Ready", theme::SUCCESS)
    } else {
        status_badge("Setup required", Color::from_rgb(0.8, 0.4, 0.3))
    };

    let header_row = row![
        platform_dot(color),
        text("Xbox").size(18).color(color),
        space::horizontal(),
        status_pill,
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // Step 1: API Key
    let step1_label = step_label("1", "API Key", color);
    let api_key_input = column![
        text_input(
            "Enter your OpenXBL API key from xbl.io",
            &app.profile.xbox_api_key
        )
        .on_input(Message::UpdateXboxApiKey)
        .padding(8)
        .size(12),
        button(
            text("Get a free API key at xbl.io")
                .size(10)
                .color(theme::ACCENT_BLUE),
        )
        .on_press(Message::OpenUrl("https://xbl.io".to_string()))
        .padding(0)
        .style(theme::transparent_btn_style()),
    ]
    .spacing(4);

    // Step 2: Gamertag (shown when API key is entered)
    let gamertag_section: Element<'_, Message> = if has_key || has_tag {
        let step2_label = step_label("2", "Gamertag", color);
        column![
            separator(color),
            step2_label,
            text_input("Your Xbox gamertag", &app.profile.xbox_gamertag)
                .on_input(Message::UpdateXboxGamertag)
                .padding(8)
                .size(12),
        ]
        .spacing(6)
        .into()
    } else {
        column![].into()
    };

    // Import button
    let import_btn = if has_key && !xbox_importing {
        button(text("Import Xbox").size(13).color(theme::BG_DARKER))
            .on_press(Message::ImportXbox)
            .padding([8, 18])
            .style(theme::primary_btn_style(theme::SUCCESS))
    } else if xbox_importing {
        button(text("Importing...").size(13).color(theme::ACCENT_GOLD))
            .padding([8, 18])
            .style(theme::loading_btn_style())
    } else {
        button(text("Import Xbox").size(13).color(theme::TEXT_HEADING))
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
        .style(theme::accent_card_style(vt.bg_card, color))
        .into()
}

// ── PlayStation setup card ──

fn psn_setup_card(app: &Spotter, vt: ViewTheme) -> Element<'_, Message> {
    let color = Color::from_rgb(0.2, 0.4, 0.9);
    let has_token = !app.profile.psn_npsso.is_empty();
    let psn_importing = app.importing.contains("PlayStation");

    let status_pill = if psn_importing {
        status_badge("Importing...", theme::ACCENT_GOLD)
    } else if has_token {
        status_badge("Ready", theme::SUCCESS)
    } else {
        status_badge("Setup required", Color::from_rgb(0.8, 0.4, 0.3))
    };

    let header_row = row![
        platform_dot(color),
        text("PlayStation").size(18).color(color),
        space::horizontal(),
        status_pill,
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // Step 1: NPSSO Token
    let step1_label = step_label("1", "NPSSO Token", color);
    let npsso_input = column![
        text_input(
            "Get from ca.account.sony.com/api/v1/ssocookie",
            &app.profile.psn_npsso,
        )
        .on_input(Message::UpdatePsnNpsso)
        .padding(8)
        .size(12),
        button(
            text("Get token from ca.account.sony.com")
                .size(10)
                .color(theme::ACCENT_BLUE),
        )
        .on_press(Message::OpenUrl(
            "https://ca.account.sony.com/api/v1/ssocookie".to_string(),
        ))
        .padding(0)
        .style(theme::transparent_btn_style()),
    ]
    .spacing(4);

    // Import button
    let import_btn = if has_token && !psn_importing {
        button(text("Import PlayStation").size(13).color(theme::BG_DARKER))
            .on_press(Message::ImportPlayStation)
            .padding([8, 18])
            .style(theme::primary_btn_style(theme::SUCCESS))
    } else if psn_importing {
        button(text("Importing...").size(13).color(theme::ACCENT_GOLD))
            .padding([8, 18])
            .style(theme::loading_btn_style())
    } else {
        button(
            text("Import PlayStation")
                .size(13)
                .color(theme::TEXT_HEADING),
        )
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
        .style(theme::accent_card_style(vt.bg_card, color))
        .into()
}

// ── Nintendo info card (no importer available) ──

fn nintendo_info_card(vt: ViewTheme) -> Element<'static, Message> {
    let color = Color::from_rgb(0.9, 0.2, 0.2);

    let header_row = row![
        platform_dot(color),
        text("Nintendo").size(18).color(color),
        space::horizontal(),
        status_badge("Manual only", Color::from_rgb(0.6, 0.5, 0.3)),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let card_content = column![
        header_row,
        text("Nintendo does not provide a public API for game libraries.")
            .size(12)
            .color(vt.text_muted),
        text("You can add Nintendo games manually via the Add Game page.")
            .size(12)
            .color(vt.text_muted),
    ]
    .spacing(10);

    container(card_content)
        .padding([14, 18])
        .width(Length::Fill)
        .style(theme::accent_card_style(vt.bg_card, color))
        .into()
}

// ── Import history card ──

fn import_history_card<'a>(app: &'a Spotter, vt: ViewTheme) -> Element<'a, Message> {
    if app.import_history.is_empty() {
        return column![].into();
    }

    let accent = app.settings.accent_color.color();
    let mut log_col = column![].spacing(4);

    for entry in app.import_history.iter().rev().take(10) {
        let is_error = entry.contains("ERROR");
        let color = if is_error {
            Color::from_rgb(1.0, 0.4, 0.4)
        } else {
            theme::SUCCESS
        };
        log_col = log_col.push(text(entry.as_str()).size(12).color(color));
    }

    let card_content = column![
        row![
            text("Import Log").size(16).color(accent),
            space::horizontal(),
            text(format!("{} entries", app.import_history.len()))
                .size(11)
                .color(vt.text_dim),
        ]
        .align_y(iced::Alignment::Center),
        separator(accent),
        log_col,
    ]
    .spacing(8);

    container(card_content)
        .padding([14, 18])
        .width(Length::Fill)
        .style(theme::accent_card_style(vt.bg_card, accent))
        .into()
}

// ── Help section ──

fn help_section<'a>(accent: Color, vt: ViewTheme) -> Element<'a, Message> {
    let tips = column![
        tip_row(
            "Steam",
            "Login to get your ID, then enter your API key from steamcommunity.com/dev/apikey",
            Color::from_rgb(0.4, 0.6, 0.9)
        ),
        tip_row(
            "GOG",
            "Click Login, sign in, paste the redirect URL back",
            Color::from_rgb(0.7, 0.5, 0.9)
        ),
        tip_row(
            "Epic",
            "Login to import full library, or scan locally (installed only)",
            Color::from_rgb(0.9, 0.9, 0.3)
        ),
        tip_row(
            "Xbox",
            "Get a free API key from xbl.io, enter it above, then import",
            Color::from_rgb(0.2, 0.8, 0.2)
        ),
        tip_row(
            "PSN",
            "Get NPSSO token from ca.account.sony.com, enter it above",
            Color::from_rgb(0.2, 0.4, 0.9)
        ),
        tip_row(
            "Nintendo",
            "No API available — add games manually via Add Game",
            Color::from_rgb(0.9, 0.2, 0.2)
        ),
    ]
    .spacing(8);

    let notes = column![
        note_row("Re-importing updates existing games and adds new ones (no duplicates)"),
        note_row("Cover images are downloaded automatically after import"),
        note_row("You can export/import JSON backups from the Profile page"),
    ]
    .spacing(4);

    let card_content = column![
        row![
            text("Setup Guide").size(16).color(accent),
            space::horizontal(),
            text("Quick Reference").size(11).color(vt.text_dim),
        ]
        .align_y(iced::Alignment::Center),
        rule::horizontal(1),
        tips,
        rule::horizontal(1),
        notes,
    ]
    .spacing(12);

    let bg = vt.bg_card;
    let border = vt.border;
    container(card_content)
        .padding([16, 20])
        .width(Length::Fill)
        .style(theme::card_style(bg, border, 10.0))
        .into()
}

// ── Shared helpers ──

fn step_label<'a>(num: &'a str, label: &'a str, color: Color) -> Element<'a, Message> {
    let num_badge = container(text(num).size(11).color(color))
        .width(22)
        .height(22)
        .center_x(22)
        .center_y(22)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(Color { a: 0.15, ..color })),
            border: Border {
                radius: 11.0.into(),
                ..Border::default()
            },
            ..container::Style::default()
        });

    row![
        num_badge,
        text(label).size(13).color(Color { a: 0.7, ..color }),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn tip_row<'a>(platform: &'a str, tip: &'a str, color: Color) -> Element<'a, Message> {
    row![
        container(text(platform).size(11).color(color))
            .width(55)
            .padding([2, 0]),
        text(tip).size(12).color(theme::TEXT_SECONDARY),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Start)
    .into()
}

fn note_row<'a>(note: &'a str) -> Element<'a, Message> {
    row![
        text("*").size(11).color(theme::TEXT_DIM).width(14),
        text(note).size(11).color(theme::TEXT_DIM),
    ]
    .spacing(4)
    .into()
}
