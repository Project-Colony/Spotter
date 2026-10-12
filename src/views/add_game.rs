use colony_ui::theme::Palette;
use iced::widget::{button, column, container, row, rule, text, text_input};
use iced::{Color, Element, Length};

use crate::app::{Message, Screen, Spotter};
use crate::models::{GameStatus, Platform};
use crate::theme;

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let header = column![
        text(format!("{} Add Game", theme::icons::ADD))
            .size(24)
            .font(colony_ui::ui_font_bold())
            .color(Palette::TEXT_PRIMARY()),
        text("Manually add a game to your library")
            .size(12)
            .color(Palette::TEXT_MUTED()),
    ]
    .spacing(4);

    // Title input
    let title_section = labeled_section(
        "Game Title *",
        text_input("Enter game title...", &app.new_game_title)
            .on_input(Message::AddGameTitleChanged)
            .padding(10)
            .size(14),
    );

    // Genre input
    let genre_section = labeled_section(
        "Genre",
        text_input("e.g. RPG, Action, Adventure...", &app.new_game_genre)
            .on_input(Message::AddGameGenreChanged)
            .padding(10)
            .size(14),
    );

    // Platform picker
    let platform_section = labeled_section(
        "Platform",
        row![
            platform_btn("Steam", Platform::Steam, app.new_game_platform),
            platform_btn("GOG", Platform::Gog, app.new_game_platform),
            platform_btn("Epic", Platform::Epic, app.new_game_platform),
            platform_btn("PlayStation", Platform::PlayStation, app.new_game_platform),
            platform_btn("Xbox", Platform::Xbox, app.new_game_platform),
            platform_btn("Nintendo", Platform::Nintendo, app.new_game_platform),
        ]
        .spacing(6),
    );

    // Status picker
    let status_section = labeled_section(
        "Status",
        row![
            status_btn("Unplayed", GameStatus::Unplayed, app.new_game_status),
            status_btn("Playing", GameStatus::Playing, app.new_game_status),
            status_btn("Completed", GameStatus::Completed, app.new_game_status),
            status_btn("Dropped", GameStatus::Dropped, app.new_game_status),
            status_btn("Wishlist", GameStatus::Wishlist, app.new_game_status),
        ]
        .spacing(6),
    );

    // Buttons
    let can_save = !app.new_game_title.trim().is_empty();
    let save_btn = if can_save {
        button(text("Add to Library").size(14))
            .on_press(Message::SaveNewGame)
            .padding([10, 24])
            .style(theme::primary_btn_style(Palette::BTN_SUCCESS()))
    } else {
        button(text("Add to Library").size(14))
            .padding([10, 24])
            .style(theme::disabled_btn_style())
    };

    let cancel_btn = button(text("Cancel").size(13))
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
        text(label).size(13).color(Palette::TEXT_PRIMARY()),
        content.into(),
    ]
    .spacing(6)
    .into()
}

fn platform_btn<'a>(label: &'a str, target: Platform, current: Platform) -> Element<'a, Message> {
    choice_chip(
        label,
        current == target,
        target.color(),
        Message::AddGamePlatformChanged(target),
    )
}

fn status_btn<'a>(label: &'a str, target: GameStatus, current: GameStatus) -> Element<'a, Message> {
    choice_chip(
        label,
        current == target,
        target.color(),
        Message::AddGameStatusChanged(target),
    )
}

/// One choice of a picker row, tinted with the choice's own colour when picked.
fn choice_chip(label: &str, active: bool, color: Color, msg: Message) -> Element<'_, Message> {
    button(text(label).size(12))
        .on_press(msg)
        .padding([6, 12])
        .style(theme::chip_style(active, color, 8.0))
        .into()
}
