//! Error types for search operations

use thiserror::Error;

/// Errors that can occur during web search
#[derive(Error, Debug)]
pub enum SearchError {
    /// HTTP request failed
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Failed to parse response
    #[error("Parse error: {0}")]
    Parse(String),

    /// No results found
    #[error("No results found for query: {0}")]
    NoResults(String),

    /// Rate limited
    #[error("Rate limited, try again later")]
    RateLimited,

    /// Timeout
    #[error("Request timed out")]
    Timeout,

    /// Invalid query
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

/// Result type for search operations
pub type Result<T> = std::result::Result<T, SearchError>;

impl From<SearchError> for neuro_core::Error {
    fn from(err: SearchError) -> Self {
        neuro_core::Error::web_search(err.to_string())
    }
}
