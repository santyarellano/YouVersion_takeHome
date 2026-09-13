use serde::{Deserialize, Serialize};

/// The unified Verse of the Day response returned by our API.
///
/// Exactly matches the 4 required fields from the specification:
/// - `day`: integer (1-366)
/// - `reference`: string (e.g. "Revelation 3:20")
/// - `text`: string
/// - `version_id`: integer (e.g. 206)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VotdResponse {
    /// Day ordinal of the year (1–366).
    pub day: u32,
    /// Human-readable scripture reference (e.g., `"Revelation 3:20"`).
    pub reference: String,
    /// Full scripture passage text.
    pub text: String,
    /// Bible translation version identifier (e.g., 206 for WEB).
    pub version_id: u64,
}

/// Standardized error object details schema:
/// `{ "code": "...", "message": "..." }`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorDetail {
    /// Machine-readable error code (e.g., `"INVALID_DAY"`, `"UPSTREAM_ERROR"`).
    pub code: String,
    /// Descriptive error message explaining the issue.
    pub message: String,
}

/// Standardized top-level error response envelope:
/// `{ "error": { "code": "...", "message": "..." } }`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorResponse {
    /// Inner error details.
    pub error: ErrorDetail,
}

impl ErrorResponse {
    /// Constructs a new `ErrorResponse` from a code and human-readable message.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ErrorDetail {
                code: code.into(),
                message: message.into(),
            },
        }
    }

    /// Serializes this error response to a JSON string, falling back to a safe static JSON on error.
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"error":{"code":"INTERNAL_ERROR","message":"Failed to serialize error"}}"#.to_string()
        })
    }
}

/// Summary item for the `GET /versions` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BibleVersionSummary {
    /// Unique numeric Bible version identifier (e.g., 206).
    pub id: u64,
    /// Standard translation abbreviation (e.g., `"WEB"`, `"ASV"`).
    pub abbreviation: String,
    /// Full translation title (e.g., `"World English Bible"`).
    pub title: String,
}

/// Response payload for `GET /versions`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionsResponse {
    /// List of available Bible translation summaries.
    pub versions: Vec<BibleVersionSummary>,
}

/// Lightweight HTTP response representation (status code + JSON body string).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    /// HTTP status code (e.g., 200, 400, 404, 502).
    pub status: u16,
    /// Serialized response body payload.
    pub body: String,
}

impl HttpResponse {
    /// Constructs an `HttpResponse` by serializing a data model to JSON with the given status code.
    pub fn json<T: Serialize>(status: u16, data: &T) -> Self {
        let body = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
        Self { status, body }
    }

    /// Constructs an `HttpResponse` containing a standardized error JSON payload.
    pub fn error(status: u16, code: &str, message: &str) -> Self {
        let err = ErrorResponse::new(code, message);
        Self {
            status,
            body: err.to_json_string(),
        }
    }
}

// -------------------------------------------------------------
// Upstream YouVersion API DTOs (internal deserialization only)
// -------------------------------------------------------------

/// Internal DTO deserialized from upstream `GET /v1/verse_of_the_days/{day}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpstreamVotdDto {
    /// Day ordinal (1–366).
    pub day: u32,
    /// Internal passage USFM reference ID (e.g., `"REV.3.20"`).
    pub passage_id: String,
}

/// Internal DTO deserialized from upstream `GET /v1/bibles/{version_id}/passages/{passage_id}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpstreamPassageDto {
    /// Passage USFM identifier (e.g., `"REV.3.20"`).
    pub id: String,
    /// Plaintext verse content.
    pub content: String,
    /// Human-readable book, chapter, and verse reference (e.g., `"Revelation 3:20"`).
    pub reference: String,
}

/// Internal DTO representing an individual Bible entry in upstream `GET /v1/bibles`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpstreamBibleItemDto {
    /// Unique Bible version ID.
    pub id: u64,
    /// Version abbreviation.
    pub abbreviation: String,
    /// Optional language-specific or regional title provided by YouVersion.
    ///
    /// When available, this provides the most descriptive, localized human-readable name.
    /// Example: `Some("World English Bible, American English Edition, without Strong's Numbers")` or `Some("American Standard Version")`.
    pub localized_title: Option<String>,

    /// Fallback standard or publisher title provided by YouVersion.
    ///
    /// Used as a fallback when `localized_title` is omitted or null in the upstream JSON.
    /// Example: `Some("World English Bible")` or `Some("The Holy Bible")`.
    pub title: Option<String>,
}

/// Internal DTO deserialized from upstream `GET /v1/bibles` containing a collection of Bibles.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpstreamBiblesResponseDto {
    /// Array of Bible version items.
    pub data: Vec<UpstreamBibleItemDto>,
}
