//! Error types for indexer operations

use thiserror::Error;

/// Errors that can occur during indexing
#[derive(Error, Debug)]
pub enum IndexerError {
    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Unsupported language
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// Parse error
    #[error("Failed to parse code: {0}")]
    ParseError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Tree-sitter error
    #[error("Tree-sitter error: {0}")]
    TreeSitter(String),
}

/// Result type for indexer operations
pub type Result<T> = std::result::Result<T, IndexerError>;

impl From<IndexerError> for neuro_core::Error {
    fn from(err: IndexerError) -> Self {
        neuro_core::Error::indexing(err.to_string())
    }
}
