use iced::widget::{container, text};
use iced::{Background, Color, Element, Length, Theme};

use crate::app::Message;
use crate::theme;

/// Small colored dot used as a platform indicator.
pub fn platform_dot(color: Color) -> Element<'static, Message> {
    container(text("").size(1))
        .width(12)
        .height(12)
        .style(theme::dot_style(color, 12.0))
        .into()
}

/// Status badge pill with colored background/border.
pub fn status_badge<'a>(label: &'a str, color: Color) -> Element<'a, Message> {
    let bg = Color { a: 0.15, ..color };
    let border = Color { a: 0.3, ..color };
    container(text(label).size(11).color(color))
        .padding([3, 10])
        .style(theme::pill_style(bg, border))
        .into()
}

/// Thin colored separator line.
pub fn separator(color: Color) -> Element<'static, Message> {
    container(text("").size(1))
        .width(Length::Fill)
        .height(1)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(Color { a: 0.12, ..color })),
            ..container::Style::default()
        })
        .into()
}
