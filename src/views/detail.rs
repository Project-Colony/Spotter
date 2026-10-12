use colony_ui::theme::Palette;
use colony_ui::{sz, ui_font};
use iced::widget::{button, column, container, progress_bar, row, rule, space, text_input};
use iced::{Background, Border, Element, Length, Theme};

use crate::app::{Message, Screen, Spotter};
use crate::models::{AchievementsDisplay, GameStatus, Platform};
use crate::theme::{self, ui_text};

pub fn view(app: &Spotter, game_id: i64) -> Element<'_, Message> {
    let date_format = app.settings.date_format;
    let game = match app.games.iter().find(|g| g.id == Some(game_id)) {
        Some(g) => g,
        None => {
            return container(
                column![
                    ui_text("Game not found", 20).color(Palette::TEXT_PRIMARY()),
                    button(ui_text(
                        format!("{} Back to Library", theme::icons::BACK),
                        13
                    ))
                    .on_press(Message::NavigateTo(Screen::Library))
                    .padding([8, 16])
                    .style(theme::outline_btn_style(Palette::ACCENT())),
                ]
                .spacing(12),
            )
            .padding(40)
            .into();
        }
    };

    let back_label = format!("{} Back to Library", theme::icons::BACK);
    let back_btn = button(ui_text(back_label, 13))
        .on_press(Message::NavigateTo(Screen::Library))
        .padding([6, 12])
        .style(theme::outline_btn_style(Palette::ACCENT()));

    // Cover image
    let cover_element: Element<'_, Message> = {
        let cover_path = crate::images::cover_path(&game.title)
            .filter(|_| app.cover_cache.contains(&game.title));
        if let Some(cover_path) = cover_path {
            container(
                iced::widget::image(iced::widget::image::Handle::from_path(&cover_path))
                    .width(460)
                    .height(215),
            )
            .style(|_: &Theme| container::Style {
                border: Border {
                    radius: 8.0.into(),
                    color: Palette::BORDER_SUBTLE(),
                    width: 1.0,
                },
                ..container::Style::default()
            })
            .into()
        } else {
            container(
                column![
                    ui_text(&game.title, 20).color(Palette::TEXT_DIMMER()),
                    ui_text("No cover image", 12).color(Palette::TEXT_MUTED()),
                ]
                .spacing(4)
                .align_x(iced::Alignment::Center),
            )
            .width(460)
            .height(215)
            .center_x(460)
            .center_y(215)
            .style(theme::card_style(
                Palette::BG_CARD(),
                Palette::BORDER_SUBTLE(),
                8.0,
            ))
            .into()
        }
    };

    let title = ui_text(&game.title, 32).color(Palette::TEXT_PRIMARY());

    // Favorite toggle
    let is_fav = app.settings.favorites.contains(&game_id);
    let fav_label = if is_fav {
        format!("{} Favorited", theme::icons::STAR)
    } else {
        format!("{} Favorite", theme::icons::STAR_O)
    };
    let fav_btn = button(ui_text(fav_label, 13))
        .on_press(Message::ToggleFavorite(game_id))
        .padding([6, 14])
        .style(theme::chip_style(is_fav, Palette::WARNING(), 8.0));

    let platform_badge = container(
        ui_text(format!("{} {}", game.platform.icon(), game.platform), 13)
            .color(game.platform.color()),
    )
    .padding([4, 10])
    .style(theme::pill_style(Palette::BG_CARD(), game.platform.color()));

    let status_badge = container(
        ui_text(format!("{} {}", game.status.icon(), game.status), 13).color(game.status.color()),
    )
    .padding([4, 10])
    .style(theme::pill_style(Palette::BG_CARD(), game.status.color()));

    let mut badges = row![platform_badge, status_badge, fav_btn].spacing(8);
    if let Some(appid) = game.steam_appid {
        let steam_badge = button(ui_text(format!("Steam #{}", appid), 11))
            .on_press(Message::OpenUrl(format!(
                "https://store.steampowered.com/app/{}",
                appid
            )))
            .padding([4, 10])
            .style(theme::outline_btn_style(Platform::Steam.color()));
        badges = badges.push(steam_badge);
    }

    // Info section
    let playtime_str = game.playtime_display();
    let rating_str = match game.rating {
        Some(r) => format!("{}/10", r),
        None => "Not rated".to_string(),
    };
    let review_str = match game.review_percent {
        Some(pct) => {
            let label = if pct >= 95 {
                "Overwhelmingly Positive"
            } else if pct >= 80 {
                "Very Positive"
            } else if pct >= 70 {
                "Mostly Positive"
            } else if pct >= 40 {
                "Mixed"
            } else if pct >= 20 {
                "Mostly Negative"
            } else {
                "Very Negative"
            };
            format!("{} ({}% positive)", label, pct)
        }
        None => "No reviews".to_string(),
    };
    let release_str = if game.release_date.is_empty() {
        "Unknown".to_string()
    } else {
        date_format.format_date(&game.release_date)
    };

    let mut info_col = column![
        info_row("Genre", &game.genre),
        info_row("Release Date", &release_str),
        info_row("Last Played", date_format.format_date(&game.last_played)),
        info_row("Playtime", &playtime_str),
        info_row("Rating", &rating_str),
        info_row("Reviews", &review_str),
    ]
    .spacing(8);

    // Description
    if !game.description.is_empty() {
        info_col = info_col.push(rule::horizontal(1));
        info_col = info_col.push(ui_text(&game.description, 12).color(Palette::TEXT_SECONDARY()));
    }

    // Tags
    if !game.tags.is_empty() {
        info_col = info_col.push(rule::horizontal(1));
        info_col = info_col.push(
            row![
                ui_text("Tags: ", 11).color(Palette::TEXT_MUTED()),
                ui_text(&game.tags, 11).color(Palette::TEXT_SECONDARY()),
            ]
            .spacing(4),
        );
    }

    let info_section = info_card("Game Info", info_col);

    // Achievements section — summary + individual list
    let ach_display = app.settings.achievements_display;
    let achievements_section = if app.achievements_loading {
        info_card(
            "Achievements",
            column![ui_text("Loading achievements...", 14).color(Palette::TEXT_MUTED()),]
                .spacing(10),
        )
    } else if ach_display == AchievementsDisplay::Hidden {
        // Hidden: don't show achievements at all
        container(space()).into()
    } else if game.achievements_total > 0 {
        let pct = game.achievement_percent();
        let mut ach_col = column![
            row![
                ui_text(
                    format!(
                        "{} / {}",
                        game.achievements_unlocked, game.achievements_total
                    ),
                    20
                )
                .color(Palette::TEXT_PRIMARY()),
                space::horizontal(),
                ui_text(format!("{:.1}%", pct), 20).color(if pct >= 100.0 {
                    Palette::WARNING()
                } else {
                    Palette::ACCENT()
                }),
            ]
            .align_y(iced::Alignment::Center),
            progress_bar(0.0..=100.0, pct).girth(12),
            ui_text(
                if pct >= 100.0 {
                    "All achievements unlocked!"
                } else {
                    "Keep going!"
                },
                12
            )
            .color(Palette::TEXT_MUTED()),
        ]
        .spacing(10);

        // Individual achievement list (only in Full mode)
        if ach_display == AchievementsDisplay::Full && !app.achievements.is_empty() {
            ach_col = ach_col.push(rule::horizontal(1));

            for ach in &app.achievements {
                let (name_color, desc_color) = if ach.unlocked {
                    (Palette::TEXT_PRIMARY(), Palette::TEXT_SECONDARY())
                } else {
                    (Palette::TEXT_MUTED(), Palette::TEXT_DIM())
                };

                // Achievement icon: show colored icon if unlocked, gray if locked
                // Falls back to colored icon for locked if gray is unavailable
                let icon_path = game.steam_appid.and_then(|appid| {
                    let gray_path =
                        crate::images::achievement_icon_path(appid, &ach.api_name, true);
                    let colored_path =
                        crate::images::achievement_icon_path(appid, &ach.api_name, false);
                    if ach.unlocked {
                        // Unlocked: use colored icon
                        colored_path
                    } else if gray_path.as_ref().is_some_and(|p| p.exists()) {
                        // Locked: prefer gray icon
                        gray_path
                    } else {
                        // Locked but no gray icon: fall back to colored
                        colored_path
                    }
                });
                let icon_element: Element<'_, Message> =
                    if let Some(icon_path) = icon_path.filter(|p| p.exists()) {
                        iced::widget::image(iced::widget::image::Handle::from_path(&icon_path))
                            .width(48)
                            .height(48)
                            .into()
                    } else {
                        // Fallback: colored square while icons are downloading, or for
                        // games outside Steam
                        let dot = if ach.unlocked {
                            Palette::SUCCESS()
                        } else {
                            Palette::TEXT_DIMMEST()
                        };
                        container(space())
                            .width(48)
                            .height(48)
                            .style(move |_: &Theme| container::Style {
                                background: Some(Background::Color(dot)),
                                border: Border {
                                    radius: 4.0.into(),
                                    ..Border::default()
                                },
                                ..container::Style::default()
                            })
                            .into()
                    };

                let mut ach_info =
                    column![ui_text(&ach.display_name, 13).color(name_color),].spacing(2);

                if !ach.description.is_empty() {
                    ach_info = ach_info.push(ui_text(&ach.description, 11).color(desc_color));
                }

                // Show unlock date for unlocked achievements
                if ach.unlocked && ach.unlock_time > 0 {
                    let datetime = date_format.format_timestamp(ach.unlock_time);
                    if !datetime.is_empty() {
                        ach_info = ach_info.push(
                            ui_text(format!("Unlocked {}", datetime), 10).color(Palette::SUCCESS()),
                        );
                    }
                }

                let ach_row = row![icon_element, ach_info]
                    .spacing(12)
                    .align_y(iced::Alignment::Center);

                ach_col = ach_col.push(
                    container(ach_row)
                        .padding([6, 8])
                        .width(Length::Fill)
                        .style(|_: &Theme| container::Style {
                            background: Some(Background::Color(Palette::BG_PRIMARY())),
                            border: Border {
                                radius: 4.0.into(),
                                ..Border::default()
                            },
                            ..container::Style::default()
                        }),
                );
            }
        }

        info_card("Achievements", ach_col)
    } else {
        info_card(
            "Achievements",
            column![
                ui_text("No achievements available for this game", 14).color(Palette::TEXT_MUTED()),
            ]
            .spacing(10),
        )
    };

    // Notes section
    let notes_len = game.notes.len();
    let (notes_counter_color, notes_hint) = if notes_len >= 2000 {
        (Palette::ERROR(), " - limit reached")
    } else if notes_len > 1800 {
        (Palette::WARNING(), " - approaching limit")
    } else {
        (Palette::TEXT_DIM(), "")
    };
    let notes_section = info_card(
        "Notes",
        column![
            text_input("Add notes about this game...", &game.notes)
                .on_input(move |s| Message::SetGameNotes(game_id, s))
                .padding(8)
                .size(sz(13))
                .font(ui_font()),
            ui_text(format!("{} / 2000{}", notes_len, notes_hint), 11).color(notes_counter_color),
        ]
        .spacing(8),
    );

    // Status change buttons (respect large_click_targets setting)
    let bp = app.settings.button_padding();
    let status_section = info_card(
        "Change Status",
        row![
            status_button("Playing", GameStatus::Playing, game.status, game_id, bp),
            status_button("Completed", GameStatus::Completed, game.status, game_id, bp),
            status_button("Unplayed", GameStatus::Unplayed, game.status, game_id, bp),
            status_button("Dropped", GameStatus::Dropped, game.status, game_id, bp),
            status_button("Wishlist", GameStatus::Wishlist, game.status, game_id, bp),
        ]
        .spacing(6),
    );

    // Rating section
    let mut rating_items: Vec<Element<'_, Message>> = (1..=10)
        .map(|r| rating_button(r, game.rating, game_id, bp))
        .collect();
    if game.rating.is_some() {
        rating_items.push(
            button(ui_text("Clear", 11))
                .on_press(Message::SetGameRating(game_id, None))
                .padding([6, 10])
                .style(theme::outline_btn_style(Palette::TEXT_MUTED()))
                .into(),
        );
    }
    let rating_section = info_card("Rate this Game", row(rating_items).spacing(4));

    // Top bar: Back button (left) + Delete button (right)
    let delete_btn = button(ui_text(format!("{} Delete", theme::icons::DELETE), 13))
        .on_press(Message::ConfirmDeleteGame(game_id))
        .padding([6, 12])
        .style(theme::danger_btn_style());

    let top_bar = row![back_btn, space::horizontal(), delete_btn,].align_y(iced::Alignment::Center);

    let content = column![
        top_bar,
        cover_element,
        title,
        badges,
        rule::horizontal(1),
        info_section,
        notes_section,
        status_section,
        rating_section,
        achievements_section,
    ]
    .spacing(16)
    .padding(24)
    .width(Length::Fill);

    iced::widget::scrollable(container(content).width(Length::Fill)).into()
}

fn info_card<'a>(title: &'a str, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    let section = column![
        ui_text(title, 14).color(Palette::TEXT_MUTED()),
        content.into(),
    ]
    .spacing(8);

    container(section)
        .padding(16)
        .width(Length::Fill)
        .style(theme::card_style(
            Palette::BG_CARD(),
            Palette::BORDER_SUBTLE(),
            8.0,
        ))
        .into()
}

fn info_row<'a>(label: &'a str, value: impl ToString) -> Element<'a, Message> {
    row![
        ui_text(label, 13).color(Palette::TEXT_MUTED()).width(120),
        ui_text(value.to_string(), 13).color(Palette::TEXT_PRIMARY()),
    ]
    .spacing(12)
    .into()
}

fn status_button<'a>(
    label: &'a str,
    target: GameStatus,
    current: GameStatus,
    game_id: i64,
    pad: u16,
) -> Element<'a, Message> {
    let color = target.color();
    // Every status keeps its own colour, picked or not.
    button(ui_text(label, 12).color(color))
        .on_press(Message::SetGameStatus(game_id, target))
        .padding([pad, pad + 4])
        .style(theme::chip_style(current == target, color, 6.0))
        .into()
}

fn rating_button<'a>(
    value: u8,
    current: Option<u8>,
    game_id: i64,
    pad: u16,
) -> Element<'a, Message> {
    let is_active = current.is_some_and(|r| value <= r);
    button(ui_text(value.to_string(), 13))
        .on_press(Message::SetGameRating(game_id, Some(value)))
        .padding([pad, pad + 4])
        .style(theme::chip_style(is_active, Palette::WARNING(), 4.0))
        .into()
}
