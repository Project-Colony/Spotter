use iced::widget::{column, container, progress_bar, row, rule, space, text};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::{Message, Spotter};
use crate::models::{GameStatus, Platform};
use crate::theme::{self, ViewTheme};

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let vt = ViewTheme::from_settings(&app.settings);

    let header = column![
        text(format!("{} Statistics", theme::icons::STATS))
            .size(24)
            .font(theme::FONT_BOLD)
            .color(Color::WHITE),
        text("Overview of your gaming journey")
            .size(12)
            .color(vt.text_muted),
    ]
    .spacing(4);

    let total_playtime: u32 = app.games.iter().map(|g| g.playtime_minutes).sum();
    let total_achievements: u32 = app.games.iter().map(|g| g.achievements_unlocked).sum();
    let total_possible: u32 = app.games.iter().map(|g| g.achievements_total).sum();
    let avg_rating: f32 = {
        // Use fold to compute average without allocating a Vec
        let (count, sum) = app
            .games
            .iter()
            .filter_map(|g| g.rating)
            .fold((0u32, 0u32), |(c, s), r| (c + 1, s + r as u32));
        if count == 0 {
            0.0
        } else {
            sum as f32 / count as f32
        }
    };

    // Overview cards
    let overview = row![
        stat_card(
            "Total Games",
            format!("{}", app.games.len()),
            theme::ACCENT_BLUE,
            vt
        ),
        stat_card(
            "Total Playtime",
            format_playtime(total_playtime),
            theme::SUCCESS,
            vt
        ),
        stat_card(
            "Achievements",
            format!("{}/{}", total_achievements, total_possible),
            theme::ACCENT_GOLD,
            vt
        ),
        stat_card(
            "Avg Rating",
            format!("{:.1}/10", avg_rating),
            Color::from_rgb(0.9, 0.5, 0.9),
            vt
        ),
    ]
    .spacing(12);

    // Status distribution
    let status_dist = status_distribution(app, vt);

    // Platform distribution
    let platform_dist = platform_distribution(app, vt);

    // Playtime graph
    let playtime_graph = playtime_chart(app, vt);

    // Most played games
    let most_played = most_played_games(app, vt);

    // Period comparison
    let comparison = period_comparison(app, vt);

    let content = column![
        header,
        rule::horizontal(1),
        overview,
        playtime_graph,
        comparison,
        row![status_dist, platform_dist].spacing(12),
        most_played,
    ]
    .spacing(16)
    .padding(24)
    .width(Length::Fill);

    iced::widget::scrollable(container(content).width(Length::Fill).height(Length::Fill)).into()
}

fn stat_card<'a>(
    label: &'a str,
    value: String,
    color: Color,
    vt: ViewTheme,
) -> Element<'a, Message> {
    let content = column![
        text(label).size(12).color(vt.text_muted),
        text(value).size(24).color(color),
    ]
    .spacing(6);

    let bg = vt.bg_card;
    let border = vt.border;
    container(content)
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

fn section_card<'a>(
    title: &'a str,
    content: iced::widget::Column<'a, Message>,
    vt: ViewTheme,
) -> Element<'a, Message> {
    let section = column![
        text(title).size(16).color(vt.text_light),
        rule::horizontal(1),
        content,
    ]
    .spacing(10);

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

fn playtime_chart<'a>(app: &'a Spotter, vt: ViewTheme) -> Element<'a, Message> {
    let max_height: f32 = 120.0;

    let is_sample_data = app.playtime_data.is_empty();
    // Use Cow to avoid cloning playtime_data on every frame render
    let data: std::borrow::Cow<'_, [(String, u32)]> = if !is_sample_data {
        std::borrow::Cow::Borrowed(&app.playtime_data)
    } else {
        std::borrow::Cow::Owned(generate_sample_playtime(app))
    };

    if data.is_empty() {
        return section_card(
            "Playtime (Last 30 Days)",
            column![
                text("No playtime data yet. Import games to start tracking!")
                    .size(13)
                    .color(vt.text_muted),
            ],
            vt,
        );
    }

    let max_minutes = data.iter().map(|(_, m)| *m).max().unwrap_or(1).max(1);

    let mut bars_row = row![].spacing(3);
    for (date, minutes) in data.iter() {
        let bar_height = (*minutes as f32 / max_minutes as f32) * max_height;
        let bar_height = bar_height.max(2.0);

        let bar = container(column![])
            .width(Length::Fill)
            .height(Length::Fixed(bar_height))
            .style(move |_: &Theme| container::Style {
                background: Some(Background::Color(theme::COMPLETED_BLUE)),
                border: Border {
                    radius: 2.0.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            });

        let label = if date.len() >= 5 {
            date[5..].to_string()
        } else {
            date.clone()
        };

        let bar_with_label = column![
            space::vertical().height(Length::Fixed(max_height - bar_height)),
            bar,
            text(label).size(8).color(theme::TEXT_DIM),
        ]
        .align_x(iced::Alignment::Center)
        .width(Length::Fill);

        bars_row = bars_row.push(bar_with_label);
    }

    let total_tracked: u32 = data.iter().map(|(_, m)| *m).sum();
    let subtitle = text(format!(
        "Total: {} tracked over {} days",
        format_playtime(total_tracked),
        data.len()
    ))
    .size(11)
    .color(vt.text_muted);

    let mut chart_content = column![
        container(bars_row)
            .height(Length::Fixed(max_height + 20.0))
            .width(Length::Fill),
        subtitle,
    ]
    .spacing(8);

    if is_sample_data {
        chart_content = chart_content.push(
            text("Estimated from currently playing games (no recorded playtime data yet)")
                .size(10)
                .color(theme::TEXT_DIM),
        );
    }

    section_card("Playtime (Last 30 Days)", chart_content, vt)
}

fn generate_sample_playtime(app: &Spotter) -> Vec<(String, u32)> {
    let mut data = Vec::new();
    let now = chrono::Local::now();

    for i in (0..14).rev() {
        let date = now - chrono::Duration::days(i);
        let date_str = date.format("%Y-%m-%d").to_string();

        let playing_count = app
            .games
            .iter()
            .filter(|g| g.status == GameStatus::Playing)
            .count() as u32;

        if playing_count > 0 {
            let base_minutes = 30 + ((i * 17 + 7) % 90);
            let minutes = base_minutes as u32 * playing_count;
            data.push((date_str, minutes));
        }
    }
    data
}

fn status_distribution<'a>(app: &'a Spotter, vt: ViewTheme) -> Element<'a, Message> {
    let total = app.games.len() as f32;
    let statuses = GameStatus::all();

    let mut bars = column![].spacing(8);
    for &status in statuses {
        let count = app.games.iter().filter(|g| g.status == status).count();
        let pct = if total > 0.0 {
            (count as f32 / total) * 100.0
        } else {
            0.0
        };
        let bar_row = row![
            text(format!("{}", status))
                .size(12)
                .color(status.color())
                .width(90),
            container(progress_bar(0.0..=100.0, pct).girth(10)).width(Length::Fill),
            text(format!("{} ({:.0}%)", count, pct))
                .size(12)
                .color(vt.text_secondary)
                .width(80),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);
        bars = bars.push(bar_row);
    }

    section_card("Status Distribution", bars, vt)
}

fn platform_distribution<'a>(app: &'a Spotter, vt: ViewTheme) -> Element<'a, Message> {
    let total = app.games.len() as f32;
    let platforms = Platform::all();

    let mut bars = column![].spacing(8);
    for &platform in platforms {
        let count = app.games.iter().filter(|g| g.platform == platform).count();
        if count == 0 {
            continue;
        }
        let pct = if total > 0.0 {
            (count as f32 / total) * 100.0
        } else {
            0.0
        };
        let bar_row = row![
            text(format!("{}", platform))
                .size(12)
                .color(platform.color())
                .width(90),
            container(progress_bar(0.0..=100.0, pct).girth(10)).width(Length::Fill),
            text(format!("{} ({:.0}%)", count, pct))
                .size(12)
                .color(vt.text_secondary)
                .width(80),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);
        bars = bars.push(bar_row);
    }

    section_card("Platform Distribution", bars, vt)
}

fn most_played_games<'a>(app: &'a Spotter, vt: ViewTheme) -> Element<'a, Message> {
    let mut sorted: Vec<&crate::models::Game> = app.games.iter().collect();
    sorted.sort_by(|a, b| b.playtime_minutes.cmp(&a.playtime_minutes));

    let max_time = sorted.first().map_or(1, |g| g.playtime_minutes) as f32;

    let mut rows = column![].spacing(6);
    for game in sorted.iter().take(5) {
        let pct = (game.playtime_minutes as f32 / max_time) * 100.0;
        let game_row = row![
            text(&game.title).size(13).color(Color::WHITE).width(200),
            container(progress_bar(0.0..=100.0, pct).girth(10)).width(Length::Fill),
            text(game.playtime_display())
                .size(12)
                .color(theme::ACCENT_BLUE)
                .width(80),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);
        rows = rows.push(game_row);
    }

    section_card("Most Played", rows, vt)
}

fn period_comparison<'a>(app: &'a Spotter, vt: ViewTheme) -> Element<'a, Message> {
    let now = chrono::Local::now();
    let this_month = now.format("%Y-%m").to_string();
    let last_month = (now - chrono::Duration::days(30))
        .format("%Y-%m")
        .to_string();

    // This month's playtime from history
    let this_month_playtime: u32 = app
        .playtime_data
        .iter()
        .filter(|(date, _)| date.starts_with(&this_month))
        .map(|(_, m)| *m)
        .sum();

    let last_month_playtime: u32 = app
        .playtime_data
        .iter()
        .filter(|(date, _)| date.starts_with(&last_month))
        .map(|(_, m)| *m)
        .sum();

    // Count games completed this month vs last
    let this_month_prefix = this_month.as_str();
    let last_month_prefix = last_month.as_str();

    let completed_this = app
        .games
        .iter()
        .filter(|g| {
            g.status == GameStatus::Completed && g.last_played.starts_with(this_month_prefix)
        })
        .count();
    let completed_last = app
        .games
        .iter()
        .filter(|g| {
            g.status == GameStatus::Completed && g.last_played.starts_with(last_month_prefix)
        })
        .count();

    let playtime_delta = this_month_playtime as i64 - last_month_playtime as i64;
    let playtime_arrow = if playtime_delta > 0 {
        format!("+{}", format_playtime(playtime_delta as u32))
    } else if playtime_delta < 0 {
        format!("-{}", format_playtime((-playtime_delta) as u32))
    } else {
        "same".to_string()
    };

    let playtime_color = if playtime_delta > 0 {
        theme::SUCCESS
    } else if playtime_delta < 0 {
        Color::from_rgb(1.0, 0.4, 0.4)
    } else {
        vt.text_muted
    };

    let completed_delta = completed_this as i64 - completed_last as i64;
    let completed_arrow = if completed_delta > 0 {
        format!("+{}", completed_delta)
    } else if completed_delta < 0 {
        format!("{}", completed_delta)
    } else {
        "same".to_string()
    };

    let completed_color = if completed_delta > 0 {
        theme::SUCCESS
    } else if completed_delta < 0 {
        Color::from_rgb(1.0, 0.4, 0.4)
    } else {
        vt.text_muted
    };

    let content = column![row![
        column![
            text("This Month").size(11).color(vt.text_muted),
            text(format_playtime(this_month_playtime))
                .size(20)
                .color(theme::ACCENT_BLUE),
            text(format!("{} completed", completed_this))
                .size(12)
                .color(theme::COMPLETED_BLUE),
        ]
        .spacing(4)
        .width(Length::Fill),
        column![
            text("vs Last Month").size(11).color(vt.text_muted),
            text(format_playtime(last_month_playtime))
                .size(20)
                .color(vt.text_secondary),
            text(format!("{} completed", completed_last))
                .size(12)
                .color(vt.text_secondary),
        ]
        .spacing(4)
        .width(Length::Fill),
        column![
            text("Change").size(11).color(vt.text_muted),
            text(playtime_arrow).size(20).color(playtime_color),
            text(format!("{} completed", completed_arrow))
                .size(12)
                .color(completed_color),
        ]
        .spacing(4)
        .width(Length::Fill),
    ]
    .spacing(16),]
    .spacing(8);

    section_card("Month Comparison", content, vt)
}

fn format_playtime(minutes: u32) -> String {
    crate::models::format_playtime(minutes)
}
