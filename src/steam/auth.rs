use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

const REDIRECT_PORTS: &[u16] = &[29876, 29877, 29878, 29879, 29880];
const STEAM_OPENID_URL: &str = "https://steamcommunity.com/openid/login";

fn url_encode(s: &str) -> String {
    crate::api_client::url_encode(s)
}

fn build_login_url(port: u16) -> String {
    let return_to = format!("http://localhost:{}/callback", port);
    let realm = format!("http://localhost:{}", port);

    let params = [
        ("openid.ns", "http://specs.openid.net/auth/2.0"),
        ("openid.mode", "checkid_setup"),
        ("openid.return_to", &return_to),
        ("openid.realm", &realm),
        (
            "openid.identity",
            "http://specs.openid.net/auth/2.0/identifier_select",
        ),
        (
            "openid.claimed_id",
            "http://specs.openid.net/auth/2.0/identifier_select",
        ),
    ];

    let query: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, url_encode(v)))
        .collect();

    format!("{}?{}", STEAM_OPENID_URL, query.join("&"))
}

fn open_browser(url: &str) -> Result<(), String> {
    crate::api_client::open_browser(url)
}

fn extract_steam_id(query: &str) -> Option<String> {
    // Look for openid.claimed_id=https://steamcommunity.com/openid/id/STEAMID64
    for param in query.split('&') {
        let parts: Vec<&str> = param.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0];
            let value = parts[1];
            // URL-decode the key for comparison
            let decoded_key = url_decode(key);
            if decoded_key == "openid.claimed_id" {
                let decoded_value = url_decode(value);
                // Extract the Steam ID from the URL
                if let Some(id) =
                    decoded_value.strip_prefix("https://steamcommunity.com/openid/id/")
                {
                    return Some(id.to_string());
                }
            }
        }
    }
    None
}

fn url_decode(s: &str) -> String {
    crate::api_client::url_decode(s)
}

/// Verify the OpenID response signature against Steam's endpoint.
/// See: https://openid.net/specs/openid-authentication-2_0.html#verification
fn verify_openid_signature(query: &str) -> Result<(), String> {
    // Build verification request: replace openid.mode with check_authentication
    let mut params: Vec<(String, String)> = Vec::new();
    for param in query.split('&') {
        let parts: Vec<&str> = param.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = url_decode(parts[0]);
            let value = url_decode(parts[1]);
            if key == "openid.mode" {
                params.push((key, "check_authentication".to_string()));
            } else {
                params.push((key, value));
            }
        }
    }

    let body: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let agent = crate::api_client::api_agent();
    let resp = agent
        .post(STEAM_OPENID_URL)
        .content_type("application/x-www-form-urlencoded")
        .send(body)
        .map_err(|e| format!("OpenID verification request failed: {}", e))?;

    let resp_body = resp
        .into_body()
        .read_to_string()
        .map_err(|e| format!("OpenID verification read error: {}", e))?;

    if resp_body.contains("is_valid:true") {
        Ok(())
    } else {
        Err("Steam OpenID signature verification failed. Login may have been tampered with.".into())
    }
}

/// Perform Steam OpenID login flow:
/// 1. Start a local HTTP server (tries ports 29876–29880)
/// 2. Open the Steam login page in the user's browser
/// 3. Wait for the callback with the Steam ID
/// 4. Return the Steam ID 64
pub fn steam_login() -> Result<String, String> {
    // Try multiple ports in case one is already in use
    let mut last_err = String::new();
    let mut bound: Option<(TcpListener, u16)> = None;
    for &port in REDIRECT_PORTS {
        match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(listener) => {
                bound = Some((listener, port));
                break;
            }
            Err(e) => {
                last_err = format!("Port {}: {}", port, e);
                eprintln!("[steam_auth] Port {} unavailable: {}", port, e);
            }
        }
    }
    let (listener, port) = bound.ok_or_else(|| {
        format!(
            "Failed to start local server on ports {:?}: {}",
            REDIRECT_PORTS, last_err
        )
    })?;

    let login_url = build_login_url(port);
    open_browser(&login_url)?;

    // Set a timeout of 2 minutes
    let timeout = Duration::from_secs(120);
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to set non-blocking: {}", e))?;

    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            return Err("Steam login timed out (2 minutes). Please try again.".into());
        }

        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = [0u8; 8192];
                stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                let n = stream.read(&mut buf).unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]).to_string();

                // Parse the query string from GET /callback?...
                if let Some(query) = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|path| path.split_once('?'))
                    .map(|(_, q)| q)
                {
                    if let Some(steam_id) = extract_steam_id(query) {
                        // Verify OpenID signature before trusting the response
                        if let Err(e) = verify_openid_signature(query) {
                            eprintln!("[steam_auth] {}", e);
                            let html = "<!DOCTYPE html><html><body style=\"background:#1a1a2e;color:#e44;font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0\">\
                                <div style=\"text-align:center\"><h1>Login Failed</h1><p>OpenID signature verification failed. Please try again.</p></div></body></html>";
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                html.len(), html
                            );
                            let _ = stream.write_all(response.as_bytes());
                            let _ = stream.flush();
                            return Err(e);
                        }

                        // Send success response
                        let html = "<!DOCTYPE html><html><body style=\"background:#1a1a2e;color:#eee;font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0\">\
                            <div style=\"text-align:center\"><h1>Steam Login Successful!</h1><p>Your Steam ID has been detected. You can close this tab and return to Spotter.</p></div></body></html>";
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            html.len(),
                            html
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                        return Ok(steam_id);
                    }
                }

                // Not the callback we expected, send a redirect
                let redirect = format!(
                    "HTTP/1.1 302 Found\r\nLocation: {}\r\nConnection: close\r\n\r\n",
                    login_url
                );
                let _ = stream.write_all(redirect.as_bytes());
                let _ = stream.flush();
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(e) => {
                return Err(format!("Server error: {}", e));
            }
        }
    }
}
