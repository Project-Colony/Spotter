use iced::widget::{
    button, column, container, progress_bar, row, rule, scrollable, space, text, text_input,
};
use iced::{Background, Border, Color, Element, Length, Theme};

use std::collections::HashSet;

use crate::app::{Message, Screen, Spotter};
use crate::models::{GameStatus, Platform, SortOrder};
use crate::theme::{self, ViewTheme};

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let vt = ViewTheme::from_settings(&app.settings);

    let header = column![
        text(format!("{} Game Library", theme::icons::LIBRARY))
            .size(24)
            .font(theme::FONT_BOLD)
            .color(Color::WHITE),
        text("Track your games across all platforms")
            .size(12)
            .color(vt.text_muted),
    ]
    .spacing(4);

    let search = text_input(
        "Search games by title, genre, or tags...",
        &app.search_query,
    )
    .id(iced::widget::Id::new("library_search"))
    .on_input(Message::SearchChanged)
    .padding(10)
    .size(14);

    let bp = vt.btn_padding;

    // Status filter chips (with icons)
    let s_playing = format!("{} Playing", GameStatus::Playing.icon());
    let s_completed = format!("{} Completed", GameStatus::Completed.icon());
    let s_unplayed = format!("{} Unplayed", GameStatus::Unplayed.icon());
    let s_dropped = format!("{} Dropped", GameStatus::Dropped.icon());
    let s_wishlist = format!("{} Wishlist", GameStatus::Wishlist.icon());

    let status_buttons = row![
        filter_chip("All", app.filter_status.is_none(), None, bp),
        filter_chip(
            &s_playing,
            app.filter_status == Some(GameStatus::Playing),
            Some(GameStatus::Playing),
            bp
        ),
        filter_chip(
            &s_completed,
            app.filter_status == Some(GameStatus::Completed),
            Some(GameStatus::Completed),
            bp
        ),
        filter_chip(
            &s_unplayed,
            app.filter_status == Some(GameStatus::Unplayed),
            Some(GameStatus::Unplayed),
            bp
        ),
        filter_chip(
            &s_dropped,
            app.filter_status == Some(GameStatus::Dropped),
            Some(GameStatus::Dropped),
            bp
        ),
        filter_chip(
            &s_wishlist,
            app.filter_status == Some(GameStatus::Wishlist),
            Some(GameStatus::Wishlist),
            bp
        ),
    ]
    .spacing(6);

    // Platform filter chips (with icons)
    let p_steam = format!("{} Steam", Platform::Steam.icon());
    let p_gog = format!("{} GOG", Platform::Gog.icon());
    let p_epic = format!("{} Epic", Platform::Epic.icon());
    let p_ps = format!("{} PlayStation", Platform::PlayStation.icon());
    let p_xbox = format!("{} Xbox", Platform::Xbox.icon());
    let p_nin = format!("{} Nintendo", Platform::Nintendo.icon());

    let platform_buttons = row![
        platform_chip("All", app.filter_platform.is_none(), None, bp),
        platform_chip(
            &p_steam,
            app.filter_platform == Some(Platform::Steam),
            Some(Platform::Steam),
            bp
        ),
        platform_chip(
            &p_gog,
            app.filter_platform == Some(Platform::Gog),
            Some(Platform::Gog),
            bp
        ),
        platform_chip(
            &p_epic,
            app.filter_platform == Some(Platform::Epic),
            Some(Platform::Epic),
            bp
        ),
        platform_chip(
            &p_ps,
            app.filter_platform == Some(Platform::PlayStation),
            Some(Platform::PlayStation),
            bp
        ),
        platform_chip(
            &p_xbox,
            app.filter_platform == Some(Platform::Xbox),
            Some(Platform::Xbox),
            bp
        ),
        platform_chip(
            &p_nin,
            app.filter_platform == Some(Platform::Nintendo),
            Some(Platform::Nintendo),
            bp
        ),
    ]
    .spacing(6);

    // Sort buttons
    let sort_row = row![
        text("Sort:").size(12).color(vt.text_muted),
        sort_chip("A-Z", SortOrder::TitleAsc, app.sort_order, vt.bg_card),
        sort_chip("Z-A", SortOrder::TitleDesc, app.sort_order, vt.bg_card),
        sort_chip(
            "Playtime",
            SortOrder::PlaytimeDesc,
            app.sort_order,
            vt.bg_card
        ),
        sort_chip("Rating", SortOrder::RatingDesc, app.sort_order, vt.bg_card),
        sort_chip(
            "Recent",
            SortOrder::LastPlayedDesc,
            app.sort_order,
            vt.bg_card
        ),
        sort_chip(
            &format!("{} Favs", theme::icons::STAR),
            SortOrder::FavoritesFirst,
            app.sort_order,
            vt.bg_card
        ),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    let filtered: Vec<(usize, &crate::models::Game)> = app
        .filtered_cache
        .iter()
        .filter_map(|&i| app.games.get(i).map(|g| (i, g)))
        .collect();

    // Bulk mode toggle + count
    let bulk_toggle = button(
        text(if app.bulk_mode {
            "Exit Select"
        } else {
            "Select"
        })
        .size(12)
        .color(if app.bulk_mode {
            theme::ACCENT_GOLD
        } else {
            theme::TEXT_MUTED
        }),
    )
    .on_press(Message::ToggleBulkMode)
    .padding([4, 10])
    .style(theme::chip_style(
        app.bulk_mode,
        theme::TEXT_MUTED,
        theme::ACCENT_GOLD,
        8.0,
    ));

    let count_text = row![
        text(format!("{} game(s)", filtered.len()))
            .size(12)
            .color(vt.text_muted),
        space::horizontal(),
        bulk_toggle,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Bulk action bar (shown when items are selected)
    let bulk_bar: Element<Message> = if app.bulk_mode && !app.bulk_selected.is_empty() {
        let sel_count = app.bulk_selected.len();
        let accent = app.settings.accent_color.color();
        let bar_bg = vt.bg_card;
        let bar_border = vt.border;
        container(
            row![
                text(format!("{} selected", sel_count))
                    .size(13)
                    .color(accent),
                button(text("All").size(11).color(theme::ACCENT_BLUE))
                    .on_press(Message::BulkSelectAll)
                    .padding([4, 10])
                    .style(theme::outline_btn_style(theme::ACCENT_BLUE, bar_bg)),
                button(text("None").size(11).color(theme::TEXT_MUTED))
                    .on_press(Message::BulkDeselectAll)
                    .padding([4, 10])
                    .style(theme::outline_btn_style(theme::TEXT_MUTED, bar_bg)),
                space::horizontal(),
                button(
                    text(format!("{} Playing", GameStatus::Playing.icon()))
                        .size(11)
                        .color(GameStatus::Playing.color()),
                )
                .on_press(Message::BulkSetStatus(GameStatus::Playing))
                .padding([4, 10])
                .style(theme::outline_btn_style(
                    GameStatus::Playing.color(),
                    bar_bg
                )),
                button(
                    text(format!("{} Completed", GameStatus::Completed.icon()))
                        .size(11)
                        .color(GameStatus::Completed.color()),
                )
                .on_press(Message::BulkSetStatus(GameStatus::Completed))
                .padding([4, 10])
                .style(theme::outline_btn_style(
                    GameStatus::Completed.color(),
                    bar_bg,
                )),
                button(
                    text(format!("{} Dropped", GameStatus::Dropped.icon()))
                        .size(11)
                        .color(GameStatus::Dropped.color()),
                )
                .on_press(Message::BulkSetStatus(GameStatus::Dropped))
                .padding([4, 10])
                .style(theme::outline_btn_style(
                    GameStatus::Dropped.color(),
                    bar_bg
                )),
                button(
                    text("Delete")
                        .size(11)
                        .color(Color::from_rgb(1.0, 0.4, 0.4))
                )
                .on_press(Message::BulkDelete)
                .padding([4, 10])
                .style(theme::outline_btn_style(
                    Color::from_rgb(1.0, 0.4, 0.4),
                    bar_bg,
                )),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        )
        .padding([8, 12])
        .width(Length::Fill)
        .style(theme::card_style(bar_bg, bar_border, 8.0))
        .into()
    } else {
        column![].into()
    };

    let show_covers = app.settings.show_covers_in_list;
    let compact = app.settings.compact_list;
    let favorites = &app.settings.favorites;
    let bulk_mode = app.bulk_mode;

    let game_list: Element<Message> = if filtered.is_empty() {
        // Build a helpful empty-state showing which filters are active
        let mut empty_col = column![text("No games match your filters")
            .size(18)
            .color(vt.text_muted),]
        .spacing(8)
        .align_x(iced::Alignment::Center);

        // Describe active filters
        let mut active_parts: Vec<String> = Vec::new();
        if !app.search_query.is_empty() {
            active_parts.push(format!("search \"{}\"", app.search_query));
        }
        if let Some(ref s) = app.filter_status {
            active_parts.push(format!("status = {}", s));
        }
        if let Some(ref p) = app.filter_platform {
            active_parts.push(format!("platform = {}", p));
        }
        if !active_parts.is_empty() {
            empty_col = empty_col.push(
                text(format!("Active filters: {}", active_parts.join(", ")))
                    .size(13)
                    .color(vt.text_dim),
            );
        }

        // Suggest clearing filters
        let mut hint_row = row![].spacing(6).align_y(iced::Alignment::Center);
        hint_row = hint_row.push(
            text("Try broadening your search or")
                .size(12)
                .color(vt.text_dim),
        );
        if app.filter_status.is_some() {
            hint_row = hint_row.push(
                button(
                    text("clear status filter")
                        .size(12)
                        .color(theme::ACCENT_BLUE),
                )
                .on_press(Message::FilterStatus(None))
                .padding([4, 8])
                .style(theme::transparent_btn_style()),
            );
        }
        if app.filter_platform.is_some() {
            hint_row = hint_row.push(
                button(
                    text("clear platform filter")
                        .size(12)
                        .color(theme::ACCENT_BLUE),
                )
                .on_press(Message::FilterPlatform(None))
                .padding([4, 8])
                .style(theme::transparent_btn_style()),
            );
        }
        if !app.search_query.is_empty() {
            hint_row = hint_row.push(
                button(text("clear search").size(12).color(theme::ACCENT_BLUE))
                    .on_press(Message::SearchChanged(String::new()))
                    .padding([4, 8])
                    .style(theme::transparent_btn_style()),
            );
        }
        empty_col = empty_col.push(hint_row);

        // Show "Clear all" if multiple filters are active
        let active_count = (!app.search_query.is_empty() as u8)
            + (app.filter_status.is_some() as u8)
            + (app.filter_platform.is_some() as u8);
        if active_count > 1 {
            empty_col = empty_col.push(
                button(text("Clear all filters").size(12).color(theme::ACCENT_BLUE))
                    .on_press(Message::ClearAllFilters)
                    .padding([6, 14])
                    .style(theme::outline_btn_style(theme::ACCENT_BLUE, vt.bg_card)),
            );
        }

        container(empty_col)
            .padding(40)
            .center_x(Length::Fill)
            .into()
    } else {
        let show_descriptions = app.settings.show_game_descriptions;
        let bulk_selected = &app.bulk_selected;
        let cover_cache = &app.cover_cache;
        let total_filtered = filtered.len();
        let limit = app.visible_card_limit;
        let cards: Vec<Element<Message>> = filtered
            .iter()
            .take(limit)
            .map(|(_index, game)| {
                game_card(
                    game,
                    show_covers,
                    compact,
                    show_descriptions,
                    vt,
                    favorites.contains(&game.id.unwrap_or(-1)),
                    bulk_mode,
                    game.id.is_some_and(|id| bulk_selected.contains(&id)),
                    cover_cache,
                )
            })
            .collect();

        let spacing = if compact { 4 } else { 8 };
        let mut game_column = column![].spacing(spacing).width(Length::Fill);
        for card in cards {
            game_column = game_column.push(card);
        }
        if total_filtered > limit {
            let remaining = total_filtered - limit;
            game_column = game_column.push(
                container(
                    button(
                        text(format!("Show more ({} remaining)", remaining))
                            .size(13)
                            .color(theme::ACCENT_BLUE),
                    )
                    .on_press(Message::ShowMoreCards)
                    .padding([8, 20])
                    .style(theme::outline_btn_style(theme::ACCENT_BLUE, vt.bg_card)),
                )
                .padding([12, 0])
                .center_x(Length::Fill),
            );
        }
        scrollable(game_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    // Onboarding banner for first-time users
    let onboarding: Element<Message> = if app.first_launch {
        let accent = app.settings.accent_color.color();
        let import_btn = button(text("Import Games").size(12).color(theme::BG_DARKER))
            .on_press(Message::NavigateTo(crate::app::Screen::Import))
            .padding([6, 16])
            .style(theme::primary_btn_style(theme::SUCCESS));
        let add_btn = button(text("Add Manually").size(12).color(accent))
            .on_press(Message::NavigateTo(crate::app::Screen::AddGame))
            .padding([6, 16])
            .style(theme::outline_btn_style(accent, vt.bg_card));
        let dismiss_btn = button(text("Dismiss").size(12).color(theme::TEXT_MUTED))
            .on_press(Message::DismissOnboarding)
            .padding([6, 16])
            .style(theme::transparent_btn_style());
        let onboard_card = container(
            column![
                text("Welcome to Spotter!").size(18).color(accent),
                text("Import from Steam, GOG, Epic, Xbox, PlayStation — or add games manually.")
                    .size(13)
                    .color(vt.text_secondary),
                row![import_btn, add_btn, dismiss_btn].spacing(8),
            ]
            .spacing(8),
        )
        .padding([14, 18])
        .width(Length::Fill)
        .style(theme::accent_card_style(vt.bg_card, accent));
        onboard_card.into()
    } else {
        column![].into()
    };

    let content = column![
        header,
        onboarding,
        search,
        status_buttons,
        platform_buttons,
        sort_row,
        count_text,
        bulk_bar,
        rule::horizontal(1),
        game_list,
    ]
    .spacing(12)
    .padding(24)
    .width(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn filter_chip(
    label: &str,
    is_active: bool,
    status: Option<GameStatus>,
    pad: u16,
) -> Element<'static, Message> {
    let label = label.to_string();
    let color = if is_active {
        theme::ACCENT_BLUE
    } else {
        theme::TEXT_MUTED
    };

    button(text(label).size(12).color(color))
        .on_press(Message::FilterStatus(status))
        .padding([pad, 12])
        .style(move |_theme: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let base_bg = if is_active {
                Color::from_rgb(0.15, 0.2, 0.3)
            } else {
                theme::BG_BUTTON
            };
            let bg = if hover {
                crate::theme::lighten(base_bg, 0.04)
            } else {
                base_bg
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: color,
                border: Border {
                    radius: 12.0.into(),
                    color: if is_active {
                        Color::from_rgb(0.3, 0.4, 0.6)
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

fn platform_chip(
    label: &str,
    is_active: bool,
    platform: Option<Platform>,
    pad: u16,
) -> Element<'static, Message> {
    let label = label.to_string();
    let color = if is_active {
        Color::from_rgb(0.8, 0.6, 1.0)
    } else {
        theme::TEXT_MUTED
    };

    button(text(label).size(11).color(color))
        .on_press(Message::FilterPlatform(platform))
        .padding([pad.saturating_sub(2), 10])
        .style(move |_theme: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let base_bg = if is_active {
                Color::from_rgb(0.18, 0.12, 0.28)
            } else {
                theme::BG_BUTTON
            };
            let bg = if hover {
                crate::theme::lighten(base_bg, 0.04)
            } else {
                base_bg
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: color,
                border: Border {
                    radius: 10.0.into(),
                    color: if is_active {
                        Color::from_rgb(0.4, 0.3, 0.6)
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

fn sort_chip(
    label: &str,
    target: SortOrder,
    current: SortOrder,
    bg_card: Color,
) -> Element<'static, Message> {
    let label = label.to_string();
    let is_active = current == target;
    let color = if is_active {
        theme::ACCENT_GOLD
    } else {
        theme::TEXT_DIM
    };

    button(text(label).size(11).color(color))
        .on_press(Message::SetSortOrder(target))
        .padding([4, 8])
        .style(move |_theme: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let base_bg = if is_active {
                Color::from_rgb(0.18, 0.16, 0.08)
            } else {
                bg_card
            };
            let bg = if hover {
                crate::theme::lighten(base_bg, 0.04)
            } else {
                base_bg
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: color,
                border: Border {
                    radius: 6.0.into(),
                    color: if is_active {
                        Color::from_rgb(0.5, 0.42, 0.15)
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

#[allow(clippy::too_many_arguments)]
fn game_card<'a>(
    game: &'a crate::models::Game,
    show_covers: bool,
    compact: bool,
    show_descriptions: bool,
    vt: ViewTheme,
    is_favorite: bool,
    bulk_mode: bool,
    is_selected: bool,
    cover_cache: &HashSet<String>,
) -> Element<'a, Message> {
    let status_color = game.status.color();
    let platform_color = game.platform.color();
    // Games without a DB id should not be navigable (they haven't been persisted yet)
    let game_id = match game.id {
        Some(id) => id,
        None => return iced::widget::text("...").into(),
    };

    let bg_card = vt.bg_card;
    let border_color = vt.border;

    // Cover image thumbnail (respects show_covers_in_list setting)
    let cover_element: Option<Element<'a, Message>> = if show_covers {
        let cover_path = crate::images::cover_path(&game.title);
        if cover_cache.contains(&game.title) {
            Some(
                container(
                    iced::widget::image(iced::widget::image::Handle::from_path(&cover_path))
                        .width(120)
                        .height(57),
                )
                .width(120)
                .into(),
            )
        } else {
            Some(
                container(
                    text(game.title.chars().next().unwrap_or('?').to_string())
                        .size(22)
                        .color(theme::TEXT_HEADING),
                )
                .width(120)
                .height(57)
                .center_x(120)
                .center_y(57)
                .style(move |_: &Theme| container::Style {
                    background: Some(Background::Color(bg_card)),
                    border: Border {
                        radius: 4.0.into(),
                        color: border_color,
                        width: 1.0,
                    },
                    ..container::Style::default()
                })
                .into(),
            )
        }
    } else {
        None
    };

    // Bulk selection checkbox
    let mut title_items: Vec<Element<'a, Message>> = Vec::new();
    if bulk_mode {
        let check_label = if is_selected { "[x]" } else { "[ ]" };
        let check_color = if is_selected {
            theme::ACCENT_BLUE
        } else {
            theme::TEXT_DIM
        };
        title_items.push(text(check_label).size(14).color(check_color).into());
    }

    // Favorite star
    if is_favorite {
        title_items.push(
            text(theme::icons::STAR)
                .size(14)
                .color(theme::ACCENT_GOLD)
                .into(),
        );
    }

    // Status label (respects show_status_labels setting)
    if vt.show_status_labels {
        title_items.push(
            text(format!("[{}]", game.status))
                .size(11)
                .color(status_color)
                .into(),
        );
    } else {
        // Show a small color dot instead
        title_items.push(
            container(text("").size(1))
                .width(8)
                .height(8)
                .style(move |_: &Theme| container::Style {
                    background: Some(Background::Color(status_color)),
                    border: Border {
                        radius: 4.0.into(),
                        ..Border::default()
                    },
                    ..container::Style::default()
                })
                .into(),
        );
    }
    title_items.push(text(&game.title).size(16).color(Color::WHITE).into());
    title_items.push(space::horizontal().into());
    title_items.push(
        text(format!("{} {}", game.platform.icon(), game.platform))
            .size(12)
            .color(platform_color)
            .into(),
    );

    let title_row = row(title_items).spacing(8).align_y(iced::Alignment::Center);

    let mut info_items = row![text(&game.genre).size(12).color(vt.text_muted),].spacing(0);

    if !game.release_date.is_empty() {
        info_items = info_items.push(
            text(format!(
                " · {}",
                vt.date_format.format_date(&game.release_date)
            ))
            .size(12)
            .color(vt.text_dim),
        );
    }

    if let Some(pct) = game.review_percent {
        let review_color = if pct >= 70 {
            theme::SUCCESS
        } else if pct >= 40 {
            Color::from_rgb(0.9, 0.7, 0.2)
        } else {
            Color::from_rgb(0.9, 0.3, 0.3)
        };
        info_items = info_items.push(
            text(format!(" · {}% positive", pct))
                .size(12)
                .color(review_color),
        );
    }

    info_items = info_items.push(
        text(format!(
            " · Last: {}",
            vt.date_format.format_date(&game.last_played)
        ))
        .size(12)
        .color(vt.text_dim),
    );

    let info_row = info_items;

    let playtime_text = text(format!("Playtime: {}", game.playtime_display()))
        .size(12)
        .color(vt.text_light);

    let stats_row = if game.achievements_total > 0 {
        let pct = game.achievement_percent();
        row![
            playtime_text,
            space::horizontal(),
            text(format!(
                "{}/{} ({:.0}%)",
                game.achievements_unlocked, game.achievements_total, pct
            ))
            .size(12)
            .color(vt.text_light),
            container(progress_bar(0.0..=100.0, pct).girth(6)).width(120),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
    } else {
        row![
            playtime_text,
            space::horizontal(),
            text("No achievements").size(12).color(vt.text_dim),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
    };

    let rating_text = match game.rating {
        Some(r) => {
            let filled = theme::icons::STAR.repeat(r as usize);
            let empty = theme::icons::STAR_O.repeat(10_usize.saturating_sub(r as usize));
            text(format!("{}{} {}/10", filled, empty, r))
                .size(11)
                .color(theme::ACCENT_GOLD)
        }
        None => text("Not rated").size(11).color(vt.text_dim),
    };

    let text_spacing = if compact { 3 } else { 6 };
    let mut text_info = column![title_row, info_row, stats_row, rating_text,].spacing(text_spacing);

    if show_descriptions && !game.description.is_empty() {
        let desc = if game.description.chars().count() > 120 {
            let truncated: String = game.description.chars().take(120).collect();
            format!("{}...", truncated)
        } else {
            game.description.clone()
        };
        text_info = text_info.push(text(desc).size(11).color(vt.text_dim));
    }

    let card_content = if let Some(cover) = cover_element {
        row![cover, text_info]
            .spacing(12)
            .align_y(iced::Alignment::Center)
    } else {
        row![text_info].spacing(0).align_y(iced::Alignment::Center)
    };

    let padding = if compact { 8 } else { 14 };
    let card = container(card_content)
        .padding(padding)
        .width(Length::Fill)
        .style(theme::card_style(bg_card, border_color, 8.0));

    let card_message = if bulk_mode {
        Message::ToggleBulkSelect(game_id)
    } else {
        Message::NavigateTo(Screen::GameDetail(game_id))
    };

    let selected_border = is_selected && bulk_mode;
    button(card)
        .on_press(card_message)
        .padding(0)
        .width(Length::Fill)
        .style(move |_: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: if selected_border {
                    Some(Background::Color(Color::from_rgba(0.6, 0.8, 1.0, 0.05)))
                } else if hover {
                    Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03)))
                } else {
                    None
                },
                border: if selected_border {
                    Border {
                        radius: 8.0.into(),
                        color: Color::from_rgba(0.6, 0.8, 1.0, 0.3),
                        width: 1.0,
                    }
                } else {
                    Border {
                        radius: 8.0.into(),
                        ..Border::default()
                    }
                },
                ..button::Style::default()
            }
        })
        .into()
}
