# Configuration

All preferences are saved to `preferences.json` in Spotter's config folder (`~/.config/Colony/Spotter/preferences/preferences.json` on Linux; see [Data Storage](../README.md#data-storage) for the other platforms) and applied immediately on change.

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
| `remember_filters` | bool | true | Persist filter selections across sessions |
| `show_game_descriptions` | bool | false | Show store descriptions in library cards |
| `achievements_display` | AchievementsDisplay | Full | Full list / Summary only / Hidden |
| `download_covers_auto` | bool | true | Automatically download cover art on import |

### Appearances

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `compact_list` | bool | false | Reduced spacing in game list (4px vs 8px) |
| `show_covers_in_list` | bool | true | Show cover thumbnails in library |
| `sidebar_width` | u16 | 220 | Sidebar width in pixels (160-400, presets: 180/220/280) |

### Accessibility

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `show_status_labels` | bool | true | Show text labels alongside status dots |
| `large_click_targets` | bool | false | Increase button padding (8px → 12px) |

### Shared Colony preferences

Theme, accent, text sizes and the accessibility switches are the ones every Colony program has (`colony_ui::preferences::StandardPreferences`), saved in the same file under the same names:

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `selected_theme`, `selected_variant` | string | `gruvbox`, `dark` | A shared Colony theme; an unknown one falls back to Gruvbox dark |
| `selected_accent` | string | none | Accent override (`red`, `orange`, `yellow`, `green`, `blue`, `indigo`, `violet`, `amber`); none uses the theme's own |
| `auto_accent` | bool | false | Use the theme's accent even when one is picked |
| `font_size` | small / default / large | default | Appearance text size |
| `animations` | bool | true | Animations and fades |
| `high_contrast` | bool | false | Push text and borders away from the background |
| `dyslexia_font` | bool | false | Use OpenDyslexic |
| `reduce_motion` | bool | false | Turn every animation off |
| `text_size_a11y` | small / default / large / xlarge | default | Accessibility text size, multiplied with `font_size` |

## Upgrading from Spotter 0.3

Spotter 0.3 and earlier kept the settings in the library database, with three dark themes, six accents and an interface scale of its own. The first start of a newer Spotter imports them into `preferences.json`:

| Old setting | Becomes |
|-------------|---------|
| `theme_mode` Dark / Darker / Midnight | Tokyo Night night / Material deep ocean / Modus vivendi |
| `accent_color` Blue, Cyan / Purple / Green / Gold / Red | `blue` / `violet` / `green` / `amber` / `red` |
| `ui_scale` Small / Normal / Large | `text_size_a11y` small / default / large |

The old scale enlarged the whole window; the text sizes now enlarge the text only, and the Appearance and Accessibility sizes multiply (0.7225x to 1.68x).

## Style Helpers

Spotter defines no colour of its own. Widgets read the active Colony palette (`colony_ui::theme::Palette`, for example `Palette::TEXT_MUTED()`), so a theme, accent or high-contrast change reaches every screen. Platforms and game statuses use the shared accent swatches (`theme::shared_accent`), the same on every theme; text on a filled button takes `colony_ui::contrast_on` of the fill.

Reusable style functions in `theme.rs`:

| Function | Description |
|----------|-------------|
| `card_style(bg, border, radius)` | Rounded card container |
| `chip_style(active, color, radius)` | Chip toggle: tinted with `color` when active, outlined otherwise |
| `transparent_btn_style()` | Invisible button wrapper |
| `primary_btn_style(bg)` | Solid filled action button, label in `contrast_on(bg)` |
| `outline_btn_style(color)` | Outlined button on any surface |
| `disabled_btn_style()` | Dimmed disabled button |
| `loading_btn_style()` | Warning-tinted state for imports and logins in progress |
| `danger_btn_style()` | Delete action |
| `accent_card_style(accent)` | Card with a border tinted by `accent` |
| `dot_style(color, size)` | Small circular indicator |
| `pill_style(bg, border)` | Rounded status badge |

## Settings Schema Versioning

Preferences include a `version` field (currently `2`: the move to `preferences.json`), and `Settings` is `#[serde(default)]`, so a missing field takes its default.

`preferences::parse` also reads the file one field at a time when the whole struct does not parse: a value this build does not accept (for example one added by a newer version) falls back to that field's default, and every other preference keeps its stored value.

If the startup load fails anyway, for example because `preferences.json` is not JSON at all, Spotter shows an error and runs on defaults, and refuses every write of saved data for the rest of the session: preferences, profile and game saves, deletions, exports and the auto-backup are all refused (`Spotter::spawn_write`), so the data on disk stays as it was.
