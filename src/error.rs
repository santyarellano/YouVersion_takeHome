use std::fmt;

use crate::models::HttpResponse;

/// Application error types mapped to standard HTTP status codes and JSON error schemas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    /// Invalid day parameter (not an integer between 1 and 366). Mapped to HTTP 400.
    InvalidDay(String),
    /// Invalid version parameter (not a positive integer). Mapped to HTTP 400.
    InvalidVersion(String),
    /// Upstream YouVersion API failure (4xx, 5xx, or network timeout). Mapped to HTTP 502.
    UpstreamError(String),
    /// Unrecognized endpoint or path. Mapped to HTTP 404.
    NotFound(String),
    /// Unimplemented feature or stub. Mapped to HTTP 501.
    NotImplemented(String),
    /// Generic internal server error. Mapped to HTTP 500.
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidDay(msg) => write!(f, "INVALID_DAY: {}", msg),
            AppError::InvalidVersion(msg) => write!(f, "INVALID_VERSION: {}", msg),
            AppError::UpstreamError(msg) => write!(f, "UPSTREAM_ERROR: {}", msg),
            AppError::NotFound(msg) => write!(f, "NOT_FOUND: {}", msg),
            AppError::NotImplemented(msg) => write!(f, "NOT_IMPLEMENTED: {}", msg),
            AppError::InternalError(msg) => write!(f, "INTERNAL_ERROR: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl AppError {
    /// Returns the associated HTTP status code for this error variant.
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::InvalidDay(_) => 400,
            AppError::InvalidVersion(_) => 400,
            AppError::UpstreamError(_) => 502,
            AppError::NotFound(_) => 404,
            AppError::NotImplemented(_) => 501,
            AppError::InternalError(_) => 500,
        }
    }

    /// Returns the standardized error string code required by the specification.
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::InvalidDay(_) => "INVALID_DAY",
            AppError::InvalidVersion(_) => "INVALID_VERSION",
            AppError::UpstreamError(_) => "UPSTREAM_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::NotImplemented(_) => "NOT_IMPLEMENTED",
            AppError::InternalError(_) => "INTERNAL_ERROR",
        }
    }

    /// Returns the human-readable description for this error.
    pub fn error_message(&self) -> String {
        match self {
            AppError::InvalidDay(msg)
            | AppError::InvalidVersion(msg)
            | AppError::UpstreamError(msg)
            | AppError::NotFound(msg)
            | AppError::NotImplemented(msg)
            | AppError::InternalError(msg) => msg.clone(),
        }
    }

    /// Converts this error into an [`HttpResponse`] with matching status code and JSON error body.
    pub fn to_http_response(&self) -> HttpResponse {
        HttpResponse::error(self.status_code(), self.error_code(), &self.error_message())
    }
}
