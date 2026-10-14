# API integrations

Spotter imports from five stores. Each importer is a blocking function that
runs on a worker thread and returns `Vec<Game>`; the merge into the library is
described in [architecture.md](architecture.md#import-merge). Only Steam's Web
API and OpenXBL are documented public APIs: the GOG, Epic and PlayStation
endpoints below are the ones their own clients use, and can change without
notice. The recorded responses in `tests/integration_tests.rs` pin down what
each parser expects.

Every importer appends its steps and counts to `import.log` through
`api_client::ImportLogger` (see
[data-model.md](data-model.md#where-the-data-lives)).

## HTTP client

All requests go through `ureq` agents built in `api_client.rs`:

| | |
|---|---|
| connect timeout | 10 s |
| read timeout | 15 s for API calls, 30 s for downloads |
| `http_get_retry` | exponential backoff, 500 ms x 2^attempt; a 4xx fails at once, except 429 |
| `RateLimiter` | doubles its delay on each 429, up to 60 s, and eases back 25% per success |
| images | at most 10 MB, and the bytes must be a JPEG, PNG, WebP or GIF image |

## Steam

`steam/mod.rs` (import) and `steam/auth.rs` (login).

**Login.** Steam OpenID 2.0. `steam_login` binds the first free port among
29876 to 29880 on `127.0.0.1`, opens `steamcommunity.com/openid/login` in the
browser with `http://localhost:<port>/callback` as the return address, and
waits up to two minutes. The Steam ID is taken from `openid.claimed_id` only
after `steamcommunity.com` confirms the answer (`check_authentication`), so a
forged callback is rejected. The Web API key is typed by the user.

| Endpoint | Used for |
|---|---|
| `api.steampowered.com/IPlayerService/GetOwnedGames/v0001/` | owned games, names and playtime (key + Steam ID) |
| `store.steampowered.com/app/<appid>/` | genre, release date, description, tags, review score (HTML) |
| `steamcommunity.com/stats/<appid>/achievements` | the game's achievement list with icons (HTML) |
| `steamcommunity.com/profiles/<steamid>/stats/appid/<appid>/achievements/` | the player's unlocks and dates (HTML, public profile only) |
| `shared.fastly.steamstatic.com/store_item_assets/steam/apps/<appid>/header.jpg` | cover image |

Five worker threads share the game list, pause 500 ms between games, and retry
a store page twice. Games that already have store details or complete
achievements in the library are skipped, so a second import is much faster.
Achievement icons are downloaded during the import.

## GOG

`gog/mod.rs` (import) and `gog/auth.rs` (login).

**Login.** OAuth 2.0 with the public GOG Galaxy client id. The browser opens
`auth.gog.com/auth`; after sign-in GOG redirects to
`embed.gog.com/on_login_success`, and the user pastes that address (or its
`code`). `auth.gog.com/token` exchanges the code for an access and a refresh
token, with the parameters in a form body so they stay out of URLs. The refresh
token renews the access token at startup and when an import fails.

| Endpoint | Used for |
|---|---|
| `embed.gog.com/account/getFilteredProducts?mediaType=1&page=<n>` | owned games, all pages (token) |
| `api.gog.com/products/<id>?expand=description` | description, release date, genres and cover (public) |

GOG's API has no achievement data for Spotter.

## Epic Games

`epic/`. Two independent imports, picked by whether the user is logged in.

**Local scan** (`local.rs`), no login. Reads the Epic Games Launcher's `.item`
manifests and Legendary's `installed.json` from:

| System | Folder |
|---|---|
| Windows | `%PROGRAMDATA%\Epic\EpicGamesLauncher\Data\Manifests\` |
| macOS | `/Users/Shared/Epic Games/Launcher/Data/Manifests/` |
| Linux | `~/.config/legendary/` (`installed.json`), `~/.config/heroic/store_cache/`, and the launcher folder inside `~/.wine/drive_c/ProgramData/` |

Only entries in the `games` category are kept. Store details then come from
`store-content-ipv4.ak.epicgames.com`.

**Account import** (`auth.rs`, `online.rs`, `api.rs`, `graphql.rs`). OAuth with
the public Epic Games Launcher client id, the same one Legendary and Heroic
use. The browser opens `www.epicgames.com/id/login`, which ends on a JSON page
holding `authorizationCode`; the user pastes it, and
`account-public-service-prod.ol.epicgames.com` exchanges it for an access
token, a refresh token, the account id and the display name.

| Endpoint | Used for |
|---|---|
| `library-service.live.use1a.on.epicgames.com/library/api/public/items` | owned items, paginated |
| `entitlement-public-service-prod08.ol.epicgames.com/entitlement/api/account/<id>/entitlements` | items missing from the library service |
| `launcher-public-service-prod06.ol.epicgames.com/launcher/api/public/assets/Windows` | the launcher's own asset list |
| `catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/namespace/<ns>/bulk/items` | titles, categories, images |
| `store-content-ipv4.ak.epicgames.com/api/en-US/content/products/<slug>` | description, cover, tags, release date |
| `store.epicgames.com/graphql` | slugs, titles of unresolved items, ratings, achievement totals, fallback metadata |

The three item lists are merged by namespace and catalog item id, DLC and
non-game items are filtered out, and items whose title cannot be resolved are
dropped. Epic does not grant the launcher client the scope for player
achievements, so only the total count is known. A 401 triggers one token
refresh and a retry.

## Xbox

`xbox.rs`, through [OpenXBL](https://xbl.io), a third-party proxy for Xbox Live.
Every request carries the user's key in the `X-Authorization` header.

| Endpoint | Used for |
|---|---|
| `xbl.io/api/v2/account` | the user's XUID |
| `xbl.io/api/v2/player/titleHistory` | played titles with cover, playtime, last played and achievement counts, up to 50 pages |
| `xbl.io/api/v2/achievements/player/<xuid>/<titleid>` | Xbox One, Series and PC achievements |
| `xbl.io/api/v2/achievements/x360/<xuid>/title/<titleid>` | Xbox 360 achievements |
| `xbl.io/api/v2/marketplace/details` | descriptions and release dates for a batch of titles |

`is_game` keeps titles played on `XboxOne`, `XboxSeries`, `Xbox360`, `PC` or
`Win32`. Achievements are saved per title under `platform_id = xbox:<titleid>`.

## PlayStation

`playstation.rs`.

**Login.** The user pastes an NPSSO token from
`ca.account.sony.com/api/v1/ssocookie`. Spotter exchanges it with the public
PlayStation app client id: `ca.account.sony.com/api/authz/v3/oauth/authorize`
returns an authorization code, and `.../oauth/token` an access token. The
access token is not stored; each import exchanges the NPSSO again.

| Endpoint | Used for |
|---|---|
| `m.np.playstation.com/api/trophy/v1/users/me/trophyTitles` | every game with trophies and its earned and total trophy counts, paginated |
| `m.np.playstation.com/api/gamelist/v2/users/me/titles?categories=ps4_game,ps5_native_game` | playtime, last played and covers for PS4 and PS5 games |

The trophy list is the game list; the played-games list only enriches it.
Individual trophies are not read.

## Nintendo

No API. Games are added by hand with **Add Game**.
