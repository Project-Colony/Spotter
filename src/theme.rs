//! Spotter's widget styles. Every colour comes from the active Colony palette
//! (`colony_ui::theme::Palette`) or the shared accent swatches: Spotter defines
//! none of its own, so a theme, accent or high-contrast change reaches every
//! widget.

use colony_ui::theme::Palette;
use colony_ui::{accent_key_to_color, contrast_on};
use iced::widget::{button, container};
use iced::{Background, Border, Color, Theme};

/// Nerd Font icon constants (Font Awesome + Material Design Icons).
/// Some icons are kept as a reference catalog even if not currently used.
#[allow(dead_code)]
pub mod icons {
    // ── Navigation ──
    pub const LIBRARY: &str = "\u{f0322}"; // nf-md-bookshelf
    pub const ADD: &str = "\u{f067}"; // nf-fa-plus
    pub const STATS: &str = "\u{f080}"; // nf-fa-bar-chart
    pub const IMPORT: &str = "\u{f019}"; // nf-fa-download
    pub const PROFILE: &str = "\u{f007}"; // nf-fa-user
    pub const SETTINGS: &str = "\u{f013}"; // nf-fa-cog
    pub const BACK: &str = "\u{f060}"; // nf-fa-arrow-left
    pub const SEARCH: &str = "\u{f002}"; // nf-fa-search

    // ── Actions ──
    pub const DELETE: &str = "\u{f014}"; // nf-fa-trash
    pub const EXPORT: &str = "\u{f093}"; // nf-fa-upload
    pub const SORT: &str = "\u{f0dc}"; // nf-fa-sort
    pub const FILTER: &str = "\u{f0b0}"; // nf-fa-filter
    pub const REFRESH: &str = "\u{f021}"; // nf-fa-refresh

    // ── Favorites ──
    pub const STAR: &str = "\u{f005}"; // nf-fa-star (filled)
    pub const STAR_O: &str = "\u{f006}"; // nf-fa-star-o (outline)
    pub const HEART: &str = "\u{f004}"; // nf-fa-heart
    pub const HEART_O: &str = "\u{f08a}"; // nf-fa-heart-o
    pub const TROPHY: &str = "\u{f091}"; // nf-fa-trophy
    pub const FLAG: &str = "\u{f024}"; // nf-fa-flag
    pub const BOOKMARK: &str = "\u{f02e}"; // nf-fa-bookmark
    pub const TAG: &str = "\u{f02b}"; // nf-fa-tag

    // ── Gaming ──
    pub const GAMEPAD: &str = "\u{f11b}"; // nf-fa-gamepad
    pub const STEAM: &str = "\u{f1b6}"; // nf-fa-steam
    pub const CHECK: &str = "\u{f00c}"; // nf-fa-check
    pub const TIMES: &str = "\u{f00d}"; // nf-fa-times
    pub const PLAY: &str = "\u{f04b}"; // nf-fa-play
    pub const CIRCLE_O: &str = "\u{f10c}"; // nf-fa-circle-o
}

/// A shared Colony accent swatch by key (`colony_ui::ACCENT_OVERRIDES`): the
/// same colour on every theme, which is what a platform or a game status needs
/// to stay recognisable. Falls back to the theme's accent for an unknown key.
pub fn shared_accent(key: &str) -> Color {
    accent_key_to_color(key).unwrap_or_else(Palette::ACCENT)
}

fn is_hovered(status: button::Status) -> bool {
    matches!(status, button::Status::Hovered | button::Status::Pressed)
}

/// The outline of an inactive control. Drawn from the ink rather than from a
/// surface, so it shows on the page and on a card alike (several themes give
/// cards and borders the same colour).
fn control_outline() -> Color {
    Palette::TEXT_DIMMEST().scale_alpha(0.4)
}

// ── Reusable style helpers (with hover support) ──

/// Container style for cards with rounded corners and a border.
pub fn card_style(
    bg: Color,
    border_color: Color,
    radius: f32,
) -> impl Fn(&Theme) -> container::Style {
    move |_: &Theme| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            radius: radius.into(),
            color: border_color,
            width: 1.0,
        },
        ..container::Style::default()
    }
}

/// Chip-like toggle (filters, options, status and rating pickers): tinted with
/// `color` while active, outlined otherwise.
pub fn chip_style(
    active: bool,
    color: Color,
    radius: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let hover = is_hovered(status);
        let (background, border, text_color) = if active {
            (
                color.scale_alpha(if hover { 0.24 } else { 0.16 }),
                color.scale_alpha(0.5),
                color,
            )
        } else {
            (
                if hover {
                    Palette::BG_CARD_HOVER()
                } else {
                    Color::TRANSPARENT
                },
                control_outline(),
                Palette::TEXT_MUTED(),
            )
        };
        button::Style {
            background: Some(Background::Color(background)),
            text_color,
            border: Border {
                radius: radius.into(),
                color: border,
                width: 1.0,
            },
            ..button::Style::default()
        }
    }
}

/// Invisible button wrapper with subtle hover feedback.
pub fn transparent_btn_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_: &Theme, status: button::Status| button::Style {
        background: is_hovered(status).then(|| Background::Color(Palette::BG_CARD_HOVER())),
        text_color: Palette::TEXT_PRIMARY(),
        border: Border {
            radius: 8.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

/// Solid action button filled with `bg`, its label in whichever of near-black
/// or near-white reads on it. Hovering lets a little of the surface through.
pub fn primary_btn_style(bg: Color) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| button::Style {
        background: Some(Background::Color(if is_hovered(status) {
            bg.scale_alpha(0.85)
        } else {
            bg
        })),
        text_color: contrast_on(bg),
        border: Border {
            radius: 8.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

/// Button outlined and labelled in `color`, tinted on hover. Transparent, so it
/// sits on any surface.
pub fn outline_btn_style(color: Color) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let hover = is_hovered(status);
        button::Style {
            background: hover.then(|| Background::Color(color.scale_alpha(0.12))),
            text_color: color,
            border: Border {
                radius: 8.0.into(),
                color: color.scale_alpha(if hover { 0.5 } else { 0.35 }),
                width: 1.0,
            },
            ..button::Style::default()
        }
    }
}

/// Disabled / muted button (no hover effect).
pub fn disabled_btn_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_: &Theme, _: button::Status| button::Style {
        background: None,
        text_color: Palette::TEXT_DIMMER(),
        border: Border {
            radius: 8.0.into(),
            color: control_outline(),
            width: 1.0,
        },
        ..button::Style::default()
    }
}

/// Work in progress, such as an import or a login waiting on the browser
/// (warning tint, no hover).
pub fn loading_btn_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_: &Theme, _: button::Status| button::Style {
        background: Some(Background::Color(Palette::WARNING_BG())),
        text_color: Palette::WARNING(),
        border: Border {
            radius: 8.0.into(),
            color: Palette::WARNING().scale_alpha(0.4),
            width: 1.0,
        },
        ..button::Style::default()
    }
}

/// Card with a border tinted by `accent` (platform branding).
pub fn accent_card_style(accent: Color) -> impl Fn(&Theme) -> container::Style {
    move |_: &Theme| container::Style {
        background: Some(Background::Color(Palette::BG_CARD())),
        border: Border {
            radius: 10.0.into(),
            color: accent.scale_alpha(0.25),
            width: 1.0,
        },
        ..container::Style::default()
    }
}

/// Small colored dot used as a platform icon indicator.
pub fn dot_style(color: Color, size: f32) -> impl Fn(&Theme) -> container::Style {
    move |_: &Theme| container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            radius: (size / 2.0).into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// Danger button for delete actions, with hover effect.
pub fn danger_btn_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_: &Theme, status: button::Status| button::Style {
        background: Some(Background::Color(if is_hovered(status) {
            Palette::BTN_DANGER_HOVER()
        } else {
            Palette::BTN_DANGER_BG()
        })),
        text_color: Palette::ERROR(),
        border: Border {
            radius: 6.0.into(),
            color: Palette::ERROR().scale_alpha(0.5),
            width: 1.0,
        },
        ..button::Style::default()
    }
}

/// Status pill (small rounded container for badges).
pub fn pill_style(bg: Color, border_color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_: &Theme| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            radius: 12.0.into(),
            color: border_color,
            width: 1.0,
        },
        ..container::Style::default()
    }
}
