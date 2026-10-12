# Configuration

All preferences are saved to `preferences.json` in Spotter's config folder (`~/.config/Colony/Spotter/preferences/preferences.json` on Linux; see [Data Storage](../README.md#data-storage) for the other platforms) and applied immediately on change. There is no Save button.

## The Preferences page

Clicking **Spotter** at the top of the sidebar opens Preferences in place of the current screen; clicking it again, the page's Close button or Esc goes back to that screen. The page is the shared Colony one (`colony_ui::widgets`): General, Appearance and Accessibility come first, then Spotter's own Library and Imports. Every section starts collapsed.

### General

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `language` | `"en"`, `"fr"` or none | none | Interface language; none follows the system (`LC_ALL`, `LC_MESSAGES`, then `LANG`) |
| `start_screen` | StartScreen | Library | Screen shown on launch (Library / Statistics / Import / Profile) |
| `notifications_enabled` | bool | true | Show toast notifications for success messages (errors always show) |
| `toast_duration` | ToastDuration | Normal (4 s) | Auto-dismiss duration: Short (2 s), Normal (4 s), Long (8 s) |
| `date_format` | DateFormat | YYYY-MM-DD | Date display: YYYY-MM-DD, DD/MM/YYYY, MM/DD/YYYY |

### Appearance

Theme, accent, text size, effects and a live preview come from the shared kit (see below). Spotter adds a Layout section:

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `sidebar_width` | u16 | 220 | Sidebar width in pixels (160-400, presets: 180/220/280) |
| `compact_list` | bool | false | Reduced spacing in game list (4px vs 8px) |
| `show_covers_in_list` | bool | true | Show cover thumbnails in library |

### Accessibility

Vision, Motion and Reading come from the shared kit. Spotter adds Color cues and Navigation:

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `show_status_labels` | bool | true | Show text labels alongside status dots |
| `large_click_targets` | bool | false | Increase button padding (8px to 12px) |

### Library

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `default_status` | GameStatus | Unplayed | Default status for new games |
| `default_platform` | Platform | Steam | Default platform for new games |
| `default_sort_order` | SortOrder | Title A-Z | Default library sort |
| `remember_filters` | bool | true | Persist filter selections across sessions |
| `show_game_descriptions` | bool | false | Show store descriptions in library cards |
| `achievements_display` | AchievementsDisplay | Full | Full list / Summary only / Hidden |
| `confirm_before_delete` | bool | true | Show confirmation dialog before deleting a game |

### Imports

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `download_covers_auto` | bool | true | Automatically download cover art on import |

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

## Languages

The interface ships in English and French. Spotter's own strings are in `src/i18n/en.json` and `src/i18n/fr.json`; the Preferences skeleton and the theme and accent names come from `colony_ui::i18n`. Views never hold text: they call `i18n::t(key)`, `i18n::tf(key, vars)` to fill `{name}` placeholders, or `i18n::tn(key, count, vars)` for a string that depends on a count (`key_one` and `key_other`; French also uses the singular for zero). Numbers with decimals and percentages go through `i18n::decimal` and `i18n::percent`, which follow the French typography (`12,5 %`).

English is canonical: name a new key after its English meaning, write the English first, then the French in both files. `cargo test` fails when the two files hold different keys, when a string is empty or the placeholders differ, when a key is missing or unused, or when one repeats a shared colony-ui label. Platform, program and theme names are never translated. Game statuses are stored in the database by their English name (`GameStatus` `Display`) and shown through `GameStatus::label`.

## Preferences Schema Versioning

Preferences include a `version` field (currently `2`: the move to `preferences.json`), and `Preferences` is `#[serde(default)]`, so a missing field takes its default.

`preferences::parse` also reads the file one field at a time when the whole struct does not parse: a value this build does not accept (for example one added by a newer version) falls back to that field's default, and every other preference keeps its stored value.

If the startup load fails anyway, for example because `preferences.json` is not JSON at all, Spotter shows an error and runs on defaults, and refuses every write of saved data for the rest of the session: preferences, profile and game saves, deletions, exports and the auto-backup are all refused (`Spotter::spawn_write`), so the data on disk stays as it was.
