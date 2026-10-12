//! Spotter's interface strings, in English and French.
//!
//! Spotter's own strings live in `en.json` and `fr.json` next to this file.
//! The strings every Colony program shares (the Preferences skeleton, theme
//! and accent names) come from `colony_ui::i18n`, and [`t`] reads both, so a
//! view never needs to know which table a key is in.
//!
//! English is canonical: a key is named after its English string and written
//! in English first. Both files hold the same keys; the tests below fail the
//! build when they drift.

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::OnceLock;

use colony_ui::i18n::{self as shared, Locale};

type Table = HashMap<String, String>;

fn table(locale: Locale) -> &'static Table {
    static EN: OnceLock<Table> = OnceLock::new();
    static FR: OnceLock<Table> = OnceLock::new();
    let (cell, json) = match locale {
        Locale::En => (&EN, include_str!("en.json")),
        Locale::Fr => (&FR, include_str!("fr.json")),
    };
    // Both files are parsed by the tests, so this cannot fail in a build
    // that passed them.
    cell.get_or_init(|| serde_json::from_str(json).expect("the locale files are valid JSON"))
}

/// The string for `key` in the active locale. Looks in Spotter's table, then
/// in its English, then in the shared Colony labels, which end with the key
/// itself: a missing string shows up as its key rather than as a blank.
pub fn t(key: &str) -> &'static str {
    table(shared::locale())
        .get(key)
        .or_else(|| table(Locale::En).get(key))
        .map_or_else(|| shared::t(key), String::as_str)
}

/// [`t`] with every `{name}` replaced by its value.
pub fn tf(key: &str, vars: &[(&str, &dyn Display)]) -> String {
    vars.iter().fold(t(key).to_owned(), |text, (name, value)| {
        text.replace(&format!("{{{name}}}"), &value.to_string())
    })
}

/// A string that depends on a count: `{key}_one` for a single thing (and for
/// none, in French), `{key}_other` otherwise. `{count}` and `vars` are filled.
pub fn tn(key: &str, count: usize, vars: &[(&str, &dyn Display)]) -> String {
    let one = match shared::locale() {
        Locale::En => count == 1,
        Locale::Fr => count <= 1,
    };
    let key = format!("{key}_{}", if one { "one" } else { "other" });
    let text = tf(&key, &[("count", &count)]);
    vars.iter().fold(text, |text, (name, value)| {
        text.replace(&format!("{{{name}}}"), &value.to_string())
    })
}

/// `value` with `places` decimals and the interface language's decimal
/// separator (a comma in French).
pub fn decimal(value: f32, places: usize) -> String {
    let text = format!("{value:.places$}");
    match shared::locale() {
        Locale::En => text,
        Locale::Fr => text.replace('.', ","),
    }
}

/// A percentage, written the interface language's way ("50%", "50 %").
pub fn percent(value: f32, places: usize) -> String {
    tf("percent", &[("value", &decimal(value, places))])
}

/// Switch the interface language. `language` is the saved choice ("en",
/// "fr"); `None` follows the system.
pub fn set_language(language: Option<&str>) {
    let tag = language.map(str::to_owned).unwrap_or_else(system_language);
    shared::set_locale(Locale::from_tag(&tag));
}

/// The system language, as the POSIX locale variables give it, most specific
/// first. Empty when none is set (on Windows and macOS, usually), which means
/// English.
fn system_language() -> String {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|var| std::env::var(var).ok())
        .find(|value| !value.is_empty())
        .unwrap_or_default()
}

/// The languages Spotter's interface ships in, by their saved tag and their
/// own name (a language is always offered in itself).
pub const LANGUAGES: [(&str, &str); 2] = [("en", "English"), ("fr", "Français")];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    const EN: &str = include_str!("en.json");
    const FR: &str = include_str!("fr.json");

    fn keys(table: &Table) -> BTreeSet<&str> {
        table.keys().map(String::as_str).collect()
    }

    fn placeholders(text: &str) -> BTreeSet<&str> {
        text.split('{')
            .skip(1)
            .filter_map(|rest| rest.split_once('}').map(|(name, _)| name))
            .collect()
    }

    #[test]
    fn fr_and_en_have_identical_key_sets() {
        let (en, fr) = (keys(table(Locale::En)), keys(table(Locale::Fr)));
        assert_eq!(
            en.difference(&fr).collect::<Vec<_>>(),
            Vec::<&&str>::new(),
            "missing in fr.json"
        );
        assert_eq!(
            fr.difference(&en).collect::<Vec<_>>(),
            Vec::<&&str>::new(),
            "missing in en.json"
        );
    }

    #[test]
    fn no_string_is_empty_and_both_locales_fill_the_same_placeholders() {
        for (key, en) in table(Locale::En) {
            let fr = &table(Locale::Fr)[key];
            assert!(!en.trim().is_empty() && !fr.trim().is_empty(), "{key}");
            assert_eq!(placeholders(en), placeholders(fr), "{key}");
        }
    }

    /// A key written twice in one file keeps only one of its values without a
    /// word: every key appears once (one per line).
    #[test]
    fn no_key_is_written_twice() {
        for (json, locale) in [(EN, Locale::En), (FR, Locale::Fr)] {
            let lines = json
                .lines()
                .filter(|line| line.trim_start().starts_with('"'))
                .count();
            assert_eq!(lines, table(locale).len(), "{}", locale.as_str());
        }
    }

    /// The shared labels (theme and accent names, the Preferences skeleton)
    /// come from colony-ui only: a copy here would drift from them.
    #[test]
    fn no_key_shadows_a_shared_label() {
        for key in table(Locale::En).keys() {
            assert_eq!(shared::t_in(Locale::En, key), key, "{key} is shared");
        }
    }

    /// Every `t("...")`, `tf("...")` and `tn("...")` in the sources names a key that
    /// exists, and every key is used somewhere. The tests never switch the
    /// locale: it is process-wide, and other tests read English strings.
    #[test]
    fn every_key_used_exists_and_every_key_is_used() {
        let mut sources = String::new();
        let mut dirs = vec![std::path::PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src"
        ))];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                // This module's own tests name keys that do not exist.
                if path.is_dir() && !path.ends_with("i18n") {
                    dirs.push(path);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    sources.push_str(&std::fs::read_to_string(path).unwrap());
                }
            }
        }

        let src = sources.as_str();
        // The keys passed as the first argument of `name(...)`, however the
        // call is wrapped.
        let calls = |name: &str| -> Vec<&str> {
            let call = format!("{name}(");
            src.match_indices(&call)
                .filter(|(at, _)| !src[..*at].ends_with(|c: char| c.is_alphanumeric() || c == '_'))
                .filter_map(|(at, _)| {
                    let rest = src[at + call.len()..].trim_start().strip_prefix('"')?;
                    Some(&rest[..rest.find('"')?])
                })
                .collect()
        };
        let plurals: BTreeSet<&str> = calls("tn").into_iter().collect();
        for key in calls("t").into_iter().chain(calls("tf")) {
            assert_ne!(t(key), key, "{key} is used but missing");
        }
        for key in &plurals {
            for form in ["one", "other"] {
                let key = format!("{key}_{form}");
                assert_ne!(t(&key), key, "{key} is used but missing");
            }
        }
        for key in table(Locale::En).keys() {
            let plural = ["_one", "_other"]
                .iter()
                .find_map(|form| key.strip_suffix(form))
                .is_some_and(|base| plurals.contains(base));
            assert!(
                plural || src.contains(&format!("\"{key}\"")),
                "{key} is unused"
            );
        }
    }

    #[test]
    fn a_shared_label_reads_through_t_and_placeholders_are_filled() {
        assert_eq!(t("settings_title"), "Preferences");
        assert_eq!(t("no_such_key"), "no_such_key");
        assert_eq!(
            tf("quick_stat", &[("name", &"Playing"), ("count", &3)]),
            t("quick_stat")
                .replace("{name}", "Playing")
                .replace("{count}", "3")
        );
        assert_eq!(tn("games_count", 1, &[]), "1 game");
        assert_eq!(tn("games_count", 0, &[]), "0 games");
    }
}
