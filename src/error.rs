//! Typed errors returned by every SDK call on non-2xx responses and on
//! transport-level failures.
//!
//! The [`ErrorCode`] enum captures the known codes. Unknown server-side
//! codes pass through via [`ErrorCode::Other`] so callers can handle new
//! codes without waiting for an SDK update.

use std::fmt;

/// Single error type raised by every SDK call on failure.
#[derive(thiserror::Error, Debug, Clone)]
pub struct Error {
    /// Machine-readable error code.
    pub code: ErrorCode,
    /// HTTP status code; 0 for network / timeout failures.
    pub status: u16,
    /// Human-readable error message.
    pub message: String,
    /// Numeric `code` field from the Cryptohopper error envelope, when
    /// present. Identifies the rate-limit bucket or other server-side
    /// diagnostic code.
    pub server_code: Option<i64>,
    /// Client IP the server saw. Useful for debugging OAuth IP-whitelist
    /// mismatches.
    pub ip_address: Option<String>,
    /// Populated on 429 from the `Retry-After` header.
    pub retry_after_ms: Option<u64>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.status == 0 {
            write!(f, "cryptohopper: [{}] {}", self.code, self.message)
        } else {
            write!(
                f,
                "cryptohopper: [{} {}] {}",
                self.code, self.status, self.message
            )
        }
    }
}

impl Error {
    pub(crate) fn network(message: impl Into<String>) -> Self {
        Error {
            code: ErrorCode::NetworkError,
            status: 0,
            message: message.into(),
            server_code: None,
            ip_address: None,
            retry_after_ms: None,
        }
    }

    pub(crate) fn timeout(message: impl Into<String>) -> Self {
        Error {
            code: ErrorCode::Timeout,
            status: 0,
            message: message.into(),
            server_code: None,
            ip_address: None,
            retry_after_ms: None,
        }
    }
}

/// Error codes returned by the server or derived from transport failures.
///
/// The variants mirror the shared taxonomy across every Cryptohopper SDK.
/// Use [`ErrorCode::Other`] to match unknown server codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCode {
    ValidationError,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    RateLimited,
    ServerError,
    ServiceUnavailable,
    DeviceUnauthorized,
    NetworkError,
    Timeout,
    Unknown,
    /// Server returned a code the SDK doesn't know about yet.
    Other(String),
}

impl ErrorCode {
    /// The string representation the server uses.
    pub fn as_str(&self) -> &str {
        match self {
            ErrorCode::ValidationError => "VALIDATION_ERROR",
            ErrorCode::Unauthorized => "UNAUTHORIZED",
            ErrorCode::Forbidden => "FORBIDDEN",
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::Conflict => "CONFLICT",
            ErrorCode::RateLimited => "RATE_LIMITED",
            ErrorCode::ServerError => "SERVER_ERROR",
            ErrorCode::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            ErrorCode::DeviceUnauthorized => "DEVICE_UNAUTHORIZED",
            ErrorCode::NetworkError => "NETWORK_ERROR",
            ErrorCode::Timeout => "TIMEOUT",
            ErrorCode::Unknown => "UNKNOWN",
            ErrorCode::Other(s) => s.as_str(),
        }
    }

    /// Derive a code from an HTTP status.
    pub(crate) fn from_status(status: u16) -> Self {
        match status {
            400 | 422 => ErrorCode::ValidationError,
            401 => ErrorCode::Unauthorized,
            402 => ErrorCode::DeviceUnauthorized,
            403 => ErrorCode::Forbidden,
            404 => ErrorCode::NotFound,
            409 => ErrorCode::Conflict,
            429 => ErrorCode::RateLimited,
            503 => ErrorCode::ServiceUnavailable,
            s if s >= 500 => ErrorCode::ServerError,
            _ => ErrorCode::Unknown,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
