use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::error::AppError;
use crate::models::{
    BibleVersionSummary, UpstreamBiblesResponseDto, UpstreamPassageDto, UpstreamVotdDto,
};

/// Abstraction layer for interacting with the external YouVersion Platform API.
///
/// This trait defines the contract required by [`VotdService`](crate::service::VotdService)
/// to fetch Verse of the Day identifiers, passage texts, and available Bible versions.
/// By decoupling the upstream API behind a trait, the application supports both live HTTP
/// networking via [`HttpYouVersionClient`] and deterministic, zero-network testing via [`MockYouVersionClient`].
pub trait YouVersionClient: Send + Sync {
    /// Fetches the Verse of the Day metadata (including passage ID) for a specific day of the year (1–366).
    ///
    /// # Arguments
    /// * `day` - The day of the year (ordinal 1–366).
    ///
    /// # Errors
    /// Returns [`AppError::UpstreamError`] if the upstream request fails or returns an error status.
    fn get_verse_of_the_day(&self, day: u32) -> Result<UpstreamVotdDto, AppError>;

    /// Fetches the formatted passage content and reference for a specific Bible version and passage ID.
    ///
    /// # Arguments
    /// * `version_id` - The numeric Bible version identifier (e.g., 206 for WEB).
    /// * `passage_id` - The YouVersion passage USFM identifier (e.g., `"REV.3.20"`).
    ///
    /// # Errors
    /// Returns [`AppError::UpstreamError`] if the upstream request fails, returns 404/403/500, or times out.
    fn get_passage(&self, version_id: u64, passage_id: &str) -> Result<UpstreamPassageDto, AppError>;

    /// Fetches all available English Bible translations.
    ///
    /// # Errors
    /// Returns [`AppError::UpstreamError`] if the upstream API fails or cannot be reached.
    fn get_bibles(&self) -> Result<Vec<BibleVersionSummary>, AppError>;
}

/// Production HTTP client for YouVersion API using lightweight synchronous `ureq`.
///
/// Handles outbound HTTP requests, API authentication headers (`x-yvp-app-key`),
/// and request timeouts without requiring an asynchronous runtime or complex dependencies.
pub struct HttpYouVersionClient {
    /// Base URL for YouVersion API endpoints (e.g., `https://developers.youversion.com`).
    base_url: String,
    /// Secret API application key sent via the `x-yvp-app-key` header.
    app_key: String,
    /// Synchronous HTTP agent managing connection pooling and request timeouts.
    agent: ureq::Agent,
}

impl HttpYouVersionClient {
    /// Constructs a new `HttpYouVersionClient` configured with a base URL, API key, and 10-second timeout.
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the YouVersion API.
    /// * `app_key` - The secret application key for API authentication.
    pub fn new(base_url: String, app_key: &str) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(10))
            .build();
        Self {
            base_url,
            app_key: app_key.to_string(),
            agent,
        }
    }
}

impl YouVersionClient for HttpYouVersionClient {
    /// Calls `GET /v1/verse_of_the_days/{day}` on the upstream YouVersion API.
    fn get_verse_of_the_day(&self, day: u32) -> Result<UpstreamVotdDto, AppError> {
        let url = format!("{}/v1/verse_of_the_days/{}", self.base_url, day);
        let resp = self
            .agent
            .get(&url)
            .set("x-yvp-app-key", &self.app_key)
            .call()
            .map_err(|e| match e {
                ureq::Error::Status(code, _) => {
                    AppError::UpstreamError(format!("Upstream YouVersion API error (status {})", code))
                }
                ureq::Error::Transport(t) => {
                    AppError::UpstreamError(format!("Network request failed: {}", t))
                }
            })?;

        resp.into_json::<UpstreamVotdDto>()
            .map_err(|e| AppError::UpstreamError(format!("Failed to parse upstream response: {}", e)))
    }

    /// Calls `GET /v1/bibles/{version_id}/passages/{passage_id}?format=text` on the upstream YouVersion API.
    fn get_passage(&self, version_id: u64, passage_id: &str) -> Result<UpstreamPassageDto, AppError> {
        let url = format!(
            "{}/v1/bibles/{}/passages/{}?format=text",
            self.base_url, version_id, passage_id
        );
        let resp = self
            .agent
            .get(&url)
            .set("x-yvp-app-key", &self.app_key)
            .call()
            .map_err(|e| match e {
                ureq::Error::Status(code, _) => {
                    AppError::UpstreamError(format!("Upstream YouVersion API error (status {})", code))
                }
                ureq::Error::Transport(t) => {
                    AppError::UpstreamError(format!("Network request failed: {}", t))
                }
            })?;

        resp.into_json::<UpstreamPassageDto>()
            .map_err(|e| AppError::UpstreamError(format!("Failed to parse upstream response: {}", e)))
    }

    /// Calls `GET /v1/bibles?language_ranges[]=en*` on the upstream YouVersion API and maps to summary models.
    fn get_bibles(&self) -> Result<Vec<BibleVersionSummary>, AppError> {
        let url = format!("{}/v1/bibles?language_ranges[]=en*", self.base_url);
        let resp = self
            .agent
            .get(&url)
            .set("x-yvp-app-key", &self.app_key)
            .call()
            .map_err(|e| match e {
                ureq::Error::Status(code, _) => {
                    AppError::UpstreamError(format!("Upstream YouVersion API error (status {})", code))
                }
                ureq::Error::Transport(t) => {
                    AppError::UpstreamError(format!("Network request failed: {}", t))
                }
            })?;

        let raw = resp
            .into_json::<UpstreamBiblesResponseDto>()
            .map_err(|e| AppError::UpstreamError(format!("Failed to parse upstream response: {}", e)))?;

        let list = raw
            .data
            .into_iter()
            .map(|item| BibleVersionSummary {
                id: item.id,
                abbreviation: item.abbreviation,
                title: item
                    .localized_title
                    .or(item.title)
                    .unwrap_or_else(|| "Unknown".to_string()),
            })
            .collect();

        Ok(list)
    }
}

/// Configurable mock client for unit and integration testing without invoking the live YouVersion API.
///
/// Supports pre-configuring canned responses or errors for VOTD, passage text, and Bible lists,
/// and tracks invocation counts using atomic counters to verify caching behavior.
#[derive(Clone, Default)]
pub struct MockYouVersionClient {
    /// Pre-configured mock responses for `get_verse_of_the_day`, keyed by day (1–366).
    pub votd_responses: Arc<Mutex<HashMap<u32, Result<UpstreamVotdDto, AppError>>>>,
    /// Pre-configured mock responses for `get_passage`, keyed by `(version_id, passage_id)`.
    pub passage_responses: Arc<Mutex<HashMap<(u64, String), Result<UpstreamPassageDto, AppError>>>>,
    /// Pre-configured mock response for `get_bibles`.
    pub bibles_response: Arc<Mutex<Option<Result<Vec<BibleVersionSummary>, AppError>>>>,
    /// Atomic counter tracking total calls to `get_verse_of_the_day`.
    pub votd_call_count: Arc<AtomicUsize>,
    /// Atomic counter tracking total calls to `get_passage`.
    pub passage_call_count: Arc<AtomicUsize>,
    /// Atomic counter tracking total calls to `get_bibles`.
    pub bibles_call_count: Arc<AtomicUsize>,
}

impl MockYouVersionClient {
    /// Creates a new, empty `MockYouVersionClient` with zeroed call counters and empty response tables.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder helper to register a mock response for a given day ordinal.
    pub fn with_votd(self, day: u32, res: Result<UpstreamVotdDto, AppError>) -> Self {
        self.votd_responses.lock().unwrap().insert(day, res);
        self
    }

    /// Builder helper to register a mock passage response for a given version and passage ID.
    pub fn with_passage(
        self,
        version_id: u64,
        passage_id: &str,
        res: Result<UpstreamPassageDto, AppError>,
    ) -> Self {
        self.passage_responses
            .lock()
            .unwrap()
            .insert((version_id, passage_id.to_string()), res);
        self
    }

    /// Builder helper to register a mock response for Bible version listings.
    pub fn with_bibles(self, res: Result<Vec<BibleVersionSummary>, AppError>) -> Self {
        *self.bibles_response.lock().unwrap() = Some(res);
        self
    }

    /// Returns the number of times `get_verse_of_the_day` was invoked.
    pub fn get_votd_call_count(&self) -> usize {
        self.votd_call_count.load(Ordering::SeqCst)
    }

    /// Returns the number of times `get_passage` was invoked.
    pub fn get_passage_call_count(&self) -> usize {
        self.passage_call_count.load(Ordering::SeqCst)
    }

    /// Returns the number of times `get_bibles` was invoked.
    pub fn get_bibles_call_count(&self) -> usize {
        self.bibles_call_count.load(Ordering::SeqCst)
    }
}

impl YouVersionClient for MockYouVersionClient {
    /// Increments the VOTD invocation counter and returns the registered mock response for the day.
    fn get_verse_of_the_day(&self, day: u32) -> Result<UpstreamVotdDto, AppError> {
        self.votd_call_count.fetch_add(1, Ordering::SeqCst);
        let guard = self.votd_responses.lock().unwrap();
        match guard.get(&day) {
            Some(res) => res.clone(),
            None => Err(AppError::UpstreamError(format!("Mock VOTD not found for day {}", day))),
        }
    }

    /// Increments the passage invocation counter and returns the registered mock response for the version and passage ID.
    fn get_passage(&self, version_id: u64, passage_id: &str) -> Result<UpstreamPassageDto, AppError> {
        self.passage_call_count.fetch_add(1, Ordering::SeqCst);
        let guard = self.passage_responses.lock().unwrap();
        match guard.get(&(version_id, passage_id.to_string())) {
            Some(res) => res.clone(),
            None => Err(AppError::UpstreamError(format!(
                "Mock passage not found for version {} passage {}",
                version_id, passage_id
            ))),
        }
    }

    /// Increments the bibles invocation counter and returns the registered mock response for Bible versions.
    fn get_bibles(&self) -> Result<Vec<BibleVersionSummary>, AppError> {
        self.bibles_call_count.fetch_add(1, Ordering::SeqCst);
        let guard = self.bibles_response.lock().unwrap();
        match guard.as_ref() {
            Some(res) => res.clone(),
            None => Err(AppError::UpstreamError("Mock bibles response not set".to_string())),
        }
    }
}
