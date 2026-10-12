use colony_ui::theme::Palette;
use iced::widget::{button, column, container, row, stack, text};
use iced::{Background, Border, Color, Element, Length, Theme};

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
    let bg = color.scale_alpha(0.15);
    let border = color.scale_alpha(0.3);
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
            background: Some(Background::Color(color.scale_alpha(0.12))),
            ..container::Style::default()
        })
        .into()
}

/// Delete confirmation over a dimmed page. Clicking outside the card cancels.
pub fn delete_modal<'a>(
    title: &str,
    question: String,
    confirm_label: String,
    on_confirm: Message,
) -> Element<'a, Message> {
    let backdrop = button(
        container(text("").size(1))
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(Message::CancelDelete)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_: &Theme, _| button::Style {
        background: Some(Background::Color(Palette::BG_PRIMARY().scale_alpha(0.75))),
        ..button::Style::default()
    });

    let card = container(
        column![
            text(format!("{} {}", theme::icons::DELETE, title))
                .size(20)
                .font(colony_ui::ui_font_bold())
                .color(Palette::ERROR()),
            text(question).size(14).color(Palette::TEXT_PRIMARY()),
            text("You'll have 8 seconds to undo after deletion.")
                .size(12)
                .color(Palette::TEXT_MUTED()),
            row![
                button(text(confirm_label).size(14))
                    .on_press(on_confirm)
                    .padding([10, 24])
                    .style(theme::primary_btn_style(Palette::BTN_TRASH_HOVER())),
                button(text("Cancel").size(14))
                    .on_press(Message::CancelDelete)
                    .padding([10, 24])
                    .style(theme::outline_btn_style(Palette::TEXT_SECONDARY())),
            ]
            .spacing(12),
        ]
        .spacing(12),
    )
    .padding(24)
    .max_width(450)
    .style(|_: &Theme| container::Style {
        background: Some(Background::Color(Palette::BG_CARD())),
        border: Border {
            radius: 12.0.into(),
            color: Palette::ERROR().scale_alpha(0.6),
            width: 2.0,
        },
        ..container::Style::default()
    });

    stack![
        backdrop,
        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    ]
    .into()
}
