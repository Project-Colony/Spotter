use iced::widget::{button, column, container, progress_bar, row, rule, space, text, text_input};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::{Message, Screen, Spotter};
use crate::models::{AchievementsDisplay, GameStatus};
use crate::theme::{self, ViewTheme};

pub fn view(app: &Spotter, game_id: i64) -> Element<'_, Message> {
    let vt = ViewTheme::from_settings(&app.settings);
    let game = match app.games.iter().find(|g| g.id == Some(game_id)) {
        Some(g) => g,
        None => {
            return container(
                column![
                    text("Game not found").size(20).color(theme::TEXT_LIGHT),
                    button(
                        text(format!("{} Back to Library", theme::icons::BACK))
                            .size(13)
                            .color(theme::ACCENT_BLUE),
                    )
                    .on_press(Message::NavigateTo(Screen::Library))
                    .padding([8, 16])
                    .style(|_: &Theme, _| button::Style {
                        background: Some(Background::Color(theme::BG_BUTTON)),
                        border: Border {
                            radius: 6.0.into(),
                            ..Border::default()
                        },
                        ..button::Style::default()
                    }),
                ]
                .spacing(12),
            )
            .padding(40)
            .into();
        }
    };

    let back_label = format!("{} Back to Library", theme::icons::BACK);
    let back_btn = button(text(back_label).size(13).color(theme::ACCENT_BLUE))
        .on_press(Message::NavigateTo(Screen::Library))
        .padding([6, 12])
        .style(|_theme: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let bg = if hover {
                Color::from_rgb(
                    theme::BG_BUTTON.r + 0.04,
                    theme::BG_BUTTON.g + 0.04,
                    theme::BG_BUTTON.b + 0.04,
                )
            } else {
                theme::BG_BUTTON
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    radius: 6.0.into(),
                    ..Border::default()
                },
                ..button::Style::default()
            }
        });

    // Cover image
    let cover_element: Element<'_, Message> = {
        let cover_path = crate::images::cover_path(&game.title);
        if app.cover_cache.contains(&game.title) {
            container(
                iced::widget::image(iced::widget::image::Handle::from_path(&cover_path))
                    .width(460)
                    .height(215),
            )
            .style(|_: &Theme| container::Style {
                border: Border {
                    radius: 8.0.into(),
                    color: theme::BORDER_CARD,
                    width: 1.0,
                },
                ..container::Style::default()
            })
            .into()
        } else {
            container(
                column![
                    text(&game.title).size(20).color(theme::TEXT_HEADING),
                    text("No cover image").size(12).color(vt.text_muted),
                ]
                .spacing(4)
                .align_x(iced::Alignment::Center),
            )
            .width(460)
            .height(215)
            .center_x(460)
            .center_y(215)
            .style(|_: &Theme| container::Style {
                background: Some(Background::Color(theme::BG_DARK)),
                border: Border {
                    radius: 8.0.into(),
                    color: theme::BORDER,
                    width: 1.0,
                },
                ..container::Style::default()
            })
            .into()
        }
    };

    let title = text(&game.title).size(32).color(Color::WHITE);

    // Favorite toggle
    let is_fav = app.settings.favorites.contains(&game_id);
    let fav_label = if is_fav {
        format!("{} Favorited", theme::icons::STAR)
    } else {
        format!("{} Favorite", theme::icons::STAR_O)
    };
    let fav_color = if is_fav {
        theme::ACCENT_GOLD
    } else {
        theme::TEXT_MUTED
    };
    let fav_btn = button(text(fav_label).size(13).color(fav_color))
        .on_press(Message::ToggleFavorite(game_id))
        .padding([6, 14])
        .style(theme::chip_style(
            is_fav,
            fav_color,
            theme::ACCENT_GOLD,
            8.0,
        ));

    let platform_badge = container(
        text(format!("{} {}", game.platform.icon(), game.platform))
            .size(13)
            .color(game.platform.color()),
    )
    .padding([4, 10])
    .style(theme::badge_style(game.platform.color()));

    let status_badge = container(
        text(format!("{} {}", game.status.icon(), game.status))
            .size(13)
            .color(game.status.color()),
    )
    .padding([4, 10])
    .style(theme::badge_style(game.status.color()));

    let mut badges = row![platform_badge, status_badge, fav_btn].spacing(8);
    if let Some(appid) = game.steam_appid {
        let steam_color = Color::from_rgb(0.4, 0.6, 0.9);
        let steam_badge = button(
            text(format!("Steam #{}", appid))
                .size(11)
                .color(steam_color),
        )
        .on_press(Message::OpenUrl(format!(
            "https://store.steampowered.com/app/{}",
            appid
        )))
        .padding([4, 10])
        .style(move |_theme: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let bg = if hover {
                Color::from_rgb(0.14, 0.14, 0.2)
            } else {
                Color::from_rgb(0.1, 0.1, 0.15)
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: steam_color,
                border: Border {
                    radius: 12.0.into(),
                    color: Color::from_rgb(0.2, 0.3, 0.5),
                    width: 1.0,
                },
                ..button::Style::default()
            }
        });
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
        vt.date_format.format_date(&game.release_date)
    };

    let mut info_col = column![
        info_row("Genre", &game.genre),
        info_row("Release Date", &release_str),
        info_row("Last Played", vt.date_format.format_date(&game.last_played)),
        info_row("Playtime", &playtime_str),
        info_row("Rating", &rating_str),
        info_row("Reviews", &review_str),
    ]
    .spacing(8);

    // Description
    if !game.description.is_empty() {
        info_col = info_col.push(rule::horizontal(1));
        info_col = info_col.push(
            text(&game.description)
                .size(12)
                .color(theme::TEXT_SECONDARY),
        );
    }

    // Tags
    if !game.tags.is_empty() {
        info_col = info_col.push(rule::horizontal(1));
        info_col = info_col.push(
            row![
                text("Tags: ").size(11).color(theme::TEXT_MUTED),
                text(&game.tags).size(11).color(theme::TEXT_SECONDARY),
            ]
            .spacing(4),
        );
    }

    let info_section = info_card("Game Info", vt, info_col);

    // Achievements section — summary + individual list
    let ach_display = app.settings.achievements_display;
    let achievements_section = if app.achievements_loading {
        info_card(
            "Achievements",
            vt,
            column![text("Loading achievements...")
                .size(14)
                .color(vt.text_muted),]
            .spacing(10),
        )
    } else if ach_display == AchievementsDisplay::Hidden {
        // Hidden: don't show achievements at all
        container(text("").size(1)).into()
    } else if game.achievements_total > 0 {
        let pct = game.achievement_percent();
        let mut ach_col = column![
            row![
                text(format!(
                    "{} / {}",
                    game.achievements_unlocked, game.achievements_total
                ))
                .size(20)
                .color(Color::WHITE),
                space::horizontal(),
                text(format!("{:.1}%", pct))
                    .size(20)
                    .color(if pct >= 100.0 {
                        theme::ACCENT_GOLD
                    } else {
                        theme::ACCENT_BLUE
                    }),
            ]
            .align_y(iced::Alignment::Center),
            progress_bar(0.0..=100.0, pct).girth(12),
            text(if pct >= 100.0 {
                "All achievements unlocked!"
            } else {
                "Keep going!"
            })
            .size(12)
            .color(theme::TEXT_MUTED),
        ]
        .spacing(10);

        // Individual achievement list (only in Full mode)
        if ach_display == AchievementsDisplay::Full && !app.achievements.is_empty() {
            ach_col = ach_col.push(rule::horizontal(1));

            for ach in &app.achievements {
                let (name_color, desc_color) = if ach.unlocked {
                    (Color::WHITE, theme::TEXT_SECONDARY)
                } else {
                    (theme::TEXT_MUTED, theme::TEXT_DIM)
                };

                // Achievement icon: show colored icon if unlocked, gray if locked
                // Falls back to colored icon for locked if gray is unavailable
                let icon_element: Element<'_, Message> = if let Some(appid) = game.steam_appid {
                    let gray_path =
                        crate::images::achievement_icon_path(appid, &ach.api_name, true);
                    let colored_path =
                        crate::images::achievement_icon_path(appid, &ach.api_name, false);
                    let icon_path = if ach.unlocked {
                        // Unlocked: use colored icon
                        colored_path
                    } else if gray_path.exists() {
                        // Locked: prefer gray icon
                        gray_path
                    } else {
                        // Locked but no gray icon: fall back to colored
                        colored_path
                    };
                    if icon_path.exists() {
                        let img =
                            iced::widget::image(iced::widget::image::Handle::from_path(&icon_path))
                                .width(48)
                                .height(48);
                        let is_locked = !ach.unlocked;
                        container(img)
                            .width(48)
                            .height(48)
                            .style(move |_: &Theme| container::Style {
                                background: if is_locked {
                                    // Dim overlay for locked achievements using colored icon
                                    Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.3)))
                                } else {
                                    None
                                },
                                border: Border {
                                    radius: 4.0.into(),
                                    ..Border::default()
                                },
                                ..container::Style::default()
                            })
                            .into()
                    } else {
                        // Fallback: colored dot while icons are downloading
                        let dot = if ach.unlocked {
                            Color::from_rgb(0.2, 0.9, 0.4)
                        } else {
                            Color::from_rgb(0.35, 0.35, 0.4)
                        };
                        container(text("").size(1))
                            .width(48)
                            .height(48)
                            .center_x(48)
                            .center_y(48)
                            .style(move |_: &Theme| container::Style {
                                background: Some(Background::Color(dot)),
                                border: Border {
                                    radius: 4.0.into(),
                                    ..Border::default()
                                },
                                ..container::Style::default()
                            })
                            .into()
                    }
                } else {
                    // Non-Steam game: fallback dot
                    let dot = if ach.unlocked {
                        Color::from_rgb(0.2, 0.9, 0.4)
                    } else {
                        Color::from_rgb(0.35, 0.35, 0.4)
                    };
                    container(text("").size(1))
                        .width(48)
                        .height(48)
                        .center_x(48)
                        .center_y(48)
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
                    column![text(&ach.display_name).size(13).color(name_color),].spacing(2);

                if !ach.description.is_empty() {
                    ach_info = ach_info.push(text(&ach.description).size(11).color(desc_color));
                }

                // Show unlock date for unlocked achievements
                if ach.unlocked && ach.unlock_time > 0 {
                    let datetime = vt.date_format.format_timestamp(ach.unlock_time);
                    if !datetime.is_empty() {
                        ach_info = ach_info.push(
                            text(format!("Unlocked {}", datetime))
                                .size(10)
                                .color(theme::SUCCESS),
                        );
                    }
                }

                let ach_row = row![icon_element, ach_info]
                    .spacing(12)
                    .align_y(iced::Alignment::Center);

                let ach_bg = vt.bg_row;
                ach_col = ach_col.push(
                    container(ach_row)
                        .padding([6, 8])
                        .width(Length::Fill)
                        .style(move |_: &Theme| container::Style {
                            background: Some(Background::Color(ach_bg)),
                            border: Border {
                                radius: 4.0.into(),
                                ..Border::default()
                            },
                            ..container::Style::default()
                        }),
                );
            }
        }

        info_card("Achievements", vt, ach_col)
    } else {
        info_card(
            "Achievements",
            vt,
            column![text("No achievements available for this game")
                .size(14)
                .color(vt.text_muted),]
            .spacing(10),
        )
    };

    // Notes section
    let notes_len = game.notes.len();
    let (notes_counter_color, notes_hint) = if notes_len >= 2000 {
        (Color::from_rgb(1.0, 0.3, 0.3), " - limit reached")
    } else if notes_len > 1800 {
        (Color::from_rgb(1.0, 0.5, 0.3), " - approaching limit")
    } else {
        (theme::TEXT_DIM, "")
    };
    let notes_section = info_card(
        "Notes",
        vt,
        column![
            text_input("Add notes about this game...", &game.notes)
                .on_input(move |s| Message::SetGameNotes(game_id, s))
                .padding(8)
                .size(13),
            text(format!("{} / 2000{}", notes_len, notes_hint))
                .size(11)
                .color(notes_counter_color),
        ]
        .spacing(8),
    );

    // Status change buttons (respect large_click_targets setting)
    let bp = vt.btn_padding;
    let status_section = info_card(
        "Change Status",
        vt,
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
            button(text("Clear").size(11).color(theme::TEXT_MUTED))
                .on_press(Message::SetGameRating(game_id, None))
                .padding([6, 10])
                .style(theme::outline_btn_style(theme::TEXT_MUTED, vt.bg_card))
                .into(),
        );
    }
    let rating_section = info_card("Rate this Game", vt, row(rating_items).spacing(4));

    // Top bar: Back button (left) + Delete button (right)
    let delete_btn = button(
        text(format!("{} Delete", theme::icons::DELETE))
            .size(13)
            .color(theme::DANGER),
    )
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

fn info_card<'a>(
    title: &'a str,
    vt: ViewTheme,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let section = column![text(title).size(14).color(vt.text_muted), content.into(),].spacing(8);

    let bg = vt.bg_card;
    let border = vt.border;
    container(section)
        .padding(16)
        .width(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                radius: 8.0.into(),
                color: border,
                width: 1.0,
            },
            ..container::Style::default()
        })
        .into()
}

fn info_row<'a>(label: &'a str, value: impl ToString) -> Element<'a, Message> {
    row![
        text(label).size(13).color(theme::TEXT_MUTED).width(120),
        text(value.to_string()).size(13).color(Color::WHITE),
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
    let is_active = current == target;
    let color = target.color();

    button(text(label).size(12).color(color))
        .on_press(Message::SetGameStatus(game_id, target))
        .padding([pad, pad + 4])
        .style(move |_theme: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let bg = if is_active {
                Color::from_rgb(0.15, 0.18, 0.25)
            } else {
                theme::BG_DARK
            };
            let bg = if hover {
                Color::from_rgb(bg.r + 0.04, bg.g + 0.04, bg.b + 0.04)
            } else {
                bg
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: color,
                border: Border {
                    radius: 6.0.into(),
                    color: if is_active { color } else { theme::BORDER_CARD },
                    width: 1.0,
                },
                ..button::Style::default()
            }
        })
        .into()
}

fn rating_button<'a>(
    value: u8,
    current: Option<u8>,
    game_id: i64,
    pad: u16,
) -> Element<'a, Message> {
    let is_active = current.is_some_and(|r| value <= r);
    let color = if is_active {
        theme::ACCENT_GOLD
    } else {
        theme::TEXT_HEADING
    };

    button(text(format!("{}", value)).size(13).color(color))
        .on_press(Message::SetGameRating(game_id, Some(value)))
        .padding([pad, pad + 4])
        .style(move |_theme: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let bg = if is_active {
                Color::from_rgb(0.2, 0.18, 0.08)
            } else {
                theme::BG_DARK
            };
            let bg = if hover {
                Color::from_rgb(bg.r + 0.04, bg.g + 0.04, bg.b + 0.04)
            } else {
                bg
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: color,
                border: Border {
                    radius: 4.0.into(),
                    color: if is_active {
                        Color::from_rgb(0.5, 0.42, 0.15)
                    } else {
                        theme::BORDER_CARD
                    },
                    width: 1.0,
                },
                ..button::Style::default()
            }
        })
        .into()
}
