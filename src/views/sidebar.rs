use colony_ui::theme::Palette;
use colony_ui::widgets::selection_style;
use iced::widget::{button, column, container, rule, space, text};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::{Message, Screen, Spotter};
use crate::models::GameStatus;
use crate::theme;

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let accent = Palette::ACCENT();
    let is_settings = app.screen == Screen::Settings;

    let title_label = format!("{}  SPOTTER", theme::icons::GAMEPAD);
    let title_text = text(title_label)
        .size(22)
        .font(colony_ui::ui_font_bold())
        .color(if is_settings {
            Palette::TEXT_PRIMARY()
        } else {
            accent
        });

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

    let subtitle = text("Game Tracker").size(12).color(Palette::TEXT_MUTED());

    let total_games = text(format!("{} games", app.games.len()))
        .size(11)
        .color(Palette::TEXT_DIM());

    let nav_pad = app.settings.button_padding();
    let nav_library = nav_button(
        &format!("{}  Library", theme::icons::LIBRARY),
        Screen::Library,
        &app.screen,
        nav_pad,
    );
    let nav_add_game = nav_button(
        &format!("{}  Add Game", theme::icons::ADD),
        Screen::AddGame,
        &app.screen,
        nav_pad,
    );
    let nav_stats = nav_button(
        &format!("{}  Statistics", theme::icons::STATS),
        Screen::Statistics,
        &app.screen,
        nav_pad,
    );
    let nav_import = nav_button(
        &format!("{}  Import", theme::icons::IMPORT),
        Screen::Import,
        &app.screen,
        nav_pad,
    );
    let nav_profile = nav_button(
        &format!("{}  Profile", theme::icons::PROFILE),
        Screen::Profile,
        &app.screen,
        nav_pad,
    );

    let playing_count = app
        .games
        .iter()
        .filter(|g| g.status == GameStatus::Playing)
        .count();
    let completed_count = app
        .games
        .iter()
        .filter(|g| g.status == GameStatus::Completed)
        .count();
    let backlog_count = app
        .games
        .iter()
        .filter(|g| g.status == GameStatus::Unplayed)
        .count();

    let fav_count = app.settings.favorites.len();
    let mut quick_stats = column![
        text("Quick Stats").size(11).color(Palette::TEXT_MUTED()),
        text(format!("  Playing: {}", playing_count))
            .size(12)
            .color(GameStatus::Playing.color()),
        text(format!("  Completed: {}", completed_count))
            .size(12)
            .color(GameStatus::Completed.color()),
        text(format!("  Unplayed: {}", backlog_count))
            .size(12)
            .color(GameStatus::Unplayed.color()),
    ]
    .spacing(4);

    if fav_count > 0 {
        quick_stats = quick_stats.push(
            text(format!("  {} Favorites: {}", theme::icons::STAR, fav_count))
                .size(12)
                .color(Palette::WARNING()),
        );
    }

    let user_badge = container(
        text(format!(
            "{} {}",
            theme::icons::PROFILE,
            app.profile.username
        ))
        .size(12)
        .color(Palette::TEXT_SECONDARY()),
    )
    .padding([4, 8])
    .style(theme::card_style(
        Palette::BG_CARD(),
        Palette::BORDER_SUBTLE(),
        8.0,
    ));

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
    let sidebar = container(sidebar_content)
        .width(Length::Fixed(sidebar_width))
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Palette::BG_SIDEBAR())),
            ..container::Style::default()
        });

    iced::widget::row![sidebar, rule::vertical(1)].into()
}

fn nav_button(
    label: &str,
    target: Screen,
    current: &Screen,
    pad: u16,
) -> Element<'static, Message> {
    let is_active = *current == target
        || matches!((current, &target), (Screen::GameDetail(_), Screen::Library));

    // Selection is shown by background, as in every Colony sidebar.
    button(text(label.to_string()).size(14))
        .on_press(Message::NavigateTo(target))
        .padding([pad, 14])
        .width(Length::Fill)
        .style(selection_style(is_active))
        .into()
}
