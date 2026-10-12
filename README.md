<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/icons/icon.png">
  <img src="assets/icons/icon.png" alt="Spotter" width="96">
</picture>

**A local game library and statistics tracker for players whose games are spread across Steam, GOG, Epic Games, Xbox and PlayStation.**

</div>

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Colony app](https://img.shields.io/badge/Colony-game-purple)](https://github.com/Project-Colony/Colony)
[![Platforms](https://img.shields.io/badge/platforms-linux%20%7C%20windows%20%7C%20macOS-lightgrey)](#installation)

Your games are spread across five stores, each with its own launcher, its own
playtime counter and its own idea of what you finished. Spotter imports them
into one SQLite database on your own machine, keeps your status, rating and
notes for every game across re-imports, and turns the result into playtime and
completion statistics. Nothing leaves your computer except the requests to the
stores you import from.

> **Status:** every importer is tested against recorded Steam, GOG, Epic,
> Xbox and PlayStation responses, not against live accounts. Only Steam's Web
> API and OpenXBL are documented APIs: the Steam store and achievement pages
> are scraped, and the GOG, Epic and PlayStation endpoints are the ones their
> own clients use, so a store can break an import without notice until the
> next release. Platform tokens live in the operating system's keyring since
> 0.3.0. Every
> release asset is signed with the Project Colony ed25519 key; the Windows
> executable has no Authenticode signature yet, pending SignPath Foundation
> approval. The Colony Preferences page, the French translation and the move
> to the standard Colony folders are covered by unit tests and still need a
> check on a real desktop install.

## Why Spotter

- **Every store, one list.** Steam, GOG, Epic Games, Xbox (through OpenXBL)
  and PlayStation Network imports land in the same library, and Nintendo games
  are added by hand. Each store's launcher only shows its own games.
- **Your data stays yours.** The library is a local SQLite file. There is no
  account, no sync server and no telemetry; exports are plain JSON and CSV.
- **Re-import without losing anything.** A second import updates playtime,
  achievements and store details, and never touches the status, rating or
  notes you set.
- **Tokens in the keyring.** API keys and login tokens go to Windows Credential
  Manager, the macOS Keychain or the Secret Service. Only a system without a
  usable keyring falls back to a file, and Spotter tells you when it does.

```
Steam ─┐
GOG ───┤
Epic ──┼──► import + merge ──► spotter.db ──► library, statistics, exports
Xbox ──┤                         (local)
PSN ───┘        Nintendo: added by hand
```

## What it does

- **Imports** from Steam (Web API key, Steam login), GOG (browser login), Epic
  Games (account login for the whole library, or a scan of the games installed
  through the Epic Games Launcher or Legendary), Xbox (an OpenXBL API key) and
  PlayStation Network (an NPSSO token).
- **Merges** each import into the existing library by store id, then by title,
  so importing again does not duplicate your games.
- **Achievements**: the full list with unlock dates for Steam and Xbox (with
  icons for Steam), trophy counts for PlayStation, the total for Epic Games.
- **Statistics**: totals, average rating, playtime over the last 30 days,
  this month against last month, status and platform distribution, and the
  most played games.
- **Library tools**: search, status and platform filters, six sort orders,
  favourites, ratings, notes, and manual entries.
- **Backups**: JSON and CSV export, JSON import, and an automatic JSON backup
  after every import.
- **Colony Preferences**: the shared Colony themes and accents, high contrast,
  a dyslexia-friendly font, text scaling, reduced motion, and an English or
  French interface.

## Installation

### Via Colony (recommended)

Search for **Spotter** in [Colony](https://github.com/Project-Colony/Colony)
and install it. Colony checks the release signature before it installs, and
updates arrive through the launcher.

### Direct binary download

Grab the asset for your platform from the
[latest release](https://github.com/Project-Colony/Spotter/releases/latest):

| Platform | Asset |
|---|---|
| Linux | `spotter-linux` |
| Windows | `spotter-windows.exe` |
| macOS (Apple Silicon) | `spotter-macos-arm` |
| macOS (Intel) | `spotter-macos-x86` |

```bash
chmod +x spotter-linux && ./spotter-linux
```

- **Linux** needs a Wayland or X11 session and libxkbcommon, which every
  desktop distribution ships.
- **Windows**: until the executable carries an Authenticode signature,
  SmartScreen may ask you to confirm with **More info** then **Run anyway**.
- **macOS**: the binary is not notarized, so remove the quarantine attribute
  first:
  `chmod +x spotter-macos-arm && xattr -d com.apple.quarantine spotter-macos-arm && ./spotter-macos-arm`
  (use `spotter-macos-x86` on Intel).

`spotter --version` prints the version and exits without opening a window.
To connect your stores, see
[docs/guide/platform-setup.md](docs/guide/platform-setup.md).

### Build from source

```bash
git clone https://github.com/Project-Colony/Spotter
cd Spotter
cargo build --release
```

Requires Rust 1.90 or newer. On Linux, install the windowing and keyring
development packages first; the list is in
[docs/internals/development.md](docs/internals/development.md).

## Documentation

Full documentation is in [docs/](docs/). Start at
[docs/README.md](docs/README.md).

## Code signing policy

Every release asset is signed by the Project Colony organisation in CI, never on
a developer machine. Next to each asset on the release page:

| File | What it is |
|---|---|
| `<asset>.sig` | an ed25519 signature over the asset, made with the organisation's release key |
| `<asset>.meta` | three lines binding the asset to its file name, its sha256 and the release version |
| `<asset>.meta.sig` | an ed25519 signature over the `.meta` |

The private key is an organisation secret, used only by the shared
[sign-and-publish workflow](https://github.com/Project-Colony/Project-Colony-Resources/blob/main/.github/workflows/sign-and-publish.yml)
in a job that builds nothing; the jobs that compile Spotter never see it.
Colony checks all three files before it installs or updates Spotter, and
refuses a release older than the one installed. To check a download yourself
with OpenSSL 3 (the same commands work for every asset):

```bash
cat > colony-release.pub <<'EOF'
-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEARNjg3Nn8H6/aBg1unwGjkUTcrdTxERNefVaqU8cFu0s=
-----END PUBLIC KEY-----
EOF
a=spotter-linux
openssl pkeyutl -verify -pubin -inkey colony-release.pub -rawin -in "$a" -sigfile "$a.sig"
openssl pkeyutl -verify -pubin -inkey colony-release.pub -rawin -in "$a.meta" -sigfile "$a.meta.sig"
cat "$a.meta"     # version=<tag>, asset=<file name>, sha256=<digest>
sha256sum "$a"    # the digest must equal the sha256 line
```

How releases are built, signed and published:
[design/releases.md](https://github.com/Project-Colony/Project-Colony-Resources/blob/main/design/releases.md#5-signing).

<!--
Add the following only once SignPath signs the Windows build, that is once
`signpath-project-slug` is set in .github/workflows/release.yml. Before that it
would promise a signature the .exe does not carry.

Windows releases are also Authenticode-signed. Free code signing provided by
[SignPath.io](https://about.signpath.io/), certificate by
[SignPath Foundation](https://signpath.org/).

- Committers and reviewers: [MotherSphere](https://github.com/MotherSphere)
- Approvers: [MotherSphere](https://github.com/MotherSphere)
-->

## Privacy

Spotter sends no telemetry, no analytics and no crash reports, has no update
check, and sends nothing to its developers. It contacts a store only when you
use a feature that needs it.

| Data | Stored or sent | Where, and why |
|---|---|---|
| Your library | stored | `library/spotter.db` in Spotter's data folder: games, playtime history, achievements, your profile name and connected account ids |
| Preferences | stored | `preferences/preferences.json` in Spotter's config folder |
| Credentials | stored | your Steam API key, OpenXBL key, PlayStation NPSSO token and the GOG and Epic login tokens, in the operating system's keyring (Windows Credential Manager, macOS Keychain, or the Secret Service on Linux such as GNOME Keyring or KWallet). Without a usable keyring they are written unencrypted to `credentials/tokens.json` in the config folder (owner-only on Linux and macOS), and the Profile page says so |
| Exports and backups | stored | `exports/` in the data folder: `spotter_export.json` and `spotter_export.csv` when you export, `spotter_auto_backup.json` after every import |
| Import log | stored | `logs/import.log` in the data folder (rotated to `import.log.1` at 5 MB): import steps, game counts and errors, plus your Xbox user id |
| Covers and achievement icons | stored | `covers/` and `achievement_icons/` in Spotter's cache folder; safe to delete, they are downloaded again |
| Steam import | sent | when you import: your Web API key and Steam ID to `api.steampowered.com` (owned games), your Steam ID to `steamcommunity.com` (achievements) and game ids to `store.steampowered.com` (store details) |
| Steam login | sent | when you click Login: Steam's sign-in page opens in your browser, the answer comes back to a listener on `127.0.0.1` (ports 29876 to 29880) of your own computer, and Spotter asks `steamcommunity.com` to confirm it |
| GOG import | sent | when you log in: the code you paste goes to `auth.gog.com` in exchange for a token; when you import, that token goes to `embed.gog.com` (your games) and game ids to `api.gog.com` (store details) |
| Epic Games import | sent | when you log in: the code you paste goes to `account-public-service-prod.ol.epicgames.com`; when you import, the token and your account id go to Epic's library, entitlement, launcher and catalog services (`*.epicgames.com`), and game ids to `store.epicgames.com` and `store-content-ipv4.ak.epicgames.com`. The local scan reads installed games from disk, then asks the store for their details |
| Xbox import | sent | your OpenXBL API key to `xbl.io` (titles, achievements, store details) |
| PlayStation import | sent | your NPSSO token to `ca.account.sony.com` in exchange for an access token, then that token to `m.np.playstation.com` (trophies and played games) |
| Token renewal | sent | at startup if GOG or Epic Games is connected, and when an import finds the login expired: the saved refresh token to `auth.gog.com` or `account-public-service-prod.ol.epicgames.com` |
| Images | sent | requests for the cover and achievement image addresses stored with each game, which come from those stores (or from a JSON backup you import): covers after an import unless automatic cover downloads are off, Steam achievement icons during a Steam import and when you open a Steam game |

The config, data and cache folders are `Colony/Spotter/` under your system's
config, data and cache directories; the full table per operating system is in
[docs/internals/data-model.md](docs/internals/data-model.md#where-the-data-lives).

Spotter connects to no server except the ones in this table, and only for the
action named there. Links such as a game's store page open in your browser;
Spotter itself sends nothing for them. Each store's own privacy policy covers
what it does with those requests:
[Steam](https://store.steampowered.com/privacy_agreement/),
[GOG](https://support.gog.com/hc/articles/212632109-Privacy-Policy?product=gog),
[Epic Games](https://www.epicgames.com/site/privacypolicy),
[OpenXBL](https://xbl.io/agreement),
[PlayStation](https://www.playstation.com/legal/privacy-policy/).

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

The interface fonts come with
[colony-ui](https://github.com/Project-Colony/Project-Colony-Resources/tree/main/crates/colony-ui):
JetBrains Mono Nerd Font, OpenDyslexic and Font Awesome 6 Free, each under the
SIL Open Font License 1.1, with their licence files in
[colony-ui's assets/fonts](https://github.com/Project-Colony/Project-Colony-Resources/tree/main/crates/colony-ui/assets/fonts).
