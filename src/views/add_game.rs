use colony_ui::theme::Palette;
use colony_ui::{sz, ui_font};
use iced::widget::{button, column, container, row, rule, text_input};
use iced::{Color, Element, Length};

use crate::app::{Message, Screen, Spotter};
use crate::i18n::t;
use crate::models::{GameStatus, Platform};
use crate::theme::{self, ui_text};

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let header = column![
        ui_text(format!("{} {}", theme::icons::ADD, t("nav_add_game")), 24)
            .font(colony_ui::ui_font_bold())
            .color(Palette::TEXT_PRIMARY()),
        ui_text(t("add_subtitle"), 12).color(Palette::TEXT_MUTED()),
    ]
    .spacing(4);

    // Title input
    let title_section = labeled_section(
        t("add_title_label"),
        text_input(t("add_title_placeholder"), &app.new_game_title)
            .on_input(Message::AddGameTitleChanged)
            .padding(10)
            .size(sz(14))
            .font(ui_font()),
    );

    // Genre input
    let genre_section = labeled_section(
        t("detail_genre"),
        text_input(t("add_genre_placeholder"), &app.new_game_genre)
            .on_input(Message::AddGameGenreChanged)
            .padding(10)
            .size(sz(14))
            .font(ui_font()),
    );

    // Platform picker
    let platform_section = labeled_section(
        t("add_platform"),
        row(Platform::ALL.map(|platform| {
            choice_chip(
                platform.to_string(),
                app.new_game_platform == platform,
                platform.color(),
                Message::AddGamePlatformChanged(platform),
            )
        }))
        .spacing(6)
        .wrap(),
    );

    // Status picker
    let status_section = labeled_section(
        t("add_status"),
        row(GameStatus::ALL.map(|status| {
            choice_chip(
                status.label().to_owned(),
                app.new_game_status == status,
                status.color(),
                Message::AddGameStatusChanged(status),
            )
        }))
        .spacing(6)
        .wrap(),
    );

    // Buttons
    let can_save = !app.new_game_title.trim().is_empty();
    let save_btn = if can_save {
        button(ui_text(t("add_save"), 14))
            .on_press(Message::SaveNewGame)
            .padding([10, 24])
            .style(theme::primary_btn_style(Palette::BTN_SUCCESS()))
    } else {
        button(ui_text(t("add_save"), 14))
            .padding([10, 24])
            .style(theme::disabled_btn_style())
    };

    let cancel_btn = button(ui_text(t("action_cancel"), 13))
        .on_press(Message::NavigateTo(Screen::Library))
        .padding([8, 16])
        .style(theme::outline_btn_style(Palette::TEXT_SECONDARY()));

    let form_card = container(
        column![
            title_section,
            genre_section,
            rule::horizontal(1),
            platform_section,
            status_section,
            rule::horizontal(1),
            row![save_btn, cancel_btn].spacing(12),
        ]
        .spacing(16),
    )
    .padding(20)
    .width(Length::Fill)
    .style(theme::card_style(
        Palette::BG_CARD(),
        Palette::BORDER_SUBTLE(),
        8.0,
    ));

    let content = column![header, rule::horizontal(1), form_card,]
        .spacing(16)
        .padding(24)
        .width(Length::Fill);

    iced::widget::scrollable(container(content).width(Length::Fill).height(Length::Fill)).into()
}

fn labeled_section<'a>(
    label: &'a str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        ui_text(label, 13).color(Palette::TEXT_PRIMARY()),
        content.into(),
    ]
    .spacing(6)
    .into()
}

/// One choice of a picker row, tinted with the choice's own colour when picked.
fn choice_chip(
    label: String,
    active: bool,
    color: Color,
    msg: Message,
) -> Element<'static, Message> {
    button(ui_text(label, 12))
        .on_press(msg)
        .padding([6, 12])
        .style(theme::chip_style(active, color, 8.0))
        .into()
}
