use serde::Deserialize;

/// Epic Games OAuth credentials — well-known launcher client used by Legendary/Heroic.
/// These are the same credentials every open-source Epic client uses.
const EPIC_CLIENT_ID: &str = "34a02cf8f4414e29b15921876da36f9a";
const EPIC_CLIENT_SECRET: &str = "daafbccc737745039dffe53d94fc76cf";

const EPIC_OAUTH_HOST: &str = "https://account-public-service-prod.ol.epicgames.com";
/// Login URL that goes through Epic's login page first, then redirects to the
/// API endpoint that returns the JSON with authorizationCode.
/// Without the /id/login wrapper, the redirect endpoint returns null codes.
const EPIC_LOGIN_URL: &str =
    "https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect%3FclientId%3D34a02cf8f4414e29b15921876da36f9a%26responseType%3Dcode";

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: String,
    #[serde(default)]
    account_id: String,
    #[serde(alias = "displayName", default)]
    display_name: String,
}

/// Result of a successful Epic login: (access_token, refresh_token, account_id, display_name).
pub type EpicLoginResult = (String, String, String, String);

/// Build the authorization URL that opens in the user's browser.
/// The user logs in on epicgames.com and receives a JSON response containing
/// an authorization code, which they paste back into Spotter.
pub fn build_auth_url() -> String {
    EPIC_LOGIN_URL.to_string()
}

/// Open the Epic login page in the default browser.
pub fn open_login_page() -> Result<(), String> {
    crate::api_client::open_browser(&build_auth_url())
}

/// Extract the authorization code from user input.
/// Accepts either:
///   - Raw JSON like `{"redirectUrl":"...","authorizationCode":"abc123...",...}`
///   - Just the code string directly
///   - A URL containing `?code=abc123`
pub fn extract_code(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try to parse as JSON (Epic's redirect returns JSON with authorizationCode)
    if trimmed.starts_with('{') {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(code) = val.get("authorizationCode").and_then(|v| v.as_str()) {
                if !code.is_empty() {
                    return Some(code.to_string());
                }
            }
        }
    }

    // Try URL with ?code= parameter
    if let Some(idx) = trimmed.find("code=") {
        let rest = &trimmed[idx + 5..];
        let code = rest.split('&').next().unwrap_or(rest).trim();
        if !code.is_empty() {
            return Some(code.to_string());
        }
    }

    // Otherwise treat the whole input as the code (32-char hex string)
    let cleaned = trimmed.trim_matches('"');
    if !cleaned.is_empty() && cleaned.len() <= 64 {
        return Some(cleaned.to_string());
    }

    Some(trimmed.to_string())
}

/// Exchange an authorization code for access + refresh tokens.
/// Returns (access_token, refresh_token, account_id, display_name).
pub fn exchange_code(code: &str) -> Result<EpicLoginResult, String> {
    let url = format!("{}/account/api/oauth/token", EPIC_OAUTH_HOST);

    let agent = crate::api_client::api_agent();

    let body = format!(
        "grant_type=authorization_code&code={}&token_type=eg1",
        crate::api_client::url_encode(code),
    );

    let response = agent
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", &format!("Basic {}", basic_auth()))
        .send(&body)
        .map_err(|e| match e {
            ureq::Error::StatusCode(code) => {
                format!("Epic returned error {}", code)
            }
            other => format!("Token exchange failed: {}", other),
        })?;

    let resp: TokenResponse = response
        .into_body()
        .read_json()
        .map_err(|e| format!("Token parse error: {}", e))?;

    if resp.access_token.is_empty() {
        return Err("Epic returned an empty access token".into());
    }

    eprintln!(
        "[epic_auth] Login successful: account_id={}, display_name={}",
        resp.account_id, resp.display_name
    );

    Ok((
        resp.access_token,
        resp.refresh_token,
        resp.account_id,
        resp.display_name,
    ))
}

/// Refresh an expired access token using a refresh token.
/// Returns (access_token, refresh_token, account_id, display_name).
pub fn refresh_token(refresh: &str) -> Result<EpicLoginResult, String> {
    if refresh.is_empty() {
        return Err("No refresh token available. Please login again.".into());
    }

    let url = format!("{}/account/api/oauth/token", EPIC_OAUTH_HOST);
    let agent = crate::api_client::api_agent();

    let body = format!(
        "grant_type=refresh_token&refresh_token={}&token_type=eg1",
        crate::api_client::url_encode(refresh),
    );

    let response = agent
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", &format!("Basic {}", basic_auth()))
        .send(&body)
        .map_err(|e| match e {
            ureq::Error::StatusCode(code) => {
                format!("Epic token refresh error {}", code)
            }
            other => format!("Epic token refresh failed: {}", other),
        })?;

    let resp: TokenResponse = response
        .into_body()
        .read_json()
        .map_err(|e| format!("Token refresh parse error: {}", e))?;

    if resp.access_token.is_empty() {
        return Err("Epic returned an empty access token on refresh".into());
    }

    eprintln!(
        "[epic_auth] Token refreshed for account_id={}",
        resp.account_id
    );

    Ok((
        resp.access_token,
        resp.refresh_token,
        resp.account_id,
        resp.display_name,
    ))
}

/// Verify that an access token is still valid.
/// Returns the account_id if valid, or an error if expired/invalid.
#[allow(dead_code)]
pub fn verify_token(token: &str) -> Result<String, String> {
    let url = format!("{}/account/api/oauth/verify", EPIC_OAUTH_HOST);
    let agent = crate::api_client::api_agent();

    let body = crate::api_client::http_get_bearer(&agent, &url, token, 0)?;
    let val: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Verify parse error: {}", e))?;

    val.get("account_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Token verification failed: no account_id".into())
}

/// Base64-encode the client_id:client_secret pair for HTTP Basic auth.
fn basic_auth() -> String {
    use std::io::Write;
    let raw = format!("{}:{}", EPIC_CLIENT_ID, EPIC_CLIENT_SECRET);
    let mut buf = Vec::with_capacity(raw.len() * 4 / 3 + 4);
    {
        let mut encoder = Base64Encoder::new(&mut buf);
        encoder.write_all(raw.as_bytes()).unwrap();
        encoder.finish();
    }
    String::from_utf8(buf).unwrap()
}

/// Minimal Base64 encoder (avoids adding a dependency just for this).
struct Base64Encoder<'a> {
    out: &'a mut Vec<u8>,
    buf: [u8; 3],
    pos: usize,
}

const B64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

impl<'a> Base64Encoder<'a> {
    fn new(out: &'a mut Vec<u8>) -> Self {
        Self {
            out,
            buf: [0; 3],
            pos: 0,
        }
    }

    fn flush_triplet(&mut self) {
        let b = &self.buf;
        self.out.push(B64_CHARS[(b[0] >> 2) as usize]);
        self.out
            .push(B64_CHARS[((b[0] & 0x03) << 4 | b[1] >> 4) as usize]);
        if self.pos > 1 {
            self.out
                .push(B64_CHARS[((b[1] & 0x0f) << 2 | b[2] >> 6) as usize]);
        } else {
            self.out.push(b'=');
        }
        if self.pos > 2 {
            self.out.push(B64_CHARS[(b[2] & 0x3f) as usize]);
        } else {
            self.out.push(b'=');
        }
        self.buf = [0; 3];
        self.pos = 0;
    }

    fn finish(mut self) {
        if self.pos > 0 {
            self.flush_triplet();
        }
    }
}

impl<'a> std::io::Write for Base64Encoder<'a> {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        for &byte in data {
            self.buf[self.pos] = byte;
            self.pos += 1;
            if self.pos == 3 {
                self.flush_triplet();
            }
        }
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
