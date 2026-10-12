# Platform setup

Everything below happens on the **Import** screen: each store has a card with
its steps, a status pill (Setup required, Ready, Importing) and an import
button. What you enter there is kept in the operating system's keyring, see
[Privacy](../../README.md#privacy).

| Store | What Spotter needs | Where to get it |
|---|---|---|
| Steam | your Steam ID and a Steam Web API key | the **Login** button, then [steamcommunity.com/dev/apikey](https://steamcommunity.com/dev/apikey) |
| GOG | a GOG login | the **Login** button, then paste the address GOG redirects you to |
| Epic Games | nothing, or an Epic login for the whole library | the **Login** button, then paste the answer Epic shows |
| Xbox | an OpenXBL API key | [xbl.io](https://xbl.io) (free account) |
| PlayStation | an NPSSO token | [ca.account.sony.com/api/v1/ssocookie](https://ca.account.sony.com/api/v1/ssocookie) while signed in to PlayStation |
| Nintendo | nothing | no import: add games with **Add Game** |

## Steam

1. Click **Login**. Steam's sign-in page opens in your browser. Sign in and
   approve; Steam sends you back to a page served by Spotter on `127.0.0.1`,
   and the card shows your Steam ID. Spotter waits two minutes for the answer.
2. Create a Web API key at
   [steamcommunity.com/dev/apikey](https://steamcommunity.com/dev/apikey) (any
   domain name works) and paste it into **Steam Web API key**.
3. Click **Import Steam**.

Achievements are read from your Steam community profile, so set **Game
details** to Public in Steam's privacy settings, or the import finds your games
but no achievements. The import lists your owned games, then reads each game's
store page and your achievements, five games at a time.

## GOG

1. Click **Login**. GOG's sign-in page opens in your browser.
2. After signing in, the browser lands on a mostly blank
   `embed.gog.com/on_login_success?...` page. Copy that whole address from the
   address bar, paste it into the card, and click **Complete**. Pasting only
   the `code=` value works too.
3. Click **Import GOG**.

GOG gives Spotter a token that expires; Spotter renews it at startup, and
again when an import finds it expired, so you log in only once. GOG has no achievement data in the
API Spotter uses.

## Epic Games

The login is optional, and it decides what the import button does:

- **Without a login**, the import scans this computer for installed games: the
  Epic Games Launcher's manifests (on Windows, on macOS, or in the default Wine
  prefix `~/.wine` on Linux) and Legendary's `installed.json`. The folders are
  listed in
  [api-integrations.md](../internals/api-integrations.md#epic-games).
- **With a login**, it imports every game the account owns. Click **Login**,
  sign in on the page that opens, and copy what Epic shows afterwards: a short
  JSON text that contains `authorizationCode`. Paste it into the card and click
  **Complete**. The code alone, or an address with `?code=`, works too.

Then click the import button. Spotter renews the Epic login at startup, like
GOG's.

## Xbox

Spotter reads Xbox data through [OpenXBL](https://xbl.io), a third-party
service, not through Microsoft directly.

1. Sign in at [xbl.io](https://xbl.io) with your Microsoft account and copy
   your API key.
2. Paste it into the card. The **Gamertag** field appears; it is only a label
   for your profile, the import does not need it.
3. Click **Import Xbox**.

The import reads your title history (Xbox One, Xbox Series, Xbox 360 and PC
games), each game's achievements, and store details. OpenXBL's free plan has an
hourly request limit, which a large library can reach.

## PlayStation

1. Sign in at [playstation.com](https://www.playstation.com) in your browser.
2. In the same browser, open
   [ca.account.sony.com/api/v1/ssocookie](https://ca.account.sony.com/api/v1/ssocookie).
   It shows `{"npsso":"..."}`. Copy the 64-character value between the quotes.
3. Paste it into **NPSSO Token** and click **Import PlayStation**.

The NPSSO token is a login cookie for your Sony account: treat it like a
password. It expires after about two months, and signing out of
playstation.com ends it sooner; paste a new one when the import says it is no
longer valid. Spotter imports every game in your trophy list with its trophy
counts, and adds playtime and covers for PS4 and PS5 games. It does not read
individual trophies.

## Nintendo

Nintendo offers no library API. Add your games with **Add Game** in the
sidebar.

## After an import

- Importing again updates playtime, achievements and store details, adds new
  games, and keeps your status, rating and notes.
- Covers download after each import unless **Auto-download covers on
  import** is off in Preferences.
- Every import writes `spotter_auto_backup.json` and a line per step to
  `import.log`; their folders are listed in
  [data-model.md](../internals/data-model.md#where-the-data-lives).
