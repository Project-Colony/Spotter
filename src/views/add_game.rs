use iced::widget::{button, column, container, row, rule, text, text_input};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::{Message, Screen, Spotter};
use crate::models::{GameStatus, Platform};
use crate::theme::{self, ViewTheme};

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let vt = ViewTheme::from_settings(&app.settings);

    let header = column![
        text(format!("{} Add Game", theme::icons::ADD))
            .size(24)
            .font(theme::FONT_BOLD)
            .color(Color::WHITE),
        text("Manually add a game to your library")
            .size(12)
            .color(vt.text_muted),
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
        button(text("Add to Library").size(14).color(theme::BG_DARKER))
            .on_press(Message::SaveNewGame)
            .padding([10, 24])
            .style(|_: &Theme, status| {
                let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
                let bg = if hover {
                    crate::theme::lighten(theme::SUCCESS, 0.04)
                } else {
                    theme::SUCCESS
                };
                button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: theme::BG_DARKER,
                    border: Border {
                        radius: 8.0.into(),
                        ..Border::default()
                    },
                    ..button::Style::default()
                }
            })
    } else {
        button(text("Add to Library").size(14).color(theme::TEXT_HEADING))
            .padding([10, 24])
            .style(|_: &Theme, _| button::Style {
                background: Some(Background::Color(theme::BG_BUTTON)),
                text_color: theme::TEXT_HEADING,
                border: Border {
                    radius: 8.0.into(),
                    color: theme::BORDER_CARD,
                    width: 1.0,
                },
                ..button::Style::default()
            })
    };

    let cancel_btn = button(text("Cancel").size(13).color(theme::TEXT_SECONDARY))
        .on_press(Message::NavigateTo(Screen::Library))
        .padding([8, 16])
        .style(|_: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let bg = if hover {
                crate::theme::lighten(theme::BG_BUTTON, 0.04)
            } else {
                theme::BG_BUTTON
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: theme::TEXT_SECONDARY,
                border: Border {
                    radius: 6.0.into(),
                    color: theme::BORDER_CARD,
                    width: 1.0,
                },
                ..button::Style::default()
            }
        });

    let bg_card = vt.bg_card;
    let border_color = vt.border;
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
    .style(move |_: &Theme| container::Style {
        background: Some(Background::Color(bg_card)),
        border: Border {
            radius: 8.0.into(),
            color: border_color,
            width: 1.0,
        },
        ..container::Style::default()
    });

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
        text(label).size(13).color(theme::TEXT_LIGHT),
        content.into(),
    ]
    .spacing(6)
    .into()
}

fn platform_btn<'a>(label: &'a str, target: Platform, current: Platform) -> Element<'a, Message> {
    let is_active = current == target;
    let color = if is_active {
        target.color()
    } else {
        theme::TEXT_MUTED
    };

    button(text(label).size(12).color(color))
        .on_press(Message::AddGamePlatformChanged(target))
        .padding([6, 12])
        .style(move |_: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let bg = if is_active {
                if hover {
                    crate::theme::lighten(Color::from_rgb(0.15, 0.15, 0.22), 0.04)
                } else {
                    Color::from_rgb(0.15, 0.15, 0.22)
                }
            } else if hover {
                crate::theme::lighten(theme::BG_BUTTON, 0.04)
            } else {
                theme::BG_BUTTON
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: color,
                border: Border {
                    radius: 8.0.into(),
                    color: if is_active {
                        color
                    } else if hover {
                        Color::from_rgb(0.2, 0.2, 0.28)
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

fn status_btn<'a>(label: &'a str, target: GameStatus, current: GameStatus) -> Element<'a, Message> {
    let is_active = current == target;
    let color = if is_active {
        target.color()
    } else {
        theme::TEXT_MUTED
    };

    button(text(label).size(12).color(color))
        .on_press(Message::AddGameStatusChanged(target))
        .padding([6, 12])
        .style(move |_: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let bg = if is_active {
                if hover {
                    crate::theme::lighten(Color::from_rgb(0.15, 0.15, 0.22), 0.04)
                } else {
                    Color::from_rgb(0.15, 0.15, 0.22)
                }
            } else if hover {
                crate::theme::lighten(theme::BG_BUTTON, 0.04)
            } else {
                theme::BG_BUTTON
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: color,
                border: Border {
                    radius: 8.0.into(),
                    color: if is_active {
                        color
                    } else if hover {
                        Color::from_rgb(0.2, 0.2, 0.28)
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
