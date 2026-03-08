use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

/// Default connect timeout for all HTTP requests.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Default read timeout for API calls (JSON responses).
const READ_TIMEOUT_API: Duration = Duration::from_secs(15);

/// Longer read timeout for downloads (images, large responses).
const READ_TIMEOUT_DOWNLOAD: Duration = Duration::from_secs(30);

/// Maximum backoff delay for rate limiting (60 seconds).
const MAX_BACKOFF: Duration = Duration::from_secs(60);

/// Default maximum log file size before rotation (5 MB).
const DEFAULT_MAX_LOG_SIZE: u64 = 5_000_000;

// ───── Adaptive rate limiter ─────

/// Adaptive rate limiter that adjusts delays based on API responses.
///
/// Starts at a base delay and doubles on 429 (Too Many Requests) responses,
/// up to a maximum of 60 seconds. On success, the delay gradually decreases
/// back toward the base.
pub struct RateLimiter {
    base_delay: Duration,
    current: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter with the given base delay.
    pub fn new(base_delay: Duration) -> Self {
        Self {
            base_delay,
            current: base_delay,
        }
    }

    /// Returns the current delay to wait between requests.
    #[allow(dead_code)]
    pub fn current_delay(&self) -> Duration {
        self.current
    }

    /// Call when a 429 (Too Many Requests) response is received.
    /// Doubles the current delay up to MAX_BACKOFF.
    pub fn on_rate_limited(&mut self) {
        self.current = (self.current * 2).min(MAX_BACKOFF);
        eprintln!(
            "[rate-limit] 429 received, backing off to {}ms",
            self.current.as_millis()
        );
    }

    /// Call after a successful request. Gradually reduces the delay
    /// back toward the base, but never below it.
    pub fn on_success(&mut self) {
        if self.current > self.base_delay {
            // Reduce by 25% on each success
            let reduced = self.current * 3 / 4;
            self.current = reduced.max(self.base_delay);
        }
    }

    /// Sleep for the current delay. Call between API requests.
    pub fn wait(&self) {
        std::thread::sleep(self.current);
    }
}

// ───── Persistent import logger ─────

/// Persistent logger that writes import events to a rotating log file.
///
/// Writes to `import.log` in the specified directory. When the file exceeds
/// the maximum size, the old file is rotated to `import.log.1` (keeping
/// at most one backup).
pub struct ImportLogger {
    log_path: PathBuf,
    backup_path: PathBuf,
    max_size: u64,
    file: Mutex<Option<std::fs::File>>,
}

impl ImportLogger {
    /// Create a logger that writes to `<dir>/import.log` with default 5 MB rotation.
    pub fn new(dir: &Path) -> Self {
        Self::with_max_size(dir, DEFAULT_MAX_LOG_SIZE)
    }

    /// Create a logger with a custom max file size for rotation.
    pub fn with_max_size(dir: &Path, max_size: u64) -> Self {
        let log_path = dir.join("import.log");
        let backup_path = dir.join("import.log.1");
        Self {
            log_path,
            backup_path,
            max_size,
            file: Mutex::new(None),
        }
    }

    /// Log a message with a timestamp and module tag.
    pub fn log(&self, module: &str, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("{} [{}] {}\n", timestamp, module, message);

        let mut guard = match self.file.lock() {
            Ok(g) => g,
            Err(e) => {
                eprintln!("[logger] Lock poisoned: {}", e);
                return;
            }
        };

        // Rotate if needed — check size and rename, then drop the old handle
        if let Ok(meta) = std::fs::metadata(&self.log_path) {
            if meta.len() >= self.max_size {
                // Drop the old file handle before renaming
                *guard = None;
                let _ = std::fs::rename(&self.log_path, &self.backup_path);
                eprintln!(
                    "[logger] Rotated import.log ({} bytes) → import.log.1",
                    meta.len()
                );
            }
        }

        // Open or reuse file handle
        let file = match guard.as_mut() {
            Some(f) => f,
            None => {
                match std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.log_path)
                {
                    Ok(f) => {
                        *guard = Some(f);
                        guard.as_mut().unwrap()
                    }
                    Err(e) => {
                        eprintln!("[logger] Failed to open log file: {}", e);
                        return;
                    }
                }
            }
        };

        if let Err(e) = file.write_all(line.as_bytes()) {
            eprintln!("[logger] Failed to write log: {}", e);
        }
        let _ = file.flush();
    }
}

/// Return the path to the import log directory (same as app data dir).
pub fn log_dir() -> PathBuf {
    let dir = crate::db::base_dir().join("logs");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("[logger] Failed to create log dir: {}", e);
    }
    dir
}

/// Create an HTTP agent suitable for API calls (15s read timeout).
pub fn api_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_recv_body(Some(READ_TIMEOUT_API))
        .build()
        .new_agent()
}

/// Create an HTTP agent suitable for downloads (30s read timeout).
pub fn download_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_recv_body(Some(READ_TIMEOUT_DOWNLOAD))
        .build()
        .new_agent()
}

/// HTTP GET with configurable retries and exponential backoff.
/// Returns early on 4xx client errors (no retry), except 429 which triggers backoff.
/// Returns the response body as a String.
pub fn http_get_retry(agent: &ureq::Agent, url: &str, max_retries: u32) -> Result<String, String> {
    let mut last_err = String::new();
    let mut was_rate_limited = false;
    for attempt in 0..=max_retries {
        if attempt > 0 && !was_rate_limited {
            let wait = Duration::from_millis(500 * (1 << attempt));
            std::thread::sleep(wait);
        }
        was_rate_limited = false;
        match agent.get(url).call() {
            Ok(resp) => {
                let body = resp
                    .into_body()
                    .read_to_string()
                    .map_err(|e| format!("Read error: {}", e))?;
                return Ok(body);
            }
            Err(ureq::Error::StatusCode(429)) => {
                let wait = Duration::from_millis(1000 * (1 << attempt)).min(MAX_BACKOFF);
                eprintln!(
                    "[api] 429 Too Many Requests, waiting {}ms before retry",
                    wait.as_millis()
                );
                std::thread::sleep(wait);
                was_rate_limited = true;
                last_err = "HTTP 429 Too Many Requests".to_string();
            }
            Err(ureq::Error::StatusCode(code)) if (400..500).contains(&(code as u32)) => {
                return Err(format!("HTTP {}", code));
            }
            Err(e) => {
                last_err = e.to_string();
            }
        }
    }
    Err(format!(
        "Failed after {} attempts: {}",
        max_retries + 1,
        last_err
    ))
}

/// HTTP GET with a Bearer token and retry logic.
/// Suitable for GOG, PSN, and other OAuth-based APIs.
/// Handles 429 responses with exponential backoff.
pub fn http_get_bearer(
    agent: &ureq::Agent,
    url: &str,
    token: &str,
    max_retries: u32,
) -> Result<String, String> {
    let mut last_err = String::new();
    let mut was_rate_limited = false;
    for attempt in 0..=max_retries {
        if attempt > 0 && !was_rate_limited {
            let wait = Duration::from_millis(500 * (1 << attempt));
            std::thread::sleep(wait);
        }
        was_rate_limited = false;
        match agent
            .get(url)
            .header("Authorization", &format!("Bearer {}", token))
            .header("Accept", "application/json")
            .call()
        {
            Ok(resp) => {
                let body = resp
                    .into_body()
                    .read_to_string()
                    .map_err(|e| e.to_string())?;
                return Ok(body);
            }
            Err(ureq::Error::StatusCode(429)) => {
                let wait = Duration::from_millis(1000 * (1 << attempt)).min(MAX_BACKOFF);
                eprintln!(
                    "[api] 429 Too Many Requests, waiting {}ms before retry",
                    wait.as_millis()
                );
                std::thread::sleep(wait);
                was_rate_limited = true;
                last_err = "HTTP 429 Too Many Requests".to_string();
            }
            Err(ureq::Error::StatusCode(code)) if (400..500).contains(&(code as u32)) => {
                return Err(format!("HTTP {}", code));
            }
            Err(e) => {
                last_err = e.to_string();
            }
        }
    }
    Err(format!(
        "Failed after {} attempts: {}",
        max_retries + 1,
        last_err
    ))
}

/// HTTP GET with custom headers and retry logic.
/// Suitable for Xbox/OpenXBL and other APIs with custom auth headers.
/// Handles 429 responses with exponential backoff.
pub fn http_get_headers(
    url: &str,
    headers: &[(&str, &str)],
    max_retries: u32,
) -> Result<String, String> {
    let agent = api_agent();
    let mut last_err = String::new();
    let mut was_rate_limited = false;
    for attempt in 0..=max_retries {
        if attempt > 0 && !was_rate_limited {
            let wait = Duration::from_millis(1000 * (1 << attempt));
            std::thread::sleep(wait);
        }
        was_rate_limited = false;
        let mut req = agent.get(url);
        for &(key, value) in headers {
            req = req.header(key, value);
        }
        match req.call() {
            Ok(resp) => {
                let body = resp
                    .into_body()
                    .read_to_string()
                    .map_err(|e| e.to_string())?;
                return Ok(body);
            }
            Err(ureq::Error::StatusCode(429)) => {
                let wait = Duration::from_millis(2000 * (1 << attempt)).min(MAX_BACKOFF);
                eprintln!(
                    "[api] 429 Too Many Requests, waiting {}ms before retry",
                    wait.as_millis()
                );
                std::thread::sleep(wait);
                was_rate_limited = true;
                last_err = "HTTP 429 Too Many Requests".to_string();
            }
            Err(ureq::Error::StatusCode(code)) if (400..500).contains(&(code as u32)) => {
                return Err(format!("HTTP {}", code));
            }
            Err(e) => {
                last_err = e.to_string();
            }
        }
    }
    Err(format!(
        "Failed after {} attempts: {}",
        max_retries + 1,
        last_err
    ))
}

/// Percent-encode a string for use in URLs.
pub fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// Percent-decode a URL-encoded string, handling multi-byte UTF-8 sequences.
pub fn url_decode(s: &str) -> String {
    let mut bytes: Vec<u8> = Vec::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                bytes.push(byte);
            }
        } else if c == '+' {
            bytes.push(b' ');
        } else {
            // Encode the char as UTF-8 bytes
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            bytes.extend_from_slice(encoded.as_bytes());
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|e| {
        eprintln!("[api_client] URL decode invalid UTF-8: {}", e);
        s.to_string()
    })
}

/// Open a URL in the default system browser.
///
/// On Windows, uses `rundll32 url.dll,FileProtocolHandler` instead of
/// `cmd /C start` to avoid shell metacharacter injection risks.
/// On Linux/macOS, the URL is passed as a single argument (no shell involved).
pub fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        // Use rundll32 to open the URL via the shell URL handler.
        // This avoids passing the URL through cmd.exe where special characters
        // like &, |, >, < could be interpreted as shell operators.
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Err(format!(
            "Opening a browser is not supported on this platform"
        ));
    }
    Ok(())
}
