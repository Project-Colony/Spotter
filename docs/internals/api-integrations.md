# API Integrations

Spotter imports games from 5 platforms. Each has a distinct authentication model and data source.

## Steam

**Module:** `steam.rs`, `steam_auth.rs`

### Authentication

1. **OpenID Login** — User clicks "Login with Steam", a local HTTP server starts on `localhost:29876`. Steam redirects back with the user's 64-bit Steam ID.
2. **API Key** — User provides their Steam Web API key from [steamcommunity.com/dev/apikey](https://steamcommunity.com/dev/apikey).

### Endpoints

| Endpoint | Purpose |
|----------|---------|
| `api.steampowered.com/IPlayerService/GetOwnedGames/v1/` | Owned games list (JSON) |
| `store.steampowered.com/app/{appid}/` | Game metadata (HTML scraping) |
| `steamcommunity.com/.../GetPlayerAchievements/` | Achievement data (HTML scraping) |

### Import Flow

1. Fetch owned games via Web API (returns `appid`, `name`, `playtime_forever`)
2. Scrape store pages for metadata (genre, release date, cover, review %, description)
3. Scrape achievement pages per game
4. Concurrent scraping: 5 threads, 100ms delay between requests

### Scraped Data

From store HTML pages:
- **Genre:** `<a href="/genre/...">text</a>`
- **Release date:** `<div class="release_date">...<div class="date">DATE</div>`
- **Cover:** `<meta property="og:image" content="...">`
- **Review %:** Score extracted from review section
- **Tags:** From tag elements on the store page

## GOG

**Module:** `gog.rs`, `gog_auth.rs`

### Authentication

OAuth 2.0 using well-known Galaxy client credentials (same for all GOG Galaxy clients):
- Client ID: `46899977096215655`
- Client secret: `9d85c43b1482497dbbce61f6e4aa173a433796eeae2571571f...`
- Redirect: `https://embed.gog.com/on_login_success`

Token refresh is automatic on 401 responses.

### Endpoints

| Endpoint | Purpose |
|----------|---------|
| `embed.gog.com/user/data/games` | Owned games list (auth required) |
| `api.gog.com/products/{id}` | Game metadata (public) |

### Import Flow

1. User logs in via OAuth (browser flow)
2. Fetch owned game IDs from user endpoint
3. Fetch metadata per game from public API
4. Data: title, cover (`logo2x`), description (`lead`), release date

**Note:** GOG does not expose achievement data via API.

## Epic Games

**Module:** `epic.rs`

### Authentication

None required. Reads local manifest files directly.

### Data Source

Scans `.item` JSON manifests from the Epic Games Launcher:

| OS | Path |
|----|------|
| Windows | `C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests\` |
| macOS | `/Users/Shared/Epic Games/Launcher/Data/Manifests/` |
| Linux (Heroic) | `~/.config/heroic/store_cache/` |
| Linux (Legendary) | `~/.config/legendary/installed.json` |
| Wine/Proton | `~/.wine/drive_c/ProgramData/Epic/...` |

### Extracted Fields

- `DisplayName` → title
- `CatalogItemId` → epic_id
- `AppName` → internal reference
- `AppCategories` → used to filter non-game content

## Xbox Live

**Module:** `xbox.rs`

### Authentication

Uses [OpenXBL](https://xbl.io) as a proxy API:
- Header: `X-Authorization: {api_key}`

### Endpoint

| Endpoint | Purpose |
|----------|---------|
| `xbl.io/api/v2/player/titleHistory` | Title history with stats |

### Import Flow

1. Fetch title history
2. Filter by device type: `XboxOne`, `XboxSeries`, `Xbox360`, `PC`, `Win32`
3. Extract: title, `displayImage` (cover), `minutesPlayed`, `lastTimePlayed`, achievements (current/total)

### Filtering

The `is_game()` heuristic filters out apps by checking for Xbox-specific device categories.

## PlayStation Network

**Module:** `playstation.rs`

### Authentication

Multi-step OAuth flow:
1. User provides **NPSSO token** (extracted from browser cookies at `store.playstation.com`)
2. Exchange NPSSO → authorization code via `ca.account.sony.com/api/v1/oauth/authorize`
3. Exchange code → access token via `ca.account.sony.com/api/v1/oauth/token`

Uses hardcoded PSN Android app credentials for the OAuth client.

### Endpoints

| Endpoint | Purpose |
|----------|---------|
| `ca.account.sony.com/api/v1/oauth/authorize` | Authorization code |
| `ca.account.sony.com/api/v1/oauth/token` | Access token exchange |
| `m.np.playstation.com/api/graphql/v1/op?operationName=RetrieveTitleList` | Game library (GraphQL) |
| `m.np.playstation.com/api/trophy/v2/trophyTitles` | Trophy counts |

### Import Flow

1. Exchange NPSSO for OAuth token
2. Fetch game library via GraphQL
3. Fetch trophy counts (bronze/silver/gold/platinum → converted to unlocked/total)
4. Extract: title, playtime, last played, trophy data

**Note:** No individual trophy details are available, only counts.

## Nintendo

No API integration. Games are added manually through the "Add Game" form with `Platform::Nintendo` selected.

## HTTP Client

All API calls go through `api_client.rs`:

```
Connect timeout: 10s
Read timeout:    15s (API) / 30s (downloads)
Retry strategy:  Exponential backoff (500ms × 2^attempt)
                 4xx errors: fail immediately (no retry)
                 5xx/network: retry up to N times
```
