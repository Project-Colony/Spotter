use colony_ui::theme::Palette;
use iced::widget::{column, container, progress_bar, row, rule, space};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::{Message, Spotter};
use crate::models::{GameStatus, Platform};
use crate::theme::{self, ui_text};

pub fn view(app: &Spotter) -> Element<'_, Message> {
    let header = column![
        ui_text(format!("{} Statistics", theme::icons::STATS), 24)
            .font(colony_ui::ui_font_bold())
            .color(Palette::TEXT_PRIMARY()),
        ui_text("Overview of your gaming journey", 12).color(Palette::TEXT_MUTED()),
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
            Palette::ACCENT()
        ),
        stat_card(
            "Total Playtime",
            format_playtime(total_playtime),
            Palette::SUCCESS()
        ),
        stat_card(
            "Achievements",
            format!("{}/{}", total_achievements, total_possible),
            Palette::WARNING()
        ),
        stat_card(
            "Avg Rating",
            format!("{:.1}/10", avg_rating),
            theme::shared_accent("violet")
        ),
    ]
    .spacing(12);

    // Status distribution
    let status_dist = status_distribution(app);

    // Platform distribution
    let platform_dist = platform_distribution(app);

    // Playtime graph
    let playtime_graph = playtime_chart(app);

    // Most played games
    let most_played = most_played_games(app);

    // Period comparison
    let comparison = period_comparison(app);

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

fn stat_card<'a>(label: &'a str, value: String, color: Color) -> Element<'a, Message> {
    let content = column![
        ui_text(label, 12).color(Palette::TEXT_MUTED()),
        ui_text(value, 24).color(color),
    ]
    .spacing(6);

    container(content)
        .padding(16)
        .width(Length::Fill)
        .style(theme::card_style(
            Palette::BG_CARD(),
            Palette::BORDER_SUBTLE(),
            8.0,
        ))
        .into()
}

fn section_card<'a>(
    title: &'a str,
    content: iced::widget::Column<'a, Message>,
) -> Element<'a, Message> {
    let section = column![
        ui_text(title, 16).color(Palette::TEXT_PRIMARY()),
        rule::horizontal(1),
        content,
    ]
    .spacing(10);

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

fn playtime_chart<'a>(app: &'a Spotter) -> Element<'a, Message> {
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
                ui_text("No playtime data yet. Import games to start tracking!", 13)
                    .color(Palette::TEXT_MUTED()),
            ],
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
                background: Some(Background::Color(Palette::ACCENT_PROGRESS())),
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
            ui_text(label, 8).color(Palette::TEXT_DIM()),
        ]
        .align_x(iced::Alignment::Center)
        .width(Length::Fill);

        bars_row = bars_row.push(bar_with_label);
    }

    let total_tracked: u32 = data.iter().map(|(_, m)| *m).sum();
    let subtitle = ui_text(
        format!(
            "Total: {} tracked over {} days",
            format_playtime(total_tracked),
            data.len()
        ),
        11,
    )
    .color(Palette::TEXT_MUTED());

    let mut chart_content = column![
        container(bars_row)
            .height(Length::Fixed(max_height + 20.0))
            .width(Length::Fill),
        subtitle,
    ]
    .spacing(8);

    if is_sample_data {
        chart_content = chart_content.push(
            ui_text(
                "Estimated from currently playing games (no recorded playtime data yet)",
                10,
            )
            .color(Palette::TEXT_DIM()),
        );
    }

    section_card("Playtime (Last 30 Days)", chart_content)
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

fn status_distribution<'a>(app: &'a Spotter) -> Element<'a, Message> {
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
            ui_text(format!("{}", status), 12)
                .color(status.color())
                .width(90),
            container(progress_bar(0.0..=100.0, pct).girth(10)).width(Length::Fill),
            ui_text(format!("{} ({:.0}%)", count, pct), 12)
                .color(Palette::TEXT_SECONDARY())
                .width(80),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);
        bars = bars.push(bar_row);
    }

    section_card("Status Distribution", bars)
}

fn platform_distribution<'a>(app: &'a Spotter) -> Element<'a, Message> {
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
            ui_text(format!("{}", platform), 12)
                .color(platform.color())
                .width(90),
            container(progress_bar(0.0..=100.0, pct).girth(10)).width(Length::Fill),
            ui_text(format!("{} ({:.0}%)", count, pct), 12)
                .color(Palette::TEXT_SECONDARY())
                .width(80),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);
        bars = bars.push(bar_row);
    }

    section_card("Platform Distribution", bars)
}

fn most_played_games<'a>(app: &'a Spotter) -> Element<'a, Message> {
    let mut sorted: Vec<&crate::models::Game> = app.games.iter().collect();
    sorted.sort_by_key(|b| std::cmp::Reverse(b.playtime_minutes));

    let max_time = sorted.first().map_or(1, |g| g.playtime_minutes) as f32;

    let mut rows = column![].spacing(6);
    for game in sorted.iter().take(5) {
        let pct = (game.playtime_minutes as f32 / max_time) * 100.0;
        let game_row = row![
            ui_text(&game.title, 13)
                .color(Palette::TEXT_PRIMARY())
                .width(200),
            container(progress_bar(0.0..=100.0, pct).girth(10)).width(Length::Fill),
            ui_text(game.playtime_display(), 12)
                .color(Palette::ACCENT())
                .width(80),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);
        rows = rows.push(game_row);
    }

    section_card("Most Played", rows)
}

fn period_comparison<'a>(app: &'a Spotter) -> Element<'a, Message> {
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
        Palette::SUCCESS()
    } else if playtime_delta < 0 {
        Palette::ERROR()
    } else {
        Palette::TEXT_MUTED()
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
        Palette::SUCCESS()
    } else if completed_delta < 0 {
        Palette::ERROR()
    } else {
        Palette::TEXT_MUTED()
    };

    let content = column![row![
        column![
            ui_text("This Month", 11).color(Palette::TEXT_MUTED()),
            ui_text(format_playtime(this_month_playtime), 20).color(Palette::ACCENT()),
            ui_text(format!("{} completed", completed_this), 12)
                .color(GameStatus::Completed.color()),
        ]
        .spacing(4)
        .width(Length::Fill),
        column![
            ui_text("vs Last Month", 11).color(Palette::TEXT_MUTED()),
            ui_text(format_playtime(last_month_playtime), 20).color(Palette::TEXT_SECONDARY()),
            ui_text(format!("{} completed", completed_last), 12).color(Palette::TEXT_SECONDARY()),
        ]
        .spacing(4)
        .width(Length::Fill),
        column![
            ui_text("Change", 11).color(Palette::TEXT_MUTED()),
            ui_text(playtime_arrow, 20).color(playtime_color),
            ui_text(format!("{} completed", completed_arrow), 12).color(completed_color),
        ]
        .spacing(4)
        .width(Length::Fill),
    ]
    .spacing(16),]
    .spacing(8);

    section_card("Month Comparison", content)
}

fn format_playtime(minutes: u32) -> String {
    crate::models::format_playtime(minutes)
}
