use std::env;
use std::fs;

/// Application configuration loaded from environment variables or local `.env` file.
///
/// Implements custom [`std::fmt::Debug`] to ensure the `app_key` secret is
/// redacted and never leaked to standard output or log streams.
#[derive(Clone)]
pub struct AppConfig {
    /// Secret YouVersion application key used for API authentication (`x-yvp-app-key`).
    pub app_key: String,
    /// Base URL for the YouVersion REST API.
    pub base_url: String,
    /// Local TCP port to bind the HTTP server to.
    pub port: u16,
    /// Local network interface/host to bind to (e.g. `"0.0.0.0"` or `"127.0.0.1"`).
    pub host: String,
}

impl std::fmt::Debug for AppConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppConfig")
            .field("app_key", &"[REDACTED]")
            .field("base_url", &self.base_url)
            .field("port", &self.port)
            .field("host", &self.host)
            .finish()
    }
}

impl AppConfig {
    /// Loads configuration values from `.env` file or environment variables.
    ///
    /// - `YOUVERSION_APP_KEY`: Secret application key. Defaults to empty string if unset.
    /// - `YOUVERSION_BASE_URL`: Defaults to `"https://api.youversion.com"`.
    /// - `PORT`: Defaults to `3000`.
    /// - `HOST`: Defaults to `"0.0.0.0"`.
    pub fn from_env() -> Self {
        Self::load_dotenv();

        let app_key = env::var("YOUVERSION_APP_KEY").unwrap_or_default();
        let base_url = env::var("YOUVERSION_BASE_URL")
            .unwrap_or_else(|_| "https://api.youversion.com".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(3000);
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        Self {
            app_key,
            base_url,
            port,
            host,
        }
    }

    /// Lightweight parser that loads key-value pairs from a local `.env` file if present,
    /// without overwriting pre-existing process environment variables.
    fn load_dotenv() {
        if let Ok(contents) = fs::read_to_string(".env") {
            for line in contents.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Some((key, val)) = trimmed.split_once('=') {
                    let key = key.trim();
                    let val = val.trim().trim_matches('"').trim_matches('\'');
                    if env::var(key).is_err() {
                        env::set_var(key, val);
                    }
                }
            }
        }
    }
}
