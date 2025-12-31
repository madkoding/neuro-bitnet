//! Error types for neuro-bitnet

use thiserror::Error;

/// Main error type for neuro-bitnet operations
#[derive(Error, Debug)]
pub enum Error {
    /// Embedding generation failed
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// Storage operation failed
    #[error("Storage error: {0}")]
    Storage(String),

    /// Document not found
    #[error("Document not found: {0}")]
    NotFound(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Classification error
    #[error("Classification error: {0}")]
    Classification(String),

    /// Indexing error
    #[error("Indexing error: {0}")]
    Indexing(String),

    /// Web search error
    #[error("Web search error: {0}")]
    WebSearch(String),

    /// Server error
    #[error("Server error: {0}")]
    Server(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias using our Error
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create an embedding error
    pub fn embedding(msg: impl Into<String>) -> Self {
        Self::Embedding(msg.into())
    }

    /// Create a storage error
    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage(msg.into())
    }

    /// Create a not found error
    pub fn not_found(id: impl Into<String>) -> Self {
        Self::NotFound(id.into())
    }

    /// Create an invalid input error
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Create a classification error
    pub fn classification(msg: impl Into<String>) -> Self {
        Self::Classification(msg.into())
    }

    /// Create an indexing error
    pub fn indexing(msg: impl Into<String>) -> Self {
        Self::Indexing(msg.into())
    }

    /// Create a web search error
    pub fn web_search(msg: impl Into<String>) -> Self {
        Self::WebSearch(msg.into())
    }

    /// Create a server error
    pub fn server(msg: impl Into<String>) -> Self {
        Self::Server(msg.into())
    }

    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::not_found("doc123");
        assert_eq!(err.to_string(), "Document not found: doc123");
    }

    #[test]
    fn test_error_constructors() {
        assert!(matches!(Error::embedding("test"), Error::Embedding(_)));
        assert!(matches!(Error::storage("test"), Error::Storage(_)));
        assert!(matches!(Error::not_found("id"), Error::NotFound(_)));
    }
}
