/// Unit tests for Spotter core logic.
use std::sync::atomic::{AtomicU32, Ordering};

/// Global counter for unique test directories.
static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Create an isolated temp dir with a fresh DB connection.
/// Each call returns a unique path and connection — no global env-var side effects,
/// so tests can safely run in parallel.
fn setup_test_db() -> (std::path::PathBuf, rusqlite::Connection) {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let tmp = std::env::temp_dir().join(format!(
        "spotter_test_{}_{}_{}",
        std::process::id(),
        id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let db_dir = tmp.join("data");
    std::fs::create_dir_all(&db_dir).unwrap();
    let db_path = db_dir.join("spotter.db");
    let conn = spotter::db::open_at(&db_path).expect("Should open test DB");
    (tmp, conn)
}

// ───── Model tests ─────

#[test]
fn playtime_display_hours_and_minutes() {
    let game = spotter::models::Game {
        id: None,
        title: "Test".into(),
        platform: spotter::models::Platform::Steam,
        playtime_minutes: 125,
        achievements_unlocked: 0,
        achievements_total: 0,
        status: spotter::models::GameStatus::Playing,
        rating: None,
        genre: String::new(),
        last_played: String::new(),
        cover_url: String::new(),
        steam_appid: None,
        gog_id: None,
        epic_id: None,
        xbox_id: None,
        psn_id: None,
        notes: String::new(),
        description: String::new(),
        release_date: String::new(),
        review_percent: None,
        tags: String::new(),
    };
    assert_eq!(game.playtime_display(), "2h 05m");
}

#[test]
fn playtime_display_zero() {
    let game = spotter::models::Game {
        id: None,
        title: "Test".into(),
        platform: spotter::models::Platform::Steam,
        playtime_minutes: 0,
        achievements_unlocked: 0,
        achievements_total: 0,
        status: spotter::models::GameStatus::Unplayed,
        rating: None,
        genre: String::new(),
        last_played: String::new(),
        cover_url: String::new(),
        steam_appid: None,
        gog_id: None,
        epic_id: None,
        xbox_id: None,
        psn_id: None,
        notes: String::new(),
        description: String::new(),
        release_date: String::new(),
        review_percent: None,
        tags: String::new(),
    };
    assert_eq!(game.playtime_display(), "0m");
}

#[test]
fn playtime_display_minutes_only() {
    let game = spotter::models::Game {
        id: None,
        title: "Test".into(),
        platform: spotter::models::Platform::Steam,
        playtime_minutes: 45,
        achievements_unlocked: 0,
        achievements_total: 0,
        status: spotter::models::GameStatus::Unplayed,
        rating: None,
        genre: String::new(),
        last_played: String::new(),
        cover_url: String::new(),
        steam_appid: None,
        gog_id: None,
        epic_id: None,
        xbox_id: None,
        psn_id: None,
        notes: String::new(),
        description: String::new(),
        release_date: String::new(),
        review_percent: None,
        tags: String::new(),
    };
    assert_eq!(game.playtime_display(), "45m");
}

#[test]
fn achievement_percent_normal() {
    let game = spotter::models::Game {
        id: None,
        title: "Test".into(),
        platform: spotter::models::Platform::Steam,
        playtime_minutes: 0,
        achievements_unlocked: 25,
        achievements_total: 50,
        status: spotter::models::GameStatus::Playing,
        rating: None,
        genre: String::new(),
        last_played: String::new(),
        cover_url: String::new(),
        steam_appid: None,
        gog_id: None,
        epic_id: None,
        xbox_id: None,
        psn_id: None,
        notes: String::new(),
        description: String::new(),
        release_date: String::new(),
        review_percent: None,
        tags: String::new(),
    };
    assert!((game.achievement_percent() - 50.0).abs() < f32::EPSILON);
}

#[test]
fn achievement_percent_zero_total() {
    let game = spotter::models::Game {
        id: None,
        title: "Test".into(),
        platform: spotter::models::Platform::Steam,
        playtime_minutes: 0,
        achievements_unlocked: 0,
        achievements_total: 0,
        status: spotter::models::GameStatus::Unplayed,
        rating: None,
        genre: String::new(),
        last_played: String::new(),
        cover_url: String::new(),
        steam_appid: None,
        gog_id: None,
        epic_id: None,
        xbox_id: None,
        psn_id: None,
        notes: String::new(),
        description: String::new(),
        release_date: String::new(),
        review_percent: None,
        tags: String::new(),
    };
    assert!((game.achievement_percent() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn format_playtime_shared() {
    assert_eq!(spotter::models::format_playtime(0), "0m");
    assert_eq!(spotter::models::format_playtime(30), "30m");
    assert_eq!(spotter::models::format_playtime(60), "1h 00m");
    assert_eq!(spotter::models::format_playtime(125), "2h 05m");
    assert_eq!(spotter::models::format_playtime(1440), "24h 00m");
}

// ───── Enum parsing tests ─────

#[test]
fn platform_from_str_roundtrip() {
    use spotter::models::Platform;
    assert_eq!(Platform::from_str_name("Steam"), Platform::Steam);
    assert_eq!(Platform::from_str_name("GOG"), Platform::Gog);
    assert_eq!(Platform::from_str_name("Gog"), Platform::Gog);
    assert_eq!(Platform::from_str_name("Epic"), Platform::Epic);
    assert_eq!(
        Platform::from_str_name("PlayStation"),
        Platform::PlayStation
    );
    assert_eq!(Platform::from_str_name("Xbox"), Platform::Xbox);
    assert_eq!(Platform::from_str_name("Nintendo"), Platform::Nintendo);
    // Unknown defaults to Steam
    assert_eq!(Platform::from_str_name("Unknown"), Platform::Steam);
}

#[test]
fn game_status_from_str_roundtrip() {
    use spotter::models::GameStatus;
    assert_eq!(GameStatus::from_str_name("Playing"), GameStatus::Playing);
    assert_eq!(
        GameStatus::from_str_name("Completed"),
        GameStatus::Completed
    );
    assert_eq!(GameStatus::from_str_name("Unplayed"), GameStatus::Unplayed);
    assert_eq!(GameStatus::from_str_name("Backlog"), GameStatus::Unplayed); // legacy alias
    assert_eq!(GameStatus::from_str_name("Dropped"), GameStatus::Dropped);
    assert_eq!(GameStatus::from_str_name("Wishlist"), GameStatus::Wishlist);
    assert_eq!(GameStatus::from_str_name("Unknown"), GameStatus::Unplayed);
}

#[test]
fn platform_display_format() {
    use spotter::models::Platform;
    assert_eq!(format!("{}", Platform::Steam), "Steam");
    assert_eq!(format!("{}", Platform::Gog), "GOG");
    assert_eq!(format!("{}", Platform::Epic), "Epic");
    assert_eq!(format!("{}", Platform::PlayStation), "PlayStation");
}

#[test]
fn game_status_display_format() {
    use spotter::models::GameStatus;
    assert_eq!(format!("{}", GameStatus::Playing), "Playing");
    assert_eq!(format!("{}", GameStatus::Completed), "Completed");
    assert_eq!(format!("{}", GameStatus::Dropped), "Dropped");
}

// ───── Settings tests ─────

#[test]
fn settings_default_values() {
    let s = spotter::models::Settings::default();
    assert_eq!(s.default_status, spotter::models::GameStatus::Unplayed);
    assert_eq!(s.default_platform, spotter::models::Platform::Steam);
    assert!(s.confirm_before_delete);
    assert_eq!(s.start_screen, spotter::models::StartScreen::Library);
    assert_eq!(s.theme_mode, spotter::models::ThemeMode::Dark);
    assert_eq!(s.accent_color, spotter::models::AccentColor::Blue);
    assert!(!s.compact_list);
    assert!(s.show_covers_in_list);
    assert_eq!(s.ui_scale, spotter::models::UiScale::Normal);
    assert!(!s.high_contrast);
    assert!(s.show_status_labels);
    assert!(!s.large_click_targets);
}

#[test]
fn settings_serde_roundtrip() {
    let original = spotter::models::Settings::default();
    let json = serde_json::to_string(&original).unwrap();
    let parsed: spotter::models::Settings = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.theme_mode, original.theme_mode);
    assert_eq!(parsed.accent_color, original.accent_color);
    assert_eq!(parsed.ui_scale, original.ui_scale);
    assert_eq!(parsed.compact_list, original.compact_list);
}

#[test]
fn ui_scale_factors() {
    use spotter::models::UiScale;
    assert!((UiScale::Small.factor() - 0.85).abs() < f32::EPSILON);
    assert!((UiScale::Normal.factor() - 1.0).abs() < f32::EPSILON);
    assert!((UiScale::Large.factor() - 1.2).abs() < f32::EPSILON);
}

// ───── Theme tests ─────

#[test]
fn theme_mode_bg_card_differs() {
    use spotter::models::ThemeMode;
    let dark = ThemeMode::Dark.bg_card();
    let darker = ThemeMode::Darker.bg_card();
    let midnight = ThemeMode::Midnight.bg_card();
    // Each mode should produce different brightness
    assert!(dark.r > darker.r);
    assert!(darker.r > midnight.r);
}

#[test]
fn view_theme_high_contrast_brighter() {
    use spotter::models::Settings;
    use spotter::theme::ViewTheme;

    let mut s = Settings::default();
    let normal = ViewTheme::from_settings(&s);

    s.high_contrast = true;
    let hc = ViewTheme::from_settings(&s);

    // High contrast text should be brighter (higher r value)
    assert!(hc.text_muted.r > normal.text_muted.r);
    assert!(hc.text_dim.r > normal.text_dim.r);
    assert!(hc.border.r > normal.border.r);
}

// ───── DB tests ─────

#[test]
fn db_open_and_basic_crud() {
    let (tmp, conn) = setup_test_db();

    // Save a game
    let mut game = spotter::models::Game {
        id: None,
        title: "Test Game".into(),
        platform: spotter::models::Platform::Steam,
        playtime_minutes: 100,
        achievements_unlocked: 5,
        achievements_total: 10,
        status: spotter::models::GameStatus::Playing,
        rating: Some(8),
        genre: "RPG".into(),
        last_played: "2025-01-01".into(),
        cover_url: String::new(),
        steam_appid: Some(12345),
        gog_id: None,
        epic_id: None,
        xbox_id: None,
        psn_id: None,
        notes: "Test notes".into(),
        description: "A test game".into(),
        release_date: "2025-01-01".into(),
        review_percent: Some(95),
        tags: "rpg, test".into(),
    };

    let id = spotter::db::save_game(&conn, &game).expect("Should save game");
    assert!(id > 0);
    game.id = Some(id);

    // Load games
    let games = spotter::db::load_games(&conn).expect("Should load games");
    assert!(!games.is_empty());
    let loaded = &games[0];
    assert_eq!(loaded.title, "Test Game");
    assert_eq!(loaded.playtime_minutes, 100);
    assert_eq!(loaded.status, spotter::models::GameStatus::Playing);
    assert_eq!(loaded.rating, Some(8));
    assert_eq!(loaded.steam_appid, Some(12345));
    assert_eq!(loaded.notes, "Test notes");

    // Update game
    game.playtime_minutes = 200;
    game.status = spotter::models::GameStatus::Completed;
    spotter::db::save_game(&conn, &game).expect("Should update game");

    let games = spotter::db::load_games(&conn).expect("Should reload games");
    assert_eq!(games[0].playtime_minutes, 200);
    assert_eq!(games[0].status, spotter::models::GameStatus::Completed);

    // Delete game
    spotter::db::delete_game(&conn, id).expect("Should delete game");
    let games = spotter::db::load_games(&conn).expect("Should load after delete");
    assert!(games.is_empty());

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn db_profile_crud() {
    let (tmp, conn) = setup_test_db();

    // Load default profile
    let profile = spotter::db::load_profile(&conn).expect("Should load profile");
    assert_eq!(profile.username, "Player");

    // Update profile
    let mut p = profile;
    p.username = "TestUser".into();
    p.steam_id = "12345".into();
    spotter::db::save_profile(&conn, &p).expect("Should save profile");

    let loaded = spotter::db::load_profile(&conn).expect("Should reload profile");
    assert_eq!(loaded.username, "TestUser");
    assert_eq!(loaded.steam_id, "12345");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn db_settings_roundtrip() {
    let (tmp, conn) = setup_test_db();

    // Load defaults
    let settings = spotter::db::load_settings(&conn).expect("Should load settings");
    assert_eq!(settings.theme_mode, spotter::models::ThemeMode::Dark);

    // Modify and save
    let mut s = settings;
    s.theme_mode = spotter::models::ThemeMode::Midnight;
    s.high_contrast = true;
    s.compact_list = true;
    spotter::db::save_settings(&conn, &s).expect("Should save settings");

    let loaded = spotter::db::load_settings(&conn).expect("Should reload settings");
    assert_eq!(loaded.theme_mode, spotter::models::ThemeMode::Midnight);
    assert!(loaded.high_contrast);
    assert!(loaded.compact_list);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn db_achievements_roundtrip() {
    let (tmp, conn) = setup_test_db();

    let achs = vec![
        spotter::models::Achievement {
            api_name: "ach_1".into(),
            display_name: "First Achievement".into(),
            description: "Do something".into(),
            icon_url: "https://example.com/icon.png".into(),
            icon_gray_url: "https://example.com/gray.png".into(),
            unlocked: true,
            unlock_time: 1700000000,
        },
        spotter::models::Achievement {
            api_name: "ach_2".into(),
            display_name: "Second Achievement".into(),
            description: "Do something else".into(),
            icon_url: String::new(),
            icon_gray_url: String::new(),
            unlocked: false,
            unlock_time: 0,
        },
    ];

    spotter::db::save_achievements(&conn, 12345, &achs).expect("Should save achievements");
    let loaded = spotter::db::load_achievements(&conn, 12345).expect("Should load achievements");
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].display_name, "First Achievement");
    assert!(loaded[0].unlocked);
    assert_eq!(loaded[1].display_name, "Second Achievement");
    assert!(!loaded[1].unlocked);

    // Re-save should replace
    let achs2 = vec![achs[0].clone()];
    spotter::db::save_achievements(&conn, 12345, &achs2).expect("Should replace achievements");
    let loaded = spotter::db::load_achievements(&conn, 12345).expect("Should reload");
    assert_eq!(loaded.len(), 1);

    let _ = std::fs::remove_dir_all(&tmp);
}

// ───── Error type tests ─────

#[test]
fn error_display_format() {
    use spotter::error::SpotterError;
    let db_err = SpotterError::Database("connection failed".into());
    assert_eq!(db_err.to_string(), "Database error: connection failed");

    let net_err = SpotterError::Network("timeout".into());
    assert_eq!(net_err.to_string(), "Network error: timeout");

    let io_err = SpotterError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "not found",
    ));
    assert_eq!(io_err.to_string(), "IO error: not found");
}

#[test]
fn error_converts_to_string() {
    use spotter::error::SpotterError;
    let err = SpotterError::Import("no games found".into());
    let s: String = err.into();
    assert_eq!(s, "Import error: no games found");
}

// ───── Settings versioning tests ─────

#[test]
fn settings_default_has_version() {
    let s = spotter::models::Settings::default();
    assert_eq!(s.version, spotter::models::SETTINGS_VERSION);
}

#[test]
fn settings_version_survives_serialization() {
    let original = spotter::models::Settings::default();
    let json = serde_json::to_string(&original).unwrap();
    let parsed: spotter::models::Settings = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.version, spotter::models::SETTINGS_VERSION);
}

#[test]
fn settings_without_version_gets_default() {
    // Simulate old settings JSON without version field
    let json = r#"{"default_status":"Unplayed","default_platform":"Steam","confirm_before_delete":true,"start_screen":"Library","theme_mode":"Dark","accent_color":"Blue","compact_list":false,"show_covers_in_list":true,"ui_scale":"Normal","sidebar_width":220,"high_contrast":false,"show_status_labels":true,"large_click_targets":false}"#;
    let parsed: spotter::models::Settings = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.version, spotter::models::SETTINGS_VERSION);
}

// ───── CSV export test ─────

#[test]
fn db_export_csv_roundtrip() {
    let (tmp, conn) = setup_test_db();

    // Insert a game
    let game = spotter::models::Game {
        id: None,
        title: "CSV Test Game".into(),
        platform: spotter::models::Platform::Epic,
        playtime_minutes: 300,
        achievements_unlocked: 10,
        achievements_total: 20,
        status: spotter::models::GameStatus::Playing,
        rating: Some(7),
        genre: "Action".into(),
        last_played: "2025-06-15".into(),
        cover_url: String::new(),
        steam_appid: None,
        gog_id: None,
        epic_id: Some("epic123".into()),
        xbox_id: None,
        psn_id: None,
        notes: "Great game".into(),
        description: "An action game".into(),
        release_date: "2024-01-01".into(),
        review_percent: Some(88),
        tags: "action, adventure".into(),
    };
    spotter::db::save_game(&conn, &game).expect("Should save game");

    // Export as CSV (from loaded games)
    let games = spotter::db::load_games(&conn).expect("Should load games");
    let csv = spotter::db::export_games_csv_from_slice(&games).expect("Should export CSV");
    assert!(csv.contains("CSV Test Game"));
    assert!(csv.contains("Epic"));
    assert!(csv.contains("Action"));
    assert!(csv.contains("epic123"));

    let _ = std::fs::remove_dir_all(&tmp);
}

// ───── save_all_games batch test ─────

#[test]
fn db_save_all_games_batch() {
    let (tmp, conn) = setup_test_db();

    let mut games: Vec<spotter::models::Game> = (0..10)
        .map(|i| spotter::models::Game {
            id: None,
            title: format!("Batch Game {}", i),
            platform: spotter::models::Platform::Steam,
            playtime_minutes: i * 10,
            achievements_unlocked: 0,
            achievements_total: 0,
            status: spotter::models::GameStatus::Unplayed,
            rating: None,
            genre: String::new(),
            last_played: String::new(),
            cover_url: String::new(),
            steam_appid: None,
            gog_id: None,
            epic_id: None,
            xbox_id: None,
            psn_id: None,
            notes: String::new(),
            description: String::new(),
            release_date: String::new(),
            review_percent: None,
            tags: String::new(),
        })
        .collect();

    spotter::db::save_all_games(&conn, &mut games).expect("Should batch save");
    assert!(games.iter().all(|g| g.id.is_some()));

    let loaded = spotter::db::load_games(&conn).expect("Should load all");
    assert_eq!(loaded.len(), 10);

    let _ = std::fs::remove_dir_all(&tmp);
}

// ───── Date format tests ─────

#[test]
fn date_format_variants() {
    use spotter::models::DateFormat;
    let date = "2025-03-15";
    assert_eq!(DateFormat::Ymd.format_date(date), "2025-03-15");
    assert_eq!(DateFormat::Dmy.format_date(date), "15/03/2025");
    assert_eq!(DateFormat::Mdy.format_date(date), "03/15/2025");
}

#[test]
fn date_format_empty_input() {
    use spotter::models::DateFormat;
    // Empty strings and dash-placeholders are passed through as-is
    assert_eq!(DateFormat::Ymd.format_date(""), "");
    assert_eq!(DateFormat::Dmy.format_date(""), "");
    assert_eq!(DateFormat::Mdy.format_date("—"), "—");
}

// ───── JSON export test ─────

#[test]
fn db_export_json_roundtrip() {
    let (tmp, conn) = setup_test_db();

    let game = spotter::models::Game {
        id: None,
        title: "JSON Roundtrip".into(),
        platform: spotter::models::Platform::Gog,
        playtime_minutes: 500,
        achievements_unlocked: 0,
        achievements_total: 0,
        status: spotter::models::GameStatus::Completed,
        rating: Some(9),
        genre: "RPG".into(),
        last_played: "2025-05-20".into(),
        cover_url: String::new(),
        steam_appid: None,
        gog_id: Some("gog_abc".into()),
        epic_id: None,
        xbox_id: None,
        psn_id: None,
        notes: String::new(),
        description: "An RPG".into(),
        release_date: "2024-06-01".into(),
        review_percent: None,
        tags: "rpg".into(),
    };
    spotter::db::save_game(&conn, &game).expect("Should save");

    // Export as JSON (from loaded games)
    let games = spotter::db::load_games(&conn).expect("Should load games");
    let json = spotter::db::export_games_json_from_slice(&games).expect("Should export JSON");
    let parsed: Vec<spotter::models::Game> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].title, "JSON Roundtrip");
    assert_eq!(parsed[0].platform, spotter::models::Platform::Gog);
    assert_eq!(parsed[0].rating, Some(9));

    let _ = std::fs::remove_dir_all(&tmp);
}

// ───── API client tests ─────

#[test]
fn url_encode_roundtrip() {
    let original = "hello world&foo=bar";
    let encoded = spotter::api_client::url_encode(original);
    assert!(!encoded.contains(' '));
    assert!(!encoded.contains('&'));
    let decoded = spotter::api_client::url_decode(&encoded);
    assert_eq!(decoded, original);
}

#[test]
fn url_encode_preserves_unreserved() {
    let unreserved = "ABCxyz-_.~0123";
    let encoded = spotter::api_client::url_encode(unreserved);
    assert_eq!(encoded, unreserved, "Unreserved chars should pass through");
}

#[test]
fn url_decode_handles_plus_as_space() {
    let decoded = spotter::api_client::url_decode("hello+world");
    assert_eq!(decoded, "hello world");
}

// ───── Sort order and enum tests ─────

#[test]
fn sort_order_all_variants() {
    let all = spotter::models::SortOrder::all();
    assert_eq!(all.len(), 6);
    assert_eq!(all[0], spotter::models::SortOrder::TitleAsc);
    // FavoritesFirst should be in the list
    assert!(all.contains(&spotter::models::SortOrder::FavoritesFirst));
}

#[test]
fn platform_all_includes_six() {
    let all = spotter::models::Platform::all();
    assert_eq!(all.len(), 6);
}

#[test]
fn game_status_all_includes_five() {
    let all = spotter::models::GameStatus::all();
    assert_eq!(all.len(), 5);
}

#[test]
fn accent_color_all_variants() {
    let all = spotter::models::AccentColor::all();
    assert_eq!(all.len(), 6);
    for ac in all {
        let c = ac.color();
        assert!(c.r > 0.0 || c.g > 0.0 || c.b > 0.0);
    }
}

#[test]
fn toast_duration_millis() {
    assert_eq!(spotter::models::ToastDuration::Short.millis(), 2000);
    assert_eq!(spotter::models::ToastDuration::Normal.millis(), 4000);
    assert_eq!(spotter::models::ToastDuration::Long.millis(), 8000);
}

// ───── DB: extended tests ─────

#[test]
fn db_save_all_games_assigns_ids() {
    let (tmp, conn) = setup_test_db();

    let mut games = vec![
        make_game("Alpha", spotter::models::Platform::Steam),
        make_game("Beta", spotter::models::Platform::Gog),
    ];

    spotter::db::save_all_games(&conn, &mut games).expect("Should save");
    assert!(games[0].id.is_some());
    assert!(games[1].id.is_some());
    assert_ne!(games[0].id, games[1].id);

    let loaded = spotter::db::load_games(&conn).expect("Should load");
    assert_eq!(loaded.len(), 2);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn db_delete_game_removes_from_db() {
    let (tmp, conn) = setup_test_db();

    let mut games = vec![make_game("DeleteMe", spotter::models::Platform::Epic)];
    spotter::db::save_all_games(&conn, &mut games).expect("Should save");
    let id = games[0].id.expect("Should have id after save_all_games");

    spotter::db::delete_game(&conn, id).expect("Should delete");
    let loaded = spotter::db::load_games(&conn).expect("Should load");
    assert!(loaded.is_empty());

    let _ = std::fs::remove_dir_all(&tmp);
}

// ───── contains_ci tests ─────

#[test]
fn contains_ci_ascii_basic() {
    assert!(spotter::models::contains_ci("Hello World", "hello"));
    assert!(spotter::models::contains_ci("Hello World", "world"));
    assert!(spotter::models::contains_ci("UPPERCASE", "upper"));
    assert!(!spotter::models::contains_ci("Hello", "xyz"));
}

#[test]
fn contains_ci_empty_needle() {
    assert!(spotter::models::contains_ci("anything", ""));
    assert!(spotter::models::contains_ci("", ""));
}

#[test]
fn contains_ci_needle_longer_than_haystack() {
    assert!(!spotter::models::contains_ci("Hi", "hello world"));
}

#[test]
fn contains_ci_unicode() {
    assert!(spotter::models::contains_ci("Ü Ö Ä", "ü ö ä"));
    assert!(spotter::models::contains_ci("café", "café"));
}

// ───── Platform & GameStatus icon tests ─────

#[test]
fn platform_icons_non_empty() {
    use spotter::models::Platform;
    for p in Platform::all() {
        assert!(!p.icon().is_empty(), "Platform {:?} should have an icon", p);
    }
}

#[test]
fn game_status_icons_non_empty() {
    use spotter::models::GameStatus;
    for s in GameStatus::all() {
        assert!(
            !s.icon().is_empty(),
            "GameStatus {:?} should have an icon",
            s
        );
    }
}

#[test]
fn platform_colors_non_black() {
    use spotter::models::Platform;
    for p in Platform::all() {
        let c = p.color();
        assert!(
            c.r > 0.0 || c.g > 0.0 || c.b > 0.0,
            "Platform {:?} color should not be black",
            p
        );
    }
}

#[test]
fn game_status_colors_non_black() {
    use spotter::models::GameStatus;
    for s in GameStatus::all() {
        let c = s.color();
        assert!(
            c.r > 0.0 || c.g > 0.0 || c.b > 0.0,
            "GameStatus {:?} color should not be black",
            s
        );
    }
}

// ───── Settings favorites serialization ─────

#[test]
fn settings_favorites_serde_roundtrip() {
    let mut settings = spotter::models::Settings::default();
    settings.favorites = vec![1, 42, 100];
    let json = serde_json::to_string(&settings).unwrap();
    let parsed: spotter::models::Settings = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.favorites, vec![1, 42, 100]);
}

#[test]
fn settings_favorites_default_empty() {
    let settings = spotter::models::Settings::default();
    assert!(settings.favorites.is_empty());
}

// ───── SortOrder FavoritesFirst display ─────

#[test]
fn sort_order_favorites_first_display() {
    assert_eq!(
        format!("{}", spotter::models::SortOrder::FavoritesFirst),
        "Favorites"
    );
}

// ───── SpotterError: remaining variants ─────

#[test]
fn spotter_error_parse_display() {
    use spotter::error::SpotterError;
    let e = SpotterError::Parse("unexpected token".into());
    assert_eq!(e.to_string(), "Parse error: unexpected token");
}

#[test]
fn spotter_error_auth_display() {
    use spotter::error::SpotterError;
    let e = SpotterError::Auth("invalid token".into());
    assert_eq!(e.to_string(), "Authentication error: invalid token");
}

#[test]
fn spotter_error_other_display() {
    use spotter::error::SpotterError;
    let e = SpotterError::Other("something unexpected".into());
    assert_eq!(e.to_string(), "something unexpected");
}

#[test]
fn spotter_error_from_serde_json() {
    use spotter::error::SpotterError;
    let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let e: SpotterError = json_err.into();
    assert!(e.to_string().starts_with("JSON error:"));
}

#[test]
fn spotter_error_from_rusqlite() {
    use spotter::error::SpotterError;
    // Create a rusqlite error by opening a DB at an invalid path
    let result = rusqlite::Connection::open("/nonexistent/path/to/db.sqlite");
    if let Err(rusqlite_err) = result {
        let e: SpotterError = rusqlite_err.into();
        assert!(e.to_string().starts_with("Database error:"));
    }
    // If the open somehow succeeded, the test still passes (platform may allow it)
}

// ───── DB: daily playtime ─────

#[test]
fn db_daily_playtime_returns_zero_for_new_game() {
    let (tmp, conn) = setup_test_db();

    let entries = spotter::db::get_daily_playtime(&conn, 7).expect("Should get daily playtime");
    assert!(entries.is_empty(), "No playtime recorded yet");

    let _ = std::fs::remove_dir_all(&tmp);
}

// ───── DB: achievements by platform ─────

#[test]
fn db_achievements_by_platform_roundtrip() {
    let (tmp, conn) = setup_test_db();

    let achs = vec![
        spotter::models::Achievement {
            api_name: "plat_ach_1".into(),
            display_name: "Platform Ach 1".into(),
            description: "First platform achievement".into(),
            icon_url: "https://example.com/icon1.png".into(),
            icon_gray_url: String::new(),
            unlocked: true,
            unlock_time: 1700000000,
        },
        spotter::models::Achievement {
            api_name: "plat_ach_2".into(),
            display_name: "Platform Ach 2".into(),
            description: "Second platform achievement".into(),
            icon_url: String::new(),
            icon_gray_url: String::new(),
            unlocked: false,
            unlock_time: 0,
        },
    ];

    let platform_id = "xbox_game_123";
    spotter::db::save_achievements_by_platform(&conn, platform_id, &achs)
        .expect("Should save platform achievements");

    let loaded = spotter::db::load_achievements_by_platform(&conn, platform_id)
        .expect("Should load platform achievements");

    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].display_name, "Platform Ach 1");
    assert!(loaded[0].unlocked);
    assert_eq!(loaded[1].display_name, "Platform Ach 2");
    assert!(!loaded[1].unlocked);

    // Re-save should replace
    spotter::db::save_achievements_by_platform(&conn, platform_id, &achs[..1])
        .expect("Should replace");
    let reloaded =
        spotter::db::load_achievements_by_platform(&conn, platform_id).expect("Should reload");
    assert_eq!(reloaded.len(), 1);

    let _ = std::fs::remove_dir_all(&tmp);
}

// ───── DB: multiple games with same title ─────

#[test]
fn db_allows_duplicate_titles_different_platform() {
    let (tmp, conn) = setup_test_db();

    // Two games with same title but different platform should both be saved
    let mut games = vec![
        make_game("Duplicate Title", spotter::models::Platform::Steam),
        make_game("Duplicate Title", spotter::models::Platform::Gog),
    ];
    spotter::db::save_all_games(&conn, &mut games).expect("Should save both");

    let loaded = spotter::db::load_games(&conn).expect("Should load");
    assert_eq!(loaded.len(), 2);
    // Both have the same title
    assert!(loaded.iter().all(|g| g.title == "Duplicate Title"));
    // But different platforms
    let platforms: Vec<_> = loaded.iter().map(|g| &g.platform).collect();
    assert!(platforms.contains(&&spotter::models::Platform::Steam));
    assert!(platforms.contains(&&spotter::models::Platform::Gog));

    let _ = std::fs::remove_dir_all(&tmp);
}

/// Helper to create a minimal Game for testing.
fn make_game(title: &str, platform: spotter::models::Platform) -> spotter::models::Game {
    spotter::models::Game {
        id: None,
        title: title.into(),
        platform,
        playtime_minutes: 0,
        achievements_unlocked: 0,
        achievements_total: 0,
        status: spotter::models::GameStatus::Unplayed,
        rating: None,
        genre: String::new(),
        last_played: String::new(),
        cover_url: String::new(),
        steam_appid: None,
        gog_id: None,
        epic_id: None,
        xbox_id: None,
        psn_id: None,
        notes: String::new(),
        description: String::new(),
        release_date: String::new(),
        review_percent: None,
        tags: String::new(),
    }
}
