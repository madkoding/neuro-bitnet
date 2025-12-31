//! Error types for the LLM client.

use thiserror::Error;

/// Result type for LLM operations.
pub type Result<T> = std::result::Result<T, LlmError>;

/// Errors that can occur when interacting with an LLM server.
#[derive(Error, Debug)]
pub enum LlmError {
    /// Server connection error
    #[error("Failed to connect to LLM server: {0}")]
    ConnectionError(String),

    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Server returned an error
    #[error("Server error ({status}): {message}")]
    ServerError {
        status: u16,
        message: String,
    },

    /// JSON parsing error
    #[error("Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Server not available
    #[error("LLM server is not available at {url}")]
    ServerUnavailable {
        url: String,
    },

    /// Timeout error
    #[error("Request timed out after {seconds} seconds")]
    Timeout {
        seconds: u64,
    },

    /// Empty response
    #[error("Server returned empty response")]
    EmptyResponse,

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
