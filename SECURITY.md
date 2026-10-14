# Security Policy

Spotter holds the API keys and login tokens of up to five game stores, signs in
to them, and parses the JSON and HTML those stores send back. A flaw in any of
that is a security bug, and reports are welcome.

## Supported versions

Only the **latest release** receives security fixes. Spotter ships as a single
binary, through Colony and the release page, so the fix for a vulnerability is
the next release, not a patch to an older one.

| Version | Supported |
| --- | --- |
| latest release | yes |
| anything older | no |

## Reporting a vulnerability

Please report vulnerabilities **privately** via
[GitHub Security Advisories](https://github.com/Project-Colony/Spotter/security/advisories/new)
("Report a vulnerability"). Do not open a public issue for exploitable bugs,
and do not attach real API keys, tokens or an unredacted `spotter.db` to a
report.

Include the version (`spotter --version`), the platform, and the steps or the
file that reproduce it. The report stays private until a fixed release is out,
and you are credited in the release notes if you want to be.

## Scope

Reports of particular interest:

- **Credentials**: a store token or API key written anywhere other than the
  operating system's keyring while a keyring is usable, a fallback
  `credentials/tokens.json` readable by other users, or a secret that reaches
  a log, an export, a URL or the wrong host.
- **The Steam login listener**: Spotter accepts the OpenID answer on
  `127.0.0.1` (ports 29876 to 29880) and confirms it with `steamcommunity.com`
  before trusting the Steam ID. A way to make it accept a forged answer, or to
  reach it from another machine, is in scope.
- **Untrusted input**: crashes or worse from store responses, scraped HTML,
  downloaded images, Epic Games Launcher, Heroic or Legendary manifests, or a
  crafted JSON backup imported from the Profile page, including file names
  derived from game titles that escape the cache folder.
- **Release signing**: anything that gets an asset accepted by Colony without a
  valid signature from the Project Colony release key. See
  [Code signing policy](README.md#code-signing-policy).

Out of scope: what the stores themselves do with the requests Spotter sends
them, and the well-known public OAuth client ids of the GOG Galaxy, Epic Games
Launcher and PlayStation apps, which Spotter uses the same way every
open-source client does.
