# Configuration

All settings are stored as a JSON blob in the SQLite `settings` table and applied immediately on change.

## Settings Overview

### General

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `default_status` | GameStatus | Unplayed | Default status for new games |
| `default_platform` | Platform | Steam | Default platform for new games |
| `confirm_before_delete` | bool | true | Show confirmation dialog before deleting a game |
| `start_screen` | StartScreen | Library | Screen shown on launch (Library / Statistics / Import / Profile) |
| `notifications_enabled` | bool | true | Show toast notifications for success messages |
| `toast_duration` | ToastDuration | Normal (4s) | Auto-dismiss duration: Short (2s), Normal (4s), Long (8s) |
| `date_format` | DateFormat | YYYY-MM-DD | Date display: YYYY-MM-DD, DD/MM/YYYY, MM/DD/YYYY |
| `default_sort_order` | SortOrder | Title A-Z | Default library sort |
| `remember_filters` | bool | false | Persist filter selections across sessions |
| `show_game_descriptions` | bool | false | Show store descriptions in library cards |
| `achievements_display` | AchievementsDisplay | Full | Full list / Summary only / Hidden |
| `download_covers_auto` | bool | true | Automatically download cover art on import |

### Appearances

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `theme_mode` | ThemeMode | Dark | Dark / Darker / Midnight |
| `accent_color` | AccentColor | Blue | Blue / Purple / Green / Gold / Red / Cyan |
| `compact_list` | bool | false | Reduced spacing in game list (4px vs 8px) |
| `show_covers_in_list` | bool | true | Show cover thumbnails in library |
| `ui_scale` | UiScale | Normal | Small (0.85x) / Normal (1.0x) / Large (1.2x) |
| `sidebar_width` | u16 | 220 | Sidebar width in pixels (160-400, presets: 180/220/280) |

### Accessibility

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `high_contrast` | bool | false | Increase text/border contrast |
| `show_status_labels` | bool | true | Show text labels alongside status dots |
| `large_click_targets` | bool | false | Increase button padding (8px → 12px) |

## Theme Modes

Three dark theme variants with progressively darker backgrounds:

| Element | Dark | Darker | Midnight |
|---------|------|--------|----------|
| Main bg | `rgb(0.10, 0.10, 0.14)` | `rgb(0.06, 0.06, 0.09)` | `rgb(0.02, 0.02, 0.05)` |
| Sidebar bg | `rgb(0.08, 0.08, 0.12)` | `rgb(0.04, 0.04, 0.07)` | `rgb(0.01, 0.01, 0.03)` |
| Card bg | `rgb(0.12, 0.12, 0.16)` | `rgb(0.08, 0.08, 0.11)` | `rgb(0.04, 0.04, 0.07)` |
| Row bg | `rgb(0.09, 0.09, 0.13)` | `rgb(0.06, 0.06, 0.09)` | `rgb(0.03, 0.03, 0.05)` |

## Accent Colors

| Name | RGB | Usage |
|------|-----|-------|
| Blue | `rgb(0.6, 0.8, 1.0)` | Default. Sidebar title, active tabs, filter chips |
| Purple | `rgb(0.7, 0.5, 0.9)` | Alternative accent |
| Green | `rgb(0.4, 0.9, 0.5)` | Nature/organic theme |
| Gold | `rgb(1.0, 0.85, 0.3)` | Warm/premium theme |
| Red | `rgb(1.0, 0.45, 0.45)` | Bold/aggressive theme |
| Cyan | `rgb(0.3, 0.9, 0.9)` | Cool/tech theme |

## High Contrast Mode

When enabled, text and border colors shift toward higher luminance:

| Element | Normal | High Contrast |
|---------|--------|---------------|
| Border | `rgb(0.18, 0.18, 0.24)` | `rgb(0.35, 0.35, 0.45)` |
| Text Muted | `rgb(0.5, 0.5, 0.6)` | `rgb(0.6, 0.6, 0.7)` |
| Text Dim | `rgb(0.4, 0.4, 0.5)` | `rgb(0.5, 0.5, 0.6)` |
| Text Secondary | `rgb(0.6, 0.6, 0.7)` | `rgb(0.7, 0.7, 0.8)` |
| Text Light | `rgb(0.7, 0.7, 0.8)` | `rgb(1.0, 1.0, 1.0)` |

## Style Helpers

Reusable style functions in `theme.rs`:

| Function | Description |
|----------|-------------|
| `card_style(bg, border, radius)` | Rounded card container |
| `chip_style(active, text, accent, radius)` | Filter chip toggle |
| `transparent_btn_style()` | Invisible button wrapper |
| `primary_btn_style(bg)` | Solid filled action button |
| `outline_btn_style(color, bg)` | Outlined button |
| `disabled_btn_style()` | Grayed-out disabled button |
| `loading_btn_style()` | Gold-tinted importing state |
| `accent_card_style(bg, accent)` | Card with colored left border |
| `dot_style(color, size)` | Small circular indicator |
| `pill_style(bg, border)` | Rounded status badge |

## Settings Schema Versioning

Settings include a `version` field (currently `1`) with `#[serde(default)]` on newer fields. This allows older settings JSON to deserialize without breaking, with new fields taking their defaults.
