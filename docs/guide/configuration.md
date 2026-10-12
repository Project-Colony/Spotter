# Configuration

Every preference is set on the **Preferences** page. Open it with the
**Spotter** button at the top of the sidebar; the same button, or **Close**,
takes you back to the screen you were on. A change applies at once and is
saved at once: there is no Save button.

The page lists the categories every Colony program shares, **General**,
**Appearance** and **Accessibility**, then Spotter's own, **Library** and
**Imports**. Each category is a column of sections that start collapsed; click
a section's title to open it.

## The preferences file

Preferences are one JSON file, `preferences/preferences.json`, in Spotter's
config folder:

| System | Config folder |
|---|---|
| Linux | `~/.config/Colony/Spotter/` |
| Windows | `%LOCALAPPDATA%\Colony\Spotter\` |
| macOS | `~/Library/Application Support/Colony/Spotter/` |

You can edit it while Spotter is closed. A missing field takes its default, and
a value Spotter does not understand (for example one written by a newer
version) falls back to its default while every other field keeps its value.
If the file is not valid JSON or cannot be read, Spotter starts on defaults,
says so, and refuses to save anything for the rest of the session, so the file
on disk stays as it was.

The file also holds a little state that is not a preference: your favourite
games and, when **Remember filters** is on, the last status and platform
filters.

## Shared Colony preferences

These come from colony-ui, look and behave the same in every Colony program,
and are stored under the same keys.

| Where | Preference | Key | Default | What it changes |
|---|---|---|---|---|
| Appearance, Theme | theme | `selected_theme`, `selected_variant` | Gruvbox, dark | the colour theme, picked from the shared Colony theme list |
| Appearance, Colors & accents | accent | `selected_accent` | none | one of the eight shared accent colours; none keeps the theme's own |
| Appearance, Colors & accents | Auto accent from background | `auto_accent` | off | uses the theme's own accent instead of the one picked |
| Appearance, Typography | Text size | `font_size` | default | small, default or large |
| Appearance, Effects | Animations | `animations` | on | animated transitions and fades |
| Accessibility, Vision | High contrast | `high_contrast` | off | stronger text and border contrast |
| Accessibility, Vision | Dyslexia-friendly font | `dyslexia_font` | off | switches the interface to OpenDyslexic |
| Accessibility, Motion | Reduce motion | `reduce_motion` | off | turns every animation off, whatever **Animations** says, and imported games appear at once instead of one by one |
| Accessibility, Reading | Text size | `text_size_a11y` | default | small, default, large or extra large, multiplied with the Typography size: text ranges from 0.72x to 1.68x |

**Language** is in General: English or French. On the first start Spotter
follows the system language, and falls back to English.

## Spotter's own preferences

| Preference | Key | Default | What it changes |
|---|---|---|---|
| Default status for new games | `default_status` | Unplayed | the status pre-selected in **Add Game** |
| Default platform | `default_platform` | Steam | the platform pre-selected in **Add Game** |
| Confirm before deleting | `confirm_before_delete` | on | asks before a game is removed |
| Start screen | `start_screen` | Library | Library, Statistics, Import or Profile |
| Show notifications | `notifications_enabled` | on | success messages; errors always show |
| Toast duration | `toast_duration` | Normal | how long a message stays: Short (2 s), Normal (4 s), Long (8 s) |
| Date format | `date_format` | YYYY-MM-DD | or DD/MM/YYYY, or MM/DD/YYYY |
| Default sort order | `default_sort_order` | Title A-Z | the library's sort when Spotter starts |
| Remember filters | `remember_filters` | on | keeps the status and platform filters between sessions |
| Show descriptions in library | `show_game_descriptions` | off | store descriptions on the library cards |
| Achievements display | `achievements_display` | Full | the full list, a summary, or nothing on a game's page |
| Compact library list | `compact_list` | off | less space between games |
| Show cover images in library | `show_covers_in_list` | on | cover thumbnails next to the titles |
| Sidebar width | `sidebar_width` | 220 | in pixels, 160 to 400 |
| Auto-download covers on import | `download_covers_auto` | on | downloads covers after every import |
| Show status labels | `show_status_labels` | on | a text label next to each status colour, so status does not rely on colour alone |
| Large click targets | `large_click_targets` | off | more padding on buttons |

## Upgrading from Spotter 0.3 or older

Older versions kept everything in one `Colony/Spotter/` folder and their
settings inside the database. On the first start of a newer version, Spotter:

- copies the database, the exports and the import logs to the folders listed in
  [data-model.md](../internals/data-model.md#where-the-data-lives);
- writes the old settings to `preferences.json`, mapping the old Dark, Darker
  and Midnight themes and the six old accents to the nearest shared theme and
  accent, and the old UI scale to the text size;
- leaves the old folder untouched, so nothing is lost if something goes wrong.
  Delete it yourself once your library is all there. On a system without a
  usable keyring, the old database in that folder still holds your platform
  tokens unencrypted until you do.

If the copy fails, Spotter keeps using the old files where they are and tries
again on the next start.

Covers and achievement icons are not copied: they download again.
