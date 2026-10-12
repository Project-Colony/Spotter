use colony_ui::theme::Palette;
use colony_ui::widgets::{identity_button, nav_hint, nav_label, selection_style};
use colony_ui::Typography;
use iced::widget::{button, column, container, rule, space};
use iced::{Background, Element, Length, Theme};

use crate::app::{Message, Screen, Spotter};
use crate::i18n::{t, tf, tn};
use crate::messages::PreferencesMessage;
use crate::models::GameStatus;
use crate::theme::{self, ui_text};

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let typo = Typography::current();
    // The program's name is the way into Preferences, and back out.
    let identity = identity_button(
        &typo,
        "Spotter",
        24,
        app.screen == Screen::Preferences,
        PreferencesMessage::Toggle.into(),
    );
    let tagline = ui_text(t("app_tagline"), 12).color(Palette::TEXT_MUTED());

    let nav_pad = app.preferences.button_padding();
    let nav = |icon: &str, key: &str, target: Screen| {
        nav_button(&format!("{icon}  {}", t(key)), target, &app.screen, nav_pad)
    };
    let modifier = if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    };

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

    let fav_count = app.preferences.favorites.len();
    let stat = |name: &str, count: usize| {
        ui_text(
            format!(
                "  {}",
                tf("quick_stat", &[("name", &name), ("count", &count)])
            ),
            12,
        )
    };
    let mut quick_stats = column![
        ui_text(t("quick_stats"), 11).color(Palette::TEXT_MUTED()),
        ui_text(tn("games_count", app.games.len(), &[]), 11).color(Palette::TEXT_DIM()),
        stat(GameStatus::Playing.label(), playing_count).color(GameStatus::Playing.color()),
        stat(GameStatus::Completed.label(), completed_count).color(GameStatus::Completed.color()),
        stat(GameStatus::Unplayed.label(), backlog_count).color(GameStatus::Unplayed.color()),
    ]
    .spacing(4);

    if fav_count > 0 {
        quick_stats = quick_stats.push(
            stat(
                &format!("{} {}", theme::icons::STAR, t("favorites")),
                fav_count,
            )
            .color(Palette::WARNING()),
        );
    }

    let user_badge = container(
        ui_text(
            format!("{} {}", theme::icons::PROFILE, app.profile.username),
            12,
        )
        .color(Palette::TEXT_SECONDARY()),
    )
    .padding([4, 8])
    .style(theme::card_style(
        Palette::BG_CARD(),
        Palette::BORDER_SUBTLE(),
        8.0,
    ));

    let sidebar_content = column![
        column![identity, tagline].spacing(4),
        rule::horizontal(1),
        nav_label(&typo, t("nav_label")),
        column![
            nav(theme::icons::LIBRARY, "nav_library", Screen::Library),
            nav(theme::icons::ADD, "nav_add_game", Screen::AddGame),
            nav(theme::icons::STATS, "nav_statistics", Screen::Statistics),
            nav(theme::icons::IMPORT, "nav_import", Screen::Import),
            nav(theme::icons::PROFILE, "nav_profile", Screen::Profile),
        ]
        .spacing(4),
        nav_hint(&typo, &tf("nav_hint", &[("modifier", &modifier)])),
        rule::horizontal(1),
        quick_stats,
        space::vertical(),
        user_badge,
    ]
    .spacing(12)
    .padding(20);

    let sidebar_width = app.preferences.sidebar_width.clamp(160, 400) as f32;
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
    button(ui_text(label.to_string(), 14))
        .on_press(Message::NavigateTo(target))
        .padding([pad, 14])
        .width(Length::Fill)
        .style(selection_style(is_active))
        .into()
}
