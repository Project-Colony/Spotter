use iced::widget::{button, container};
use iced::{Background, Border, Color, Font, Theme};

use crate::models::{DateFormat, Settings};

// ── Font ──

/// Bold weight of the JetBrainsMono Nerd Font.
pub const FONT_BOLD: Font = Font {
    family: iced::font::Family::Name("JetBrainsMono NF"),
    weight: iced::font::Weight::Bold,
    ..Font::DEFAULT
};

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

// Static color constants are used in style closures (button/container styles)
// where ViewTheme isn't available. ViewTheme provides theme-mode-aware colors
// for view layout code. These are complementary, not redundant.

// ── Backgrounds ──
pub const BG_DARK: Color = Color {
    r: 0.09,
    g: 0.09,
    b: 0.13,
    a: 1.0,
};
pub const BG_DARKER: Color = Color {
    r: 0.04,
    g: 0.04,
    b: 0.08,
    a: 1.0,
};
pub const BG_BUTTON: Color = Color {
    r: 0.11,
    g: 0.11,
    b: 0.16,
    a: 1.0,
};

// ── Text (contrast-improved for WCAG readability) ──
pub const TEXT_MUTED: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.6,
    a: 1.0,
};
pub const TEXT_DIM: Color = Color {
    r: 0.42,
    g: 0.42,
    b: 0.52,
    a: 1.0,
};
pub const TEXT_SECONDARY: Color = Color {
    r: 0.62,
    g: 0.62,
    b: 0.72,
    a: 1.0,
};
pub const TEXT_LIGHT: Color = Color {
    r: 0.75,
    g: 0.75,
    b: 0.85,
    a: 1.0,
};
pub const TEXT_HEADING: Color = Color {
    r: 0.3,
    g: 0.3,
    b: 0.4,
    a: 1.0,
};

// ── Accents ──
pub const ACCENT_BLUE: Color = Color {
    r: 0.45,
    g: 0.72,
    b: 1.0,
    a: 1.0,
};
pub const ACCENT_GOLD: Color = Color {
    r: 1.0,
    g: 0.82,
    b: 0.28,
    a: 1.0,
};
pub const SUCCESS: Color = Color {
    r: 0.25,
    g: 0.85,
    b: 0.45,
    a: 1.0,
};
pub const COMPLETED_BLUE: Color = Color {
    r: 0.35,
    g: 0.62,
    b: 1.0,
    a: 1.0,
};
pub const DANGER: Color = Color {
    r: 1.0,
    g: 0.38,
    b: 0.38,
    a: 1.0,
};

// ── Borders ──
pub const BORDER: Color = Color {
    r: 0.16,
    g: 0.16,
    b: 0.22,
    a: 1.0,
};
pub const BORDER_CARD: Color = Color {
    r: 0.18,
    g: 0.18,
    b: 0.24,
    a: 1.0,
};

/// Lighten a color by a fixed amount (clamped to 1.0).
pub fn lighten(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r + amount).min(1.0),
        g: (color.g + amount).min(1.0),
        b: (color.b + amount).min(1.0),
        a: color.a,
    }
}

/// Pre-computed theme values derived from user Settings.
/// Create with `ViewTheme::from_settings(&app.settings)` at the top of each view.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct ViewTheme {
    pub bg_main: Color,
    pub bg_card: Color,
    pub bg_row: Color,
    pub bg_sidebar: Color,
    pub border: Color,
    pub text_muted: Color,
    pub text_dim: Color,
    pub text_secondary: Color,
    pub text_light: Color,
    pub btn_padding: u16,
    pub show_status_labels: bool,
    pub date_format: DateFormat,
}

impl ViewTheme {
    pub fn from_settings(s: &Settings) -> Self {
        let hc = s.high_contrast;
        let lt = s.large_click_targets;
        Self {
            bg_main: s.theme_mode.bg_main(),
            bg_card: s.theme_mode.bg_card(),
            bg_row: s.theme_mode.bg_row(),
            bg_sidebar: s.theme_mode.bg_sidebar(),
            border: if hc {
                Color::from_rgb(0.35, 0.35, 0.45)
            } else {
                BORDER
            },
            text_muted: if hc { TEXT_SECONDARY } else { TEXT_MUTED },
            text_dim: if hc { TEXT_MUTED } else { TEXT_DIM },
            text_secondary: if hc { TEXT_LIGHT } else { TEXT_SECONDARY },
            text_light: if hc { Color::WHITE } else { TEXT_LIGHT },
            btn_padding: if lt { 12 } else { 8 },
            show_status_labels: s.show_status_labels,
            date_format: s.date_format,
        }
    }
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

/// Button style for chip-like toggles (filter chips, option chips).
pub fn chip_style(
    active: bool,
    text_color: Color,
    accent: Color,
    radius: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let bg = if active {
            if hover {
                lighten(Color::from_rgb(0.14, 0.14, 0.20), 0.04)
            } else {
                Color::from_rgb(0.14, 0.14, 0.20)
            }
        } else if hover {
            lighten(BG_BUTTON, 0.04)
        } else {
            BG_BUTTON
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                radius: radius.into(),
                color: if active {
                    Color { a: 0.4, ..accent }
                } else if hover {
                    Color::from_rgb(0.2, 0.2, 0.28)
                } else {
                    Color::TRANSPARENT
                },
                width: 1.0,
            },
            ..button::Style::default()
        }
    }
}

/// Invisible button wrapper with subtle hover feedback.
pub fn transparent_btn_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_: &Theme, status: button::Status| {
        let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: if hover {
                Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03)))
            } else {
                None
            },
            border: Border {
                radius: 8.0.into(),
                ..Border::default()
            },
            ..button::Style::default()
        }
    }
}

/// Solid primary action button with hover lightening.
pub fn primary_btn_style(bg: Color) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(Background::Color(if hover {
                lighten(bg, 0.08)
            } else {
                bg
            })),
            text_color: BG_DARKER,
            border: Border {
                radius: 8.0.into(),
                ..Border::default()
            },
            ..button::Style::default()
        }
    }
}

/// Subtle outlined button with hover fill.
pub fn outline_btn_style(
    color: Color,
    bg: Color,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(Background::Color(if hover {
                lighten(bg, 0.04)
            } else {
                bg
            })),
            text_color: color,
            border: Border {
                radius: 8.0.into(),
                color: if hover {
                    Color { a: 0.5, ..color }
                } else {
                    Color { a: 0.35, ..color }
                },
                width: 1.0,
            },
            ..button::Style::default()
        }
    }
}

/// Disabled / muted button (no hover effect).
pub fn disabled_btn_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_: &Theme, _: button::Status| button::Style {
        background: Some(Background::Color(BG_BUTTON)),
        text_color: TEXT_HEADING,
        border: Border {
            radius: 8.0.into(),
            color: BORDER_CARD,
            width: 1.0,
        },
        ..button::Style::default()
    }
}

/// Pulsing/active state button (gold tint for "importing" states, no hover).
pub fn loading_btn_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_: &Theme, _: button::Status| button::Style {
        background: Some(Background::Color(Color::from_rgb(0.14, 0.13, 0.08))),
        text_color: ACCENT_GOLD,
        border: Border {
            radius: 8.0.into(),
            color: Color::from_rgb(0.3, 0.28, 0.1),
            width: 1.0,
        },
        ..button::Style::default()
    }
}

/// Card with an accent-colored left border (platform branding).
pub fn accent_card_style(bg: Color, accent: Color) -> impl Fn(&Theme) -> container::Style {
    move |_: &Theme| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            radius: 10.0.into(),
            color: Color { a: 0.25, ..accent },
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

/// Badge container style (platform/status pills in detail view).
pub fn badge_style(border_color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_: &Theme| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.18))),
        border: Border {
            radius: 12.0.into(),
            color: border_color,
            width: 1.0,
        },
        ..container::Style::default()
    }
}

/// Danger button (red) with hover effect for delete actions.
pub fn danger_btn_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_: &Theme, status: button::Status| {
        let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let bg = if hover {
            Color::from_rgb(0.25, 0.08, 0.08)
        } else {
            Color::from_rgb(0.2, 0.05, 0.05)
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: Color::from_rgb(1.0, 0.4, 0.4),
            border: Border {
                radius: 6.0.into(),
                color: Color::from_rgb(0.5, 0.15, 0.15),
                width: 1.0,
            },
            ..button::Style::default()
        }
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
