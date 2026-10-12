//! The preferences file, `<config>/preferences/preferences.json` (see
//! `crate::paths`): Spotter's [`Settings`], with the shared Colony
//! preferences flattened in.

use std::path::Path;

use colony_ui::TextSize;
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::db;
use crate::error::SpotterError;
use crate::models::{Settings, SETTINGS_VERSION};

/// Load the preferences. Without a preferences file, the settings an older
/// Spotter saved in the library database are imported and written to it
/// once; without those either, the defaults.
///
/// A file that is not JSON at all is an error: the app then refuses to save
/// anything rather than replace it with the defaults.
pub fn load(path: &Path, conn: &Connection) -> Result<Settings, SpotterError> {
    match std::fs::read_to_string(path) {
        Ok(json) => parse(&json),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let Some(row) = db::load_legacy_settings(conn)? else {
                return Ok(Settings::default());
            };
            let settings = import_legacy(&row)?;
            // The database row stays, so a failed write only means the
            // import runs again next time.
            if let Err(e) = save(path, &settings) {
                eprintln!("[preferences] Could not write {:?}: {}", path, e);
            }
            Ok(settings)
        }
        Err(e) => Err(e.into()),
    }
}

pub fn save(path: &Path, settings: &Settings) -> Result<(), SpotterError> {
    let json = serde_json::to_vec_pretty(settings)?;
    crate::paths::write_atomic(path, &json, false)?;
    Ok(())
}

/// Parse saved preferences one field at a time: a field whose value this
/// build does not accept (for example a value added by a newer version) keeps
/// its default, so one bad value never discards the other preferences.
pub fn parse(json: &str) -> Result<Settings, SpotterError> {
    if let Ok(settings) = serde_json::from_str(json) {
        return Ok(settings);
    }
    let stored: Map<String, Value> = serde_json::from_str(json)?;
    let mut kept = Map::new();
    for (key, value) in stored {
        kept.insert(key.clone(), value);
        if Settings::deserialize(&Value::Object(kept.clone())).is_err() {
            eprintln!("[preferences] Ignoring invalid stored preference {:?}", key);
            kept.remove(&key);
        }
    }
    Ok(Settings::deserialize(Value::Object(kept))?)
}

/// Turn the settings row of Spotter 0.3 and earlier into preferences. The
/// fields keep their names, except the three that became shared Colony
/// preferences: Spotter's own themes, accents and interface scale map to the
/// closest shared theme, accent and text size. High contrast already has the
/// shared name.
fn import_legacy(json: &str) -> Result<Settings, SpotterError> {
    let mut settings = parse(json)?;
    settings.version = SETTINGS_VERSION;
    let old: Value = serde_json::from_str(json)?;
    let field = |key: &str| old.get(key).and_then(Value::as_str);
    let standard = &mut settings.standard;
    if let Some((family, variant)) = field("theme_mode").and_then(legacy_theme) {
        standard.selected_theme = family.into();
        standard.selected_variant = variant.into();
    }
    standard.selected_accent = field("accent_color")
        .and_then(legacy_accent)
        .map(Into::into);
    if let Some(size) = field("ui_scale").and_then(legacy_text_size) {
        standard.text_size_a11y = size;
    }
    Ok(settings)
}

/// The shared theme closest to each of Spotter's old dark themes, by
/// background and accent.
fn legacy_theme(mode: &str) -> Option<(&'static str, &'static str)> {
    Some(match mode {
        "Dark" => ("tokyonight", "night"),
        "Darker" => ("material", "deepocean"),
        "Midnight" => ("modus", "vivendi"),
        _ => return None,
    })
}

/// The shared accent closest to each of Spotter's old accents.
fn legacy_accent(accent: &str) -> Option<&'static str> {
    Some(match accent {
        "Blue" | "Cyan" => "blue",
        "Purple" => "violet",
        "Green" => "green",
        "Gold" => "amber",
        "Red" => "red",
        _ => return None,
    })
}

/// The old interface scale (85, 100 or 120 %) is now the Accessibility text
/// size, which has the same steps.
fn legacy_text_size(scale: &str) -> Option<TextSize> {
    Some(match scale {
        "Small" => TextSize::Small,
        "Normal" => TextSize::Default,
        "Large" => TextSize::Large,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StartScreen;
    use colony_ui::preferences::StandardPreferences;

    struct Dir(std::path::PathBuf);

    impl Dir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "spotter_preferences_{}_{}",
                name,
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn db(&self) -> Connection {
            db::open_at(&self.0.join("spotter.db")).unwrap()
        }

        fn file(&self) -> std::path::PathBuf {
            self.0.join("preferences.json")
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A settings row as Spotter 0.3 wrote it.
    const ROW_03: &str = r#"{"version":1,"default_status":"Playing","default_platform":"GOG",
        "confirm_before_delete":false,"start_screen":"Statistics","notifications_enabled":true,
        "toast_duration":"Long","date_format":"Dmy","default_sort_order":"PlaytimeDesc",
        "remember_filters":true,"last_filter_status":null,"last_filter_platform":"Steam",
        "show_game_descriptions":true,"achievements_display":"Summary",
        "download_covers_auto":false,"favorites":[3,7],"theme_mode":"Darker",
        "accent_color":"Purple","compact_list":true,"show_covers_in_list":false,
        "ui_scale":"Large","sidebar_width":280,"high_contrast":true,
        "show_status_labels":false,"large_click_targets":true}"#;

    #[test]
    fn the_settings_row_is_imported_into_preferences_json() {
        let dir = Dir::new("import");
        let conn = dir.db();
        conn.execute("INSERT INTO settings (id, data) VALUES (1, ?1)", [ROW_03])
            .unwrap();

        let loaded = load(&dir.file(), &conn).unwrap();
        assert_eq!(loaded.version, SETTINGS_VERSION);
        assert_eq!(loaded.start_screen, StartScreen::Statistics);
        assert_eq!(loaded.favorites, [3, 7]);
        assert_eq!(loaded.sidebar_width, 280);
        assert!(loaded.compact_list && loaded.large_click_targets);
        assert!(!loaded.show_status_labels && !loaded.download_covers_auto);
        let standard = &loaded.standard;
        assert_eq!(
            (
                standard.selected_theme.as_str(),
                standard.selected_variant.as_str()
            ),
            ("material", "deepocean")
        );
        assert_eq!(standard.selected_accent.as_deref(), Some("violet"));
        assert_eq!(standard.text_size_a11y, TextSize::Large);
        assert!(standard.high_contrast);

        // Written once, and read back from the file from then on.
        let written = std::fs::read_to_string(dir.file()).unwrap();
        assert!(!written.contains("theme_mode"), "{written}");
        assert_eq!(parse(&written).unwrap(), loaded);
        conn.execute("DELETE FROM settings", []).unwrap();
        assert_eq!(load(&dir.file(), &conn).unwrap(), loaded);
        // The old row was only read.
        let other = dir.0.join("other.json");
        assert_eq!(load(&other, &conn).unwrap(), Settings::default());
    }

    #[test]
    fn every_legacy_theme_and_accent_maps_to_a_shipped_one() {
        for mode in ["Dark", "Darker", "Midnight"] {
            let (family, variant) = legacy_theme(mode).unwrap();
            let family = colony_ui::theme::family(family).expect(mode);
            assert!(family.variant(variant).is_some(), "{mode}");
        }
        for accent in ["Blue", "Purple", "Green", "Gold", "Red", "Cyan"] {
            let key = legacy_accent(accent).unwrap();
            assert!(colony_ui::accent_key_to_color(key).is_some(), "{accent}");
        }
        for (scale, factor) in [("Small", 0.85), ("Normal", 1.0), ("Large", 1.2)] {
            assert_eq!(legacy_text_size(scale).unwrap().factor(), factor);
        }
    }

    #[test]
    fn a_legacy_row_with_unknown_values_keeps_the_rest() {
        let settings = import_legacy(
            r#"{"theme_mode":"Oled","accent_color":42,"ui_scale":"Huge",
                "default_status":"Bogus","compact_list":true}"#,
        )
        .unwrap();
        assert_eq!(settings.standard, StandardPreferences::default());
        assert_eq!(settings.default_status, Settings::default().default_status);
        assert!(settings.compact_list);
    }

    #[test]
    fn a_fresh_install_has_the_defaults_and_writes_nothing() {
        let dir = Dir::new("fresh");
        assert_eq!(load(&dir.file(), &dir.db()).unwrap(), Settings::default());
        assert!(!dir.file().exists());
    }

    #[test]
    fn preferences_round_trip() {
        let dir = Dir::new("round_trip");
        let mut settings = Settings::default();
        settings.standard.selected_theme = "nord".into();
        settings.standard.selected_accent = Some("amber".into());
        settings.compact_list = true;
        save(&dir.file(), &settings).unwrap();
        assert_eq!(load(&dir.file(), &dir.db()).unwrap(), settings);
    }

    #[test]
    fn one_bad_value_keeps_the_other_preferences() {
        let settings =
            parse(r#"{"sidebar_width":"wide","compact_list":true,"selected_accent":"green"}"#)
                .unwrap();
        assert_eq!(settings.sidebar_width, Settings::default().sidebar_width);
        assert!(settings.compact_list);
        assert_eq!(settings.standard.selected_accent.as_deref(), Some("green"));
    }

    #[test]
    fn a_file_that_is_not_json_is_an_error() {
        let dir = Dir::new("broken");
        std::fs::write(dir.file(), "not json").unwrap();
        assert!(load(&dir.file(), &dir.db()).is_err());
    }
}
