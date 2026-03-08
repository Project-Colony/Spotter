use iced::widget::{button, column, container, rule, space, text};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::{Message, Screen, Spotter};
use crate::theme::{self, ViewTheme};

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let vt = ViewTheme::from_settings(&app.settings);
    let accent = app.settings.accent_color.color();
    let is_settings = app.screen == Screen::Settings;
    let large = app.settings.large_click_targets;

    let title_label = format!("{}  SPOTTER", theme::icons::GAMEPAD);
    let title_text = text(title_label)
        .size(22)
        .font(theme::FONT_BOLD)
        .color(if is_settings { Color::WHITE } else { accent });

    let title = button(title_text)
        .on_press(Message::NavigateTo(Screen::Settings))
        .padding(0)
        .style(move |_: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                text_color: accent,
                border: if hover {
                    Border {
                        width: 0.0,
                        color: accent,
                        radius: 0.0.into(),
                    }
                } else {
                    Border::default()
                },
                shadow: if hover {
                    iced::Shadow {
                        color: accent,
                        offset: iced::Vector::new(0.0, 2.0),
                        blur_radius: 0.0,
                    }
                } else {
                    iced::Shadow::default()
                },
                ..button::Style::default()
            }
        });

    let subtitle = text("Game Tracker").size(12).color(vt.text_muted);

    let total_games = text(format!("{} games", app.games.len()))
        .size(11)
        .color(vt.text_dim);

    let nav_pad = if large { 12 } else { 8 };
    let nav_library = nav_button(
        &format!("{}  Library", theme::icons::LIBRARY),
        Screen::Library,
        &app.screen,
        accent,
        nav_pad,
        vt,
    );
    let nav_add_game = nav_button(
        &format!("{}  Add Game", theme::icons::ADD),
        Screen::AddGame,
        &app.screen,
        accent,
        nav_pad,
        vt,
    );
    let nav_stats = nav_button(
        &format!("{}  Statistics", theme::icons::STATS),
        Screen::Statistics,
        &app.screen,
        accent,
        nav_pad,
        vt,
    );
    let nav_import = nav_button(
        &format!("{}  Import", theme::icons::IMPORT),
        Screen::Import,
        &app.screen,
        accent,
        nav_pad,
        vt,
    );
    let nav_profile = nav_button(
        &format!("{}  Profile", theme::icons::PROFILE),
        Screen::Profile,
        &app.screen,
        accent,
        nav_pad,
        vt,
    );

    let playing_count = app
        .games
        .iter()
        .filter(|g| g.status == crate::models::GameStatus::Playing)
        .count();
    let completed_count = app
        .games
        .iter()
        .filter(|g| g.status == crate::models::GameStatus::Completed)
        .count();
    let backlog_count = app
        .games
        .iter()
        .filter(|g| g.status == crate::models::GameStatus::Unplayed)
        .count();

    let fav_count = app.settings.favorites.len();
    let mut quick_stats = column![
        text("Quick Stats").size(11).color(vt.text_muted),
        text(format!("  Playing: {}", playing_count))
            .size(12)
            .color(theme::SUCCESS),
        text(format!("  Completed: {}", completed_count))
            .size(12)
            .color(theme::COMPLETED_BLUE),
        text(format!("  Unplayed: {}", backlog_count))
            .size(12)
            .color(Color::from_rgb(0.9, 0.7, 0.2)),
    ]
    .spacing(4);

    if fav_count > 0 {
        quick_stats = quick_stats.push(
            text(format!("  {} Favorites: {}", theme::icons::STAR, fav_count))
                .size(12)
                .color(theme::ACCENT_GOLD),
        );
    }

    let user_badge = container(
        text(format!(
            "{} {}",
            theme::icons::PROFILE,
            app.profile.username
        ))
        .size(12)
        .color(Color::from_rgb(0.7, 0.8, 1.0)),
    )
    .padding([4, 8])
    .style(move |_theme: &Theme| container::Style {
        background: Some(Background::Color(vt.bg_card)),
        border: Border {
            radius: 8.0.into(),
            color: vt.border,
            width: 1.0,
        },
        ..container::Style::default()
    });

    let sidebar_content = column![
        title,
        subtitle,
        total_games,
        rule::horizontal(1),
        nav_library,
        nav_add_game,
        nav_stats,
        nav_import,
        nav_profile,
        rule::horizontal(1),
        quick_stats,
        space::vertical(),
        user_badge,
    ]
    .spacing(12)
    .padding(20);

    let sidebar_width = app.settings.sidebar_width.clamp(160, 400) as f32;
    let bg_sidebar = vt.bg_sidebar;
    let sidebar = container(sidebar_content)
        .width(Length::Fixed(sidebar_width))
        .height(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(bg_sidebar)),
            border: Border {
                color: Color::from_rgb(0.15, 0.15, 0.2),
                width: 0.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

    iced::widget::row![sidebar, rule::vertical(1)].into()
}

fn nav_button(
    label: &str,
    target: Screen,
    current: &Screen,
    accent: Color,
    pad: u16,
    vt: ViewTheme,
) -> Element<'static, Message> {
    let label = label.to_string();
    let is_active = *current == target
        || matches!((current, &target), (Screen::GameDetail(_), Screen::Library));

    let label_color = if is_active {
        accent
    } else {
        theme::TEXT_SECONDARY
    };

    let label_text = text(label).size(14).color(label_color);

    let active_bg = vt.bg_card;
    button(label_text)
        .on_press(Message::NavigateTo(target))
        .padding([pad, 14])
        .width(Length::Fill)
        .style(move |_theme: &Theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let bg = if is_active {
                if hover {
                    crate::theme::lighten(active_bg, 0.04)
                } else {
                    active_bg
                }
            } else if hover {
                Color::from_rgba(1.0, 1.0, 1.0, 0.04)
            } else {
                Color::TRANSPARENT
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: label_color,
                border: Border {
                    radius: 8.0.into(),
                    color: if is_active {
                        accent
                    } else {
                        Color::TRANSPARENT
                    },
                    width: if is_active { 1.0 } else { 0.0 },
                },
                ..button::Style::default()
            }
        })
        .into()
}
