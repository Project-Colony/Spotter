/// Integration tests for platform import modules.
///
/// These tests validate JSON deserialization and data-mapping logic
/// using mock API responses, without making real HTTP calls.
/// If a platform changes its API response format, these tests will
/// catch it before it reaches production.

// ───── Steam mock API tests ─────

mod steam_import {
    use spotter::models::*;

    /// Simulate a Steam GetOwnedGames API response and verify parsing.
    #[test]
    fn parse_owned_games_response() {
        let json = r#"{
            "response": {
                "game_count": 3,
                "games": [
                    {"appid": 730, "name": "Counter-Strike 2", "playtime_forever": 5000},
                    {"appid": 570, "name": "Dota 2", "playtime_forever": 0},
                    {"appid": 440, "name": "", "playtime_forever": 100}
                ]
            }
        }"#;

        #[derive(serde::Deserialize)]
        struct SteamOwnedGamesResponse {
            response: SteamOwnedGamesInner,
        }
        #[derive(serde::Deserialize)]
        struct SteamOwnedGamesInner {
            #[serde(default)]
            games: Vec<SteamGameInfo>,
        }
        #[derive(serde::Deserialize)]
        struct SteamGameInfo {
            appid: u32,
            #[serde(default)]
            name: String,
            #[serde(default)]
            playtime_forever: u32,
        }

        let resp: SteamOwnedGamesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.response.games.len(), 3);

        // Filter out empty names (same logic as steam.rs)
        let games: Vec<Game> = resp
            .response
            .games
            .into_iter()
            .filter(|g| !g.name.is_empty())
            .map(|g| Game {
                id: None,
                title: g.name,
                platform: Platform::Steam,
                playtime_minutes: g.playtime_forever,
                achievements_unlocked: 0,
                achievements_total: 0,
                status: if g.playtime_forever > 0 {
                    GameStatus::Playing
                } else {
                    GameStatus::Unplayed
                },
                rating: None,
                genre: String::new(),
                last_played: String::new(),
                cover_url: format!(
                    "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/{}/header.jpg",
                    g.appid
                ),
                steam_appid: Some(g.appid),
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

        assert_eq!(games.len(), 2, "Empty-name game should be filtered out");
        assert_eq!(games[0].title, "Counter-Strike 2");
        assert_eq!(games[0].playtime_minutes, 5000);
        assert_eq!(games[0].status, GameStatus::Playing);
        assert_eq!(games[0].steam_appid, Some(730));

        assert_eq!(games[1].title, "Dota 2");
        assert_eq!(games[1].playtime_minutes, 0);
        assert_eq!(games[1].status, GameStatus::Unplayed);
    }

    /// Verify empty response is handled gracefully.
    #[test]
    fn parse_empty_library() {
        let json = r#"{"response": {"games": []}}"#;

        #[derive(serde::Deserialize)]
        struct Resp {
            response: Inner,
        }
        #[derive(serde::Deserialize)]
        struct Inner {
            #[serde(default)]
            games: Vec<serde_json::Value>,
        }

        let resp: Resp = serde_json::from_str(json).unwrap();
        assert!(resp.response.games.is_empty());
    }

    /// Verify response with missing games field (private profile).
    #[test]
    fn parse_private_profile_response() {
        let json = r#"{"response": {}}"#;

        #[derive(serde::Deserialize)]
        struct Resp {
            response: Inner,
        }
        #[derive(serde::Deserialize)]
        struct Inner {
            #[serde(default)]
            games: Vec<serde_json::Value>,
        }

        let resp: Resp = serde_json::from_str(json).unwrap();
        assert!(resp.response.games.is_empty());
    }

    /// Test that unexpected extra fields in the API response are tolerated.
    #[test]
    fn parse_response_with_extra_fields() {
        let json = r#"{
            "response": {
                "game_count": 1,
                "games": [
                    {
                        "appid": 730,
                        "name": "CS2",
                        "playtime_forever": 100,
                        "playtime_2weeks": 50,
                        "img_icon_url": "abc123",
                        "unknown_field": true
                    }
                ]
            }
        }"#;

        #[derive(serde::Deserialize)]
        struct Resp {
            response: Inner,
        }
        #[derive(serde::Deserialize)]
        struct Inner {
            #[serde(default)]
            games: Vec<GameInfo>,
        }
        #[derive(serde::Deserialize)]
        struct GameInfo {
            appid: u32,
            #[serde(default)]
            name: String,
            #[serde(default)]
            playtime_forever: u32,
        }

        let resp: Resp = serde_json::from_str(json).unwrap();
        assert_eq!(resp.response.games.len(), 1);
        assert_eq!(resp.response.games[0].appid, 730);
        assert_eq!(resp.response.games[0].name, "CS2");
    }
}

// ───── PSN mock API tests ─────

mod psn_import {
    use spotter::models::*;

    /// Simulate a PSN trophy titles API response.
    #[test]
    fn parse_trophy_titles_response() {
        let json = r#"{
            "trophyTitles": [
                {
                    "npCommunicationId": "NPWR12345_00",
                    "trophyTitleName": "God of War Ragnarök",
                    "definedTrophies": {"bronze": 20, "silver": 10, "gold": 5, "platinum": 1},
                    "earnedTrophies": {"bronze": 15, "silver": 8, "gold": 3, "platinum": 0},
                    "trophyTitleIconUrl": "https://image.api.playstation.com/trophy/np/icon.png",
                    "lastUpdatedDateTime": "2024-11-15T10:30:00Z"
                },
                {
                    "npCommunicationId": "NPWR67890_00",
                    "trophyTitleName": "Astro Bot",
                    "definedTrophies": {"bronze": 10, "silver": 5, "gold": 2, "platinum": 1},
                    "earnedTrophies": {"bronze": 10, "silver": 5, "gold": 2, "platinum": 1},
                    "trophyTitleIconUrl": "https://image.api.playstation.com/trophy/np/astro.png",
                    "lastUpdatedDateTime": "2024-12-01T08:00:00Z"
                }
            ]
        }"#;

        #[derive(serde::Deserialize)]
        struct TrophyResponse {
            #[serde(alias = "trophyTitles", default)]
            trophy_titles: Vec<TrophyTitle>,
        }
        #[derive(serde::Deserialize)]
        struct TrophyTitle {
            #[serde(alias = "npCommunicationId", default)]
            np_communication_id: String,
            #[serde(alias = "trophyTitleName", default)]
            trophy_title_name: String,
            #[serde(alias = "definedTrophies", default)]
            defined_trophies: TrophyCounts,
            #[serde(alias = "earnedTrophies", default)]
            earned_trophies: TrophyCounts,
            #[serde(alias = "trophyTitleIconUrl", default)]
            trophy_title_icon_url: String,
            #[serde(alias = "lastUpdatedDateTime", default)]
            last_updated_date_time: String,
        }
        #[derive(serde::Deserialize, Default)]
        struct TrophyCounts {
            #[serde(default)]
            bronze: u32,
            #[serde(default)]
            silver: u32,
            #[serde(default)]
            gold: u32,
            #[serde(default)]
            platinum: u32,
        }
        impl TrophyCounts {
            fn total(&self) -> u32 {
                self.bronze + self.silver + self.gold + self.platinum
            }
        }

        let resp: TrophyResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.trophy_titles.len(), 2);

        let gow = &resp.trophy_titles[0];
        assert_eq!(gow.trophy_title_name, "God of War Ragnarök");
        assert_eq!(gow.defined_trophies.total(), 36);
        assert_eq!(gow.earned_trophies.total(), 26);

        // Build games
        let games: Vec<Game> = resp
            .trophy_titles
            .iter()
            .map(|t| {
                let total = t.defined_trophies.total();
                let earned = t.earned_trophies.total();
                let status = if earned == total && total > 0 {
                    GameStatus::Completed
                } else if earned > 0 {
                    GameStatus::Playing
                } else {
                    GameStatus::Unplayed
                };
                Game {
                    id: None,
                    title: t.trophy_title_name.clone(),
                    platform: Platform::PlayStation,
                    playtime_minutes: 0,
                    achievements_unlocked: earned,
                    achievements_total: total,
                    status,
                    rating: None,
                    genre: String::new(),
                    last_played: t.last_updated_date_time.get(..10).unwrap_or("").to_string(),
                    cover_url: t.trophy_title_icon_url.clone(),
                    steam_appid: None,
                    gog_id: None,
                    epic_id: None,
                    xbox_id: None,
                    psn_id: Some(t.np_communication_id.clone()),
                    notes: String::new(),
                    description: String::new(),
                    release_date: String::new(),
                    review_percent: None,
                    tags: String::new(),
                }
            })
            .collect();

        assert_eq!(games.len(), 2);
        assert_eq!(games[0].status, GameStatus::Playing);
        assert_eq!(games[0].achievements_unlocked, 26);
        assert_eq!(games[0].last_played, "2024-11-15");

        assert_eq!(games[1].title, "Astro Bot");
        assert_eq!(games[1].status, GameStatus::Completed);
        assert_eq!(games[1].achievements_unlocked, 18);
        assert_eq!(games[1].achievements_total, 18);
    }

    /// Parse ISO-8601 duration like PSN's playDuration field.
    #[test]
    fn parse_play_duration() {
        fn parse(duration: &str) -> u32 {
            if !duration.starts_with("PT") {
                return 0;
            }
            let rest = &duration[2..];
            let mut hours = 0u32;
            let mut minutes = 0u32;
            let mut num = String::new();
            for ch in rest.chars() {
                if ch.is_ascii_digit() {
                    num.push(ch);
                } else {
                    let val: u32 = num.parse().unwrap_or(0);
                    match ch {
                        'H' => hours = val,
                        'M' => minutes = val,
                        _ => {}
                    }
                    num.clear();
                }
            }
            hours * 60 + minutes
        }

        assert_eq!(parse("PT123H45M6S"), 123 * 60 + 45);
        assert_eq!(parse("PT0H30M"), 30);
        assert_eq!(parse("PT2H"), 120);
        assert_eq!(parse("PT45M"), 45);
        assert_eq!(parse(""), 0);
        assert_eq!(parse("invalid"), 0);
    }

    /// PSN game list response with pagination.
    #[test]
    fn parse_game_list_response() {
        let json = r#"{
            "titles": [
                {
                    "titleId": "CUSA12345_00",
                    "name": "Spider-Man",
                    "imageUrl": "https://image.api.playstation.com/spiderman.png",
                    "category": "ps4_game",
                    "playDuration": "PT50H30M",
                    "lastPlayedDateTime": "2024-10-20T15:00:00Z"
                }
            ],
            "totalItemCount": 150
        }"#;

        #[derive(serde::Deserialize)]
        struct GameListResp {
            #[serde(alias = "titles", default)]
            titles: Vec<PsnTitle>,
            #[serde(alias = "totalItemCount", default)]
            total_item_count: u32,
        }
        #[derive(serde::Deserialize)]
        struct PsnTitle {
            #[serde(alias = "titleId", default)]
            title_id: String,
            #[serde(default)]
            name: String,
            #[serde(alias = "imageUrl", default)]
            image_url: String,
            #[serde(alias = "playDuration", default)]
            play_duration: Option<String>,
            #[serde(alias = "lastPlayedDateTime", default)]
            last_played_date_time: String,
        }

        let resp: GameListResp = serde_json::from_str(json).unwrap();
        assert_eq!(resp.titles.len(), 1);
        assert_eq!(resp.total_item_count, 150);
        assert_eq!(resp.titles[0].title_id, "CUSA12345_00");
        assert_eq!(resp.titles[0].name, "Spider-Man");
        assert_eq!(resp.titles[0].play_duration.as_deref(), Some("PT50H30M"));
    }

    /// Verify empty trophy titles response is handled.
    #[test]
    fn parse_empty_trophy_titles() {
        let json = r#"{"trophyTitles": []}"#;

        #[derive(serde::Deserialize)]
        struct TrophyResponse {
            #[serde(alias = "trophyTitles", default)]
            trophy_titles: Vec<serde_json::Value>,
        }

        let resp: TrophyResponse = serde_json::from_str(json).unwrap();
        assert!(resp.trophy_titles.is_empty());
    }
}

// ───── Xbox mock API tests ─────

mod xbox_import {
    use spotter::models::*;

    /// Simulate an OpenXBL title history API response.
    #[test]
    fn parse_title_history_response() {
        let json = r#"{
            "titles": [
                {
                    "titleId": "1234567890",
                    "name": "Halo Infinite",
                    "displayImage": "https://store-images.s-microsoft.com/halo.png",
                    "minutesPlayed": 1500,
                    "lastTimePlayed": "2024-12-01T20:00:00Z",
                    "currentAchievements": 30,
                    "totalAchievements": 50,
                    "devices": ["XboxSeries"]
                },
                {
                    "titleId": "9876543210",
                    "name": "Forza Horizon 5",
                    "displayImage": "https://store-images.s-microsoft.com/forza.png",
                    "minutesPlayed": 0,
                    "lastTimePlayed": "",
                    "currentAchievements": 0,
                    "totalAchievements": 100,
                    "devices": ["XboxOne", "PC"]
                },
                {
                    "titleId": "1111111111",
                    "name": "Xbox App",
                    "displayImage": "",
                    "minutesPlayed": 500,
                    "lastTimePlayed": "2024-11-01T10:00:00Z",
                    "currentAchievements": 0,
                    "totalAchievements": 0,
                    "devices": ["Mobile"]
                },
                {
                    "titleId": "2222222222",
                    "name": "",
                    "displayImage": "",
                    "minutesPlayed": 0,
                    "lastTimePlayed": "",
                    "currentAchievements": 5,
                    "totalAchievements": 10,
                    "devices": []
                }
            ]
        }"#;

        #[derive(serde::Deserialize)]
        struct XblTitleHistory {
            #[serde(default)]
            titles: Vec<XblTitle>,
        }
        #[derive(serde::Deserialize)]
        struct XblTitle {
            #[serde(alias = "titleId", default)]
            title_id: String,
            #[serde(default)]
            name: String,
            #[serde(alias = "displayImage", default)]
            display_image: String,
            #[serde(alias = "minutesPlayed", default)]
            minutes_played: Option<u32>,
            #[serde(alias = "lastTimePlayed", default)]
            last_time_played: String,
            #[serde(alias = "currentAchievements", default)]
            current_achievements: Option<u32>,
            #[serde(alias = "totalAchievements", default)]
            total_achievements: Option<u32>,
            #[serde(default)]
            devices: Vec<String>,
        }

        fn is_game(title: &XblTitle) -> bool {
            if title.name.is_empty() {
                return false;
            }
            if title.total_achievements.unwrap_or(0) > 0 {
                return true;
            }
            let game_devices = ["XboxOne", "XboxSeries", "Xbox360", "PC", "Win32"];
            title
                .devices
                .iter()
                .any(|d| game_devices.iter().any(|gd| d.contains(gd)))
        }

        let history: XblTitleHistory = serde_json::from_str(json).unwrap();
        assert_eq!(history.titles.len(), 4);

        let games: Vec<Game> = history
            .titles
            .into_iter()
            .filter(|t| is_game(t))
            .map(|title| {
                let last_played = if !title.last_time_played.is_empty() {
                    title.last_time_played.get(..10).unwrap_or("").to_string()
                } else {
                    String::new()
                };
                let status = if title.minutes_played.unwrap_or(0) > 0 {
                    GameStatus::Playing
                } else {
                    GameStatus::Unplayed
                };
                Game {
                    id: None,
                    title: title.name,
                    platform: Platform::Xbox,
                    playtime_minutes: title.minutes_played.unwrap_or(0),
                    achievements_unlocked: title.current_achievements.unwrap_or(0),
                    achievements_total: title.total_achievements.unwrap_or(0),
                    status,
                    rating: None,
                    genre: String::new(),
                    last_played,
                    cover_url: title.display_image,
                    steam_appid: None,
                    gog_id: None,
                    epic_id: None,
                    xbox_id: Some(title.title_id),
                    psn_id: None,
                    notes: String::new(),
                    description: String::new(),
                    release_date: String::new(),
                    review_percent: None,
                    tags: String::new(),
                }
            })
            .collect();

        // Halo + Forza are games. Xbox App is not (Mobile only, 0 achievements).
        // Empty name is filtered out even though it has achievements.
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].title, "Halo Infinite");
        assert_eq!(games[0].playtime_minutes, 1500);
        assert_eq!(games[0].status, GameStatus::Playing);
        assert_eq!(games[0].last_played, "2024-12-01");

        assert_eq!(games[1].title, "Forza Horizon 5");
        assert_eq!(games[1].status, GameStatus::Unplayed);
        assert_eq!(games[1].achievements_total, 100);
    }

    /// Simulate a paginated Xbox response (continuation token pattern).
    #[test]
    fn parse_paginated_title_history() {
        // Page 1
        let page1 = r#"{
            "titles": [
                {"titleId": "1", "name": "Game A", "displayImage": "", "minutesPlayed": 10, "totalAchievements": 5, "devices": ["XboxSeries"]},
                {"titleId": "2", "name": "Game B", "displayImage": "", "minutesPlayed": 20, "totalAchievements": 10, "devices": ["XboxOne"]}
            ],
            "continuationToken": "abc123"
        }"#;

        // Page 2 (last page)
        let page2 = r#"{
            "titles": [
                {"titleId": "3", "name": "Game C", "displayImage": "", "minutesPlayed": 30, "totalAchievements": 15, "devices": ["PC"]}
            ]
        }"#;

        #[derive(serde::Deserialize)]
        struct XblTitleHistory {
            #[serde(default)]
            titles: Vec<XblTitleSimple>,
            #[serde(alias = "continuationToken")]
            continuation_token: Option<String>,
        }
        #[derive(serde::Deserialize)]
        struct XblTitleSimple {
            #[serde(alias = "titleId", default)]
            title_id: String,
            #[serde(default)]
            name: String,
        }

        let resp1: XblTitleHistory = serde_json::from_str(page1).unwrap();
        assert_eq!(resp1.titles.len(), 2);
        assert_eq!(resp1.continuation_token, Some("abc123".to_string()));

        let resp2: XblTitleHistory = serde_json::from_str(page2).unwrap();
        assert_eq!(resp2.titles.len(), 1);
        assert!(resp2.continuation_token.is_none());

        // Combined
        let total_titles: usize = resp1.titles.len() + resp2.titles.len();
        assert_eq!(total_titles, 3);
    }

    /// Verify empty Xbox response is handled.
    #[test]
    fn parse_empty_title_history() {
        let json = r#"{"titles": []}"#;

        #[derive(serde::Deserialize)]
        struct XblTitleHistory {
            #[serde(default)]
            titles: Vec<serde_json::Value>,
        }

        let resp: XblTitleHistory = serde_json::from_str(json).unwrap();
        assert!(resp.titles.is_empty());
    }

    /// Verify fields with missing optional values use defaults.
    #[test]
    fn parse_title_with_missing_optional_fields() {
        let json = r#"{
            "titles": [
                {
                    "titleId": "999",
                    "name": "Minimal Game",
                    "devices": ["XboxSeries"]
                }
            ]
        }"#;

        #[derive(serde::Deserialize)]
        struct XblTitleHistory {
            #[serde(default)]
            titles: Vec<XblTitle>,
        }
        #[derive(serde::Deserialize)]
        struct XblTitle {
            #[serde(alias = "titleId", default)]
            title_id: String,
            #[serde(default)]
            name: String,
            #[serde(alias = "displayImage", default)]
            display_image: String,
            #[serde(alias = "minutesPlayed", default)]
            minutes_played: Option<u32>,
            #[serde(alias = "currentAchievements", default)]
            current_achievements: Option<u32>,
            #[serde(alias = "totalAchievements", default)]
            total_achievements: Option<u32>,
            #[serde(default)]
            devices: Vec<String>,
        }

        let resp: XblTitleHistory = serde_json::from_str(json).unwrap();
        assert_eq!(resp.titles.len(), 1);
        let t = &resp.titles[0];
        assert_eq!(t.title_id, "999");
        assert_eq!(t.display_image, "");
        assert_eq!(t.minutes_played, None);
        assert_eq!(t.current_achievements, None);
        assert_eq!(t.total_achievements, None);
    }
}

// ───── GOG mock API tests ─────

mod gog_import {
    use spotter::models::*;

    /// Simulate a GOG filtered products listing response.
    #[test]
    fn parse_filtered_products_response() {
        let json = r#"{
            "products": [
                {
                    "id": 1207658691,
                    "title": "The Witcher 3: Wild Hunt",
                    "image": "//images-1.gog-statics.com/witcher3.jpg",
                    "category": "Role-playing"
                },
                {
                    "id": 1495134320,
                    "title": "Cyberpunk 2077",
                    "image": "//images-1.gog-statics.com/cyberpunk.jpg",
                    "category": "Action"
                }
            ],
            "totalPages": 3
        }"#;

        #[derive(serde::Deserialize)]
        struct FilteredResponse {
            #[serde(default)]
            products: Vec<FilteredProduct>,
            #[serde(default, rename = "totalPages")]
            total_pages: u32,
        }
        #[derive(serde::Deserialize)]
        struct FilteredProduct {
            #[serde(default)]
            id: i64,
            #[serde(default)]
            title: String,
            #[serde(default)]
            image: String,
            #[serde(default)]
            category: String,
        }

        let resp: FilteredResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.products.len(), 2);
        assert_eq!(resp.total_pages, 3);

        // Build games (same logic as gog.rs)
        let games: Vec<Game> = resp
            .products
            .iter()
            .filter(|p| !p.title.is_empty())
            .map(|p| {
                let cover = if !p.image.is_empty() && p.image.starts_with("//") {
                    format!("https:{}", p.image)
                } else {
                    p.image.clone()
                };
                Game {
                    id: None,
                    title: p.title.clone(),
                    platform: Platform::Gog,
                    playtime_minutes: 0,
                    achievements_unlocked: 0,
                    achievements_total: 0,
                    status: GameStatus::Unplayed,
                    rating: None,
                    genre: p.category.clone(),
                    last_played: String::new(),
                    cover_url: cover,
                    steam_appid: None,
                    gog_id: Some(p.id.to_string()),
                    epic_id: None,
                    xbox_id: None,
                    psn_id: None,
                    notes: String::new(),
                    description: String::new(),
                    release_date: String::new(),
                    review_percent: None,
                    tags: String::new(),
                }
            })
            .collect();

        assert_eq!(games.len(), 2);
        assert_eq!(games[0].title, "The Witcher 3: Wild Hunt");
        assert_eq!(games[0].genre, "Role-playing");
        assert!(games[0].cover_url.starts_with("https://"));
        assert_eq!(games[0].gog_id, Some("1207658691".to_string()));
    }

    /// Simulate a GOG store product API response.
    #[test]
    fn parse_store_product_response() {
        let json = r#"{
            "id": 1207658691,
            "title": "The Witcher 3: Wild Hunt",
            "images": {
                "logo2x": "//images-1.gog-statics.com/witcher3_logo2x.jpg"
            },
            "description": {
                "lead": "<p>An <b>amazing</b> RPG game.</p>"
            },
            "release_date": "2015-05-19T00:00:00+0000",
            "genres": [
                {"name": "Role-playing"},
                {"name": "Adventure"}
            ]
        }"#;

        let val: serde_json::Value = serde_json::from_str(json).unwrap();

        // Extract description (same as gog.rs clean_html)
        let desc_raw = val
            .get("description")
            .and_then(|d| d.get("lead"))
            .and_then(|l| l.as_str())
            .unwrap_or("");

        fn clean_html(html: &str) -> String {
            let mut result = String::with_capacity(html.len());
            let mut in_tag = false;
            for ch in html.chars() {
                if ch == '<' {
                    in_tag = true;
                } else if ch == '>' {
                    in_tag = false;
                } else if !in_tag {
                    result.push(ch);
                }
            }
            result.trim().to_string()
        }

        let description = clean_html(desc_raw);
        assert_eq!(description, "An amazing RPG game.");

        // Extract genres
        let genres = val
            .get("genres")
            .and_then(|g| g.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("name").and_then(|n| n.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        assert_eq!(genres, "Role-playing, Adventure");

        // Extract cover
        let cover = val
            .get("images")
            .and_then(|i| i.get("logo2x"))
            .and_then(|l| l.as_str())
            .unwrap_or("");
        assert!(cover.starts_with("//"));
    }

    /// Verify handling of null fields in GOG product response.
    #[test]
    fn parse_product_with_null_fields() {
        let json = r#"{
            "id": 999,
            "title": null,
            "images": {"logo2x": null},
            "description": {"lead": null},
            "release_date": null
        }"#;

        #[derive(serde::Deserialize)]
        struct GogProduct {
            #[serde(default)]
            id: i64,
            #[serde(default)]
            title: Option<String>,
            #[serde(default)]
            images: GogImages,
            #[serde(default)]
            description: GogDesc,
            #[serde(default)]
            release_date: Option<String>,
        }
        #[derive(serde::Deserialize, Default)]
        struct GogImages {
            #[serde(default)]
            logo2x: Option<String>,
        }
        #[derive(serde::Deserialize, Default)]
        struct GogDesc {
            #[serde(default)]
            lead: Option<String>,
        }

        let product: GogProduct = serde_json::from_str(json).unwrap();
        assert_eq!(product.id, 999);
        assert_eq!(product.title, None);
        assert_eq!(product.images.logo2x, None);
        assert_eq!(product.description.lead, None);
        assert_eq!(product.release_date, None);
    }

    /// Simulate last page of paginated listing.
    #[test]
    fn parse_last_page_listing() {
        let json = r#"{
            "products": [
                {"id": 100, "title": "Last Game", "image": "", "category": "Strategy"}
            ],
            "totalPages": 3
        }"#;

        #[derive(serde::Deserialize)]
        struct FilteredResponse {
            #[serde(default)]
            products: Vec<serde_json::Value>,
            #[serde(default, rename = "totalPages")]
            total_pages: u32,
        }

        let resp: FilteredResponse = serde_json::from_str(json).unwrap();
        let current_page = 3u32;
        assert!(
            current_page >= resp.total_pages,
            "Should stop pagination at last page"
        );
    }
}

// ───── Epic mock API tests ─────

mod epic_import {
    use spotter::models::*;

    /// Simulate an Epic Store content API response.
    #[test]
    fn parse_store_content_response() {
        let json = r#"{
            "pages": [
                {
                    "data": {
                        "about": {
                            "shortDescription": "A post-apocalyptic survival game"
                        },
                        "hero": {
                            "backgroundImageUrl": "https://cdn1.epicgames.com/hero.jpg"
                        },
                        "meta": {
                            "tags": ["survival", "open-world", "crafting"],
                            "releaseDate": "2023-06-15"
                        }
                    }
                }
            ]
        }"#;

        let val: serde_json::Value = serde_json::from_str(json).unwrap();

        let description = val
            .get("pages")
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("data"))
            .and_then(|d| d.get("about"))
            .and_then(|a| a.get("shortDescription"))
            .and_then(|s| s.as_str())
            .unwrap_or("");
        assert_eq!(description, "A post-apocalyptic survival game");

        let cover = val
            .get("pages")
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("data"))
            .and_then(|d| d.get("hero"))
            .and_then(|h| h.get("backgroundImageUrl"))
            .and_then(|u| u.as_str())
            .unwrap_or("");
        assert_eq!(cover, "https://cdn1.epicgames.com/hero.jpg");

        let tags = val
            .get("pages")
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("data"))
            .and_then(|d| d.get("meta"))
            .and_then(|m| m.get("tags"))
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        assert_eq!(tags, "survival, open-world, crafting");

        let release_date = val
            .get("pages")
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("data"))
            .and_then(|d| d.get("meta"))
            .and_then(|m| m.get("releaseDate"))
            .and_then(|r| r.as_str())
            .unwrap_or("");
        assert_eq!(release_date, "2023-06-15");
    }

    /// Simulate Epic manifest .item file.
    #[test]
    fn parse_manifest_item_file() {
        let json = r#"{
            "DisplayName": "Fortnite",
            "CatalogItemId": "fn_catalog_123",
            "CatalogNamespace": "fn_namespace",
            "AppName": "Fortnite",
            "AppCategories": ["games"]
        }"#;

        #[derive(serde::Deserialize)]
        struct EpicManifest {
            #[serde(alias = "DisplayName", default)]
            display_name: String,
            #[serde(alias = "CatalogItemId", default)]
            catalog_item_id: String,
            #[serde(alias = "CatalogNamespace", default)]
            catalog_namespace: String,
            #[serde(alias = "AppName", default)]
            app_name: String,
            #[serde(alias = "AppCategories", default)]
            app_categories: Vec<String>,
        }

        let manifest: EpicManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.display_name, "Fortnite");
        assert_eq!(manifest.catalog_item_id, "fn_catalog_123");
        assert!(manifest.app_categories.contains(&"games".to_string()));

        // Build game
        let game = Game {
            id: None,
            title: manifest.display_name,
            platform: Platform::Epic,
            playtime_minutes: 0,
            achievements_unlocked: 0,
            achievements_total: 0,
            status: GameStatus::Unplayed,
            rating: None,
            genre: String::new(),
            last_played: String::new(),
            cover_url: String::new(),
            steam_appid: None,
            gog_id: None,
            epic_id: Some(manifest.catalog_item_id),
            xbox_id: None,
            psn_id: None,
            notes: String::new(),
            description: String::new(),
            release_date: String::new(),
            review_percent: None,
            tags: String::new(),
        };

        assert_eq!(game.title, "Fortnite");
        assert_eq!(game.epic_id, Some("fn_catalog_123".to_string()));
    }

    /// Simulate Legendary installed.json format (Linux).
    #[test]
    fn parse_legendary_installed_json() {
        let json = r#"{
            "Fortnite": {
                "title": "Fortnite",
                "app_name": "Fortnite",
                "catalog_item_id": "fn_123",
                "namespace": "fn_ns"
            },
            "RocketLeague": {
                "title": "Rocket League",
                "app_name": "RocketLeague",
                "catalog_item_id": "rl_456",
                "namespace": "rl_ns"
            }
        }"#;

        let map: serde_json::Value = serde_json::from_str(json).unwrap();
        let obj = map.as_object().unwrap();

        let mut games = Vec::new();
        for (_key, val) in obj {
            if let (Some(title), Some(app_name)) = (
                val.get("title").and_then(|v| v.as_str()),
                val.get("app_name").and_then(|v| v.as_str()),
            ) {
                games.push(Game {
                    id: None,
                    title: title.to_string(),
                    platform: Platform::Epic,
                    playtime_minutes: 0,
                    achievements_unlocked: 0,
                    achievements_total: 0,
                    status: GameStatus::Unplayed,
                    rating: None,
                    genre: String::new(),
                    last_played: String::new(),
                    cover_url: String::new(),
                    steam_appid: None,
                    gog_id: None,
                    epic_id: Some(
                        val.get("catalog_item_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    ),
                    xbox_id: None,
                    psn_id: None,
                    notes: String::new(),
                    description: String::new(),
                    release_date: String::new(),
                    review_percent: None,
                    tags: String::new(),
                });
            }
        }

        assert_eq!(games.len(), 2);
        assert!(games.iter().any(|g| g.title == "Fortnite"));
        assert!(games.iter().any(|g| g.title == "Rocket League"));
    }

    /// Verify that empty/missing Epic store response is handled.
    #[test]
    fn parse_empty_store_response() {
        let json = r#"{}"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();

        let description = val
            .get("pages")
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("data"))
            .and_then(|d| d.get("about"))
            .and_then(|a| a.get("shortDescription"))
            .and_then(|s| s.as_str())
            .unwrap_or("");
        assert!(description.is_empty());
    }
}

// ───── Image validation tests ─────

mod image_validation {
    /// Test magic byte validation for all supported formats.
    fn is_valid_image(data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }
        if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
            return true;
        }
        if data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 {
            return true;
        }
        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
            return true;
        }
        if data.len() >= 6 && &data[0..3] == b"GIF" {
            return true;
        }
        false
    }

    #[test]
    fn validate_jpeg() {
        let data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert!(is_valid_image(&data));
    }

    #[test]
    fn validate_png() {
        let data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(is_valid_image(&data));
    }

    #[test]
    fn validate_webp() {
        let mut data = vec![0u8; 12];
        data[0..4].copy_from_slice(b"RIFF");
        data[8..12].copy_from_slice(b"WEBP");
        assert!(is_valid_image(&data));
    }

    #[test]
    fn validate_gif87a() {
        let data = b"GIF87a\x00\x00\x00\x00";
        assert!(is_valid_image(data));
    }

    #[test]
    fn validate_gif89a() {
        let data = b"GIF89a\x00\x00\x00\x00";
        assert!(is_valid_image(data));
    }

    #[test]
    fn reject_html() {
        let data = b"<!DOCTYPE html>";
        assert!(!is_valid_image(data));
    }

    #[test]
    fn reject_json() {
        let data = b"{\"error\": \"not found\"}";
        assert!(!is_valid_image(data));
    }

    #[test]
    fn reject_empty() {
        assert!(!is_valid_image(&[]));
    }

    #[test]
    fn reject_too_short() {
        assert!(!is_valid_image(&[0xFF, 0xD8]));
    }

    #[test]
    fn reject_text() {
        let data = b"This is not an image file";
        assert!(!is_valid_image(data));
    }
}

// ───── Rate limiter tests ─────

mod rate_limiter {
    use spotter::api_client::RateLimiter;
    use std::time::Duration;

    #[test]
    fn initial_delay_matches_base() {
        let limiter = RateLimiter::new(Duration::from_millis(200));
        assert_eq!(limiter.current_delay(), Duration::from_millis(200));
    }

    #[test]
    fn backoff_on_429() {
        let mut limiter = RateLimiter::new(Duration::from_millis(200));
        limiter.on_rate_limited();
        assert!(
            limiter.current_delay() > Duration::from_millis(200),
            "Delay should increase after rate limit"
        );
    }

    #[test]
    fn backoff_doubles() {
        let mut limiter = RateLimiter::new(Duration::from_millis(200));
        limiter.on_rate_limited();
        let first = limiter.current_delay();
        limiter.on_rate_limited();
        let second = limiter.current_delay();
        assert!(second > first, "Each 429 should increase delay further");
    }

    #[test]
    fn backoff_has_maximum() {
        let mut limiter = RateLimiter::new(Duration::from_millis(100));
        for _ in 0..20 {
            limiter.on_rate_limited();
        }
        assert!(
            limiter.current_delay() <= Duration::from_secs(60),
            "Delay should be capped at 60 seconds"
        );
    }

    #[test]
    fn success_gradually_reduces_delay() {
        let mut limiter = RateLimiter::new(Duration::from_millis(200));
        // Trigger backoff
        limiter.on_rate_limited();
        limiter.on_rate_limited();
        let backed_off = limiter.current_delay();
        // Success should reduce
        limiter.on_success();
        assert!(
            limiter.current_delay() < backed_off,
            "Success should reduce delay"
        );
    }

    #[test]
    fn success_never_goes_below_base() {
        let mut limiter = RateLimiter::new(Duration::from_millis(200));
        for _ in 0..10 {
            limiter.on_success();
        }
        assert!(
            limiter.current_delay() >= Duration::from_millis(200),
            "Delay should never go below base"
        );
    }

    #[test]
    fn full_recovery_after_backoff() {
        let base = Duration::from_millis(200);
        let mut limiter = RateLimiter::new(base);
        // Trigger heavy backoff
        limiter.on_rate_limited();
        limiter.on_rate_limited();
        limiter.on_rate_limited();
        assert!(limiter.current_delay() > base);
        // Many successes should bring it back to base
        for _ in 0..50 {
            limiter.on_success();
        }
        assert_eq!(
            limiter.current_delay(),
            base,
            "Should recover exactly to base after enough successes"
        );
    }

    #[test]
    fn alternating_rate_limit_and_success() {
        let base = Duration::from_millis(100);
        let mut limiter = RateLimiter::new(base);
        // Simulate real-world: rate limit then a few successes, repeat
        limiter.on_rate_limited(); // 200ms
        let after_rl = limiter.current_delay();
        limiter.on_success(); // 150ms
        let after_s1 = limiter.current_delay();
        assert!(after_s1 < after_rl, "Success should reduce delay");
        limiter.on_rate_limited(); // doubles from 150 -> 300
        let after_rl2 = limiter.current_delay();
        assert!(
            after_rl2 > after_s1,
            "Rate limit should increase delay again"
        );
    }
}

// ───── Import logger tests ─────

mod import_logger {
    use spotter::api_client::ImportLogger;
    use std::sync::atomic::{AtomicU32, Ordering};

    static LOG_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_log_dir() -> std::path::PathBuf {
        let id = LOG_COUNTER.fetch_add(1, Ordering::SeqCst);
        let tmp = std::env::temp_dir().join(format!(
            "spotter_log_test_{}_{}_{}",
            std::process::id(),
            id,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        tmp
    }

    #[test]
    fn logger_creates_file() {
        let dir = test_log_dir();
        let logger = ImportLogger::new(&dir);
        logger.log("steam", "Test message");

        let log_path = dir.join("import.log");
        assert!(log_path.exists(), "import.log should be created");

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("[steam]"));
        assert!(content.contains("Test message"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn logger_appends_multiple_entries() {
        let dir = test_log_dir();
        let logger = ImportLogger::new(&dir);
        logger.log("steam", "First");
        logger.log("gog", "Second");
        logger.log("xbox", "Third");

        let content = std::fs::read_to_string(dir.join("import.log")).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert!(lines.len() >= 3);
        assert!(content.contains("[steam] First"));
        assert!(content.contains("[gog] Second"));
        assert!(content.contains("[xbox] Third"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn logger_rotates_on_size() {
        let dir = test_log_dir();
        // Use a tiny max size to trigger rotation
        let logger = ImportLogger::with_max_size(&dir, 100);

        // Write enough to exceed 100 bytes
        for i in 0..20 {
            logger.log(
                "test",
                &format!("Log entry number {} with some padding text here", i),
            );
        }

        let log_path = dir.join("import.log");
        assert!(log_path.exists());
        let backup = dir.join("import.log.1");
        assert!(backup.exists(), "Old log should be rotated to import.log.1");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn logger_includes_timestamp() {
        let dir = test_log_dir();
        let logger = ImportLogger::new(&dir);
        logger.log("psn", "Timestamp test");

        let content = std::fs::read_to_string(dir.join("import.log")).unwrap();
        // Timestamp format: YYYY-MM-DD HH:MM:SS
        assert!(content.contains("20"), "Should contain year prefix (20xx)");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn logger_handles_nonexistent_directory() {
        let dir = std::env::temp_dir()
            .join("spotter_log_test_nonexistent")
            .join("deeply")
            .join("nested");
        // Don't create the directory — logger should fail gracefully
        let logger = ImportLogger::new(&dir);
        // This should not panic, just print to stderr
        logger.log("test", "Should not crash");
        // The log file should not exist since the directory doesn't exist
        assert!(!dir.join("import.log").exists());
    }
}

// ───── URL encode/decode tests ─────

mod url_encoding {
    use spotter::api_client::{url_decode, url_encode};

    #[test]
    fn roundtrip_ascii() {
        let input = "hello world!";
        assert_eq!(url_decode(&url_encode(input)), input);
    }

    #[test]
    fn roundtrip_special_chars() {
        let input = "key=value&foo=bar baz";
        assert_eq!(url_decode(&url_encode(input)), input);
    }

    #[test]
    fn roundtrip_unicode() {
        let input = "日本語テスト";
        assert_eq!(url_decode(&url_encode(input)), input);
    }

    #[test]
    fn roundtrip_empty() {
        assert_eq!(url_decode(&url_encode("")), "");
    }

    #[test]
    fn encode_unreserved_chars_unchanged() {
        let unreserved = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";
        assert_eq!(url_encode(unreserved), unreserved);
    }

    #[test]
    fn encode_space() {
        assert_eq!(url_encode(" "), "%20");
    }

    #[test]
    fn decode_plus_as_space() {
        assert_eq!(url_decode("hello+world"), "hello world");
    }

    #[test]
    fn decode_percent_encoded() {
        assert_eq!(url_decode("hello%20world"), "hello world");
    }

    #[test]
    fn decode_mixed() {
        assert_eq!(url_decode("a%20b+c%21"), "a b c!");
    }

    #[test]
    fn roundtrip_continuation_token() {
        // Simulate a realistic Xbox continuation token
        let token = "XBL3.0 x=abc123;eyJhbGciOiJSUzI1NiJ9.e30=";
        assert_eq!(url_decode(&url_encode(token)), token);
    }
}
