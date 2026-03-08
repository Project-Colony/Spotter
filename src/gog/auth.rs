use serde::Deserialize;

// Well-known GOG Galaxy OAuth credentials (used by all open-source GOG clients).
const GOG_CLIENT_ID: &str = "46899977096215655";
const GOG_CLIENT_SECRET: &str = "9d85c43b1482497dbbce61f6e4aa173a433796eeae2ca8c5f6129f2dc4de46d9";
const GOG_AUTH_URL: &str = "https://auth.gog.com/auth";
const GOG_TOKEN_URL: &str = "https://auth.gog.com/token";
const GOG_REDIRECT_URI: &str = "https://embed.gog.com/on_login_success?origin=client";

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: String,
}

/// Build the GOG OAuth authorization URL that opens in the user's browser.
pub fn build_auth_url() -> String {
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&layout=client2",
        GOG_AUTH_URL,
        GOG_CLIENT_ID,
        url_encode(GOG_REDIRECT_URI),
    )
}

/// Open the GOG login page in the default browser.
pub fn open_login_page() -> Result<(), String> {
    let url = build_auth_url();
    open_browser(&url)
}

/// Extract the authorization code from user input.
/// Accepts either the full redirect URL or just the raw code.
pub fn extract_code(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    // If input contains "code=", extract the value
    if let Some(idx) = trimmed.find("code=") {
        let rest = &trimmed[idx + 5..];
        // Code ends at '&' or end of string
        let code = rest.split('&').next().unwrap_or(rest);
        let code = code.trim();
        if !code.is_empty() {
            return Some(code.to_string());
        }
    }
    // Otherwise treat the whole input as the code
    Some(trimmed.to_string())
}

/// Exchange an authorization code for access + refresh tokens.
/// Returns (access_token, refresh_token).
pub fn exchange_code(code: &str) -> Result<(String, String), String> {
    let url = format!(
        "{}?client_id={}&client_secret={}&grant_type=authorization_code&code={}&redirect_uri={}",
        GOG_TOKEN_URL,
        GOG_CLIENT_ID,
        GOG_CLIENT_SECRET,
        url_encode(code),
        url_encode(GOG_REDIRECT_URI),
    );

    let agent = crate::api_client::api_agent();

    let response = agent.get(&url).call().map_err(|e| match e {
        ureq::Error::StatusCode(code) => {
            format!("GOG returned error {}", code)
        }
        other => format!("Token exchange failed: {}", other),
    })?;

    let resp: TokenResponse = response
        .into_body()
        .read_json()
        .map_err(|e| format!("Token parse error: {}", e))?;

    if resp.access_token.is_empty() {
        return Err("GOG returned an empty access token".into());
    }

    Ok((resp.access_token, resp.refresh_token))
}

/// Refresh an expired access token using a refresh token.
/// Returns (new_access_token, new_refresh_token).
pub fn refresh_token(refresh: &str) -> Result<(String, String), String> {
    if refresh.is_empty() {
        return Err("No refresh token available. Please login again.".into());
    }

    let url = format!(
        "{}?client_id={}&client_secret={}&grant_type=refresh_token&refresh_token={}",
        GOG_TOKEN_URL, GOG_CLIENT_ID, GOG_CLIENT_SECRET, refresh,
    );

    let agent = crate::api_client::api_agent();

    let response = agent.get(&url).call().map_err(|e| match e {
        ureq::Error::StatusCode(code) => {
            format!("Token refresh error {}", code)
        }
        other => format!("Token refresh failed: {}", other),
    })?;

    let resp: TokenResponse = response
        .into_body()
        .read_json()
        .map_err(|e| format!("Token refresh parse error: {}", e))?;

    if resp.access_token.is_empty() {
        return Err("GOG returned an empty access token on refresh".into());
    }

    Ok((resp.access_token, resp.refresh_token))
}

fn open_browser(url: &str) -> Result<(), String> {
    crate::api_client::open_browser(url)
}

fn url_encode(s: &str) -> String {
    crate::api_client::url_encode(s)
}
