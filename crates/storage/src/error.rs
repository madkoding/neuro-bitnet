//! Error types for storage operations

use thiserror::Error;

/// Errors that can occur during storage operations
#[derive(Error, Debug)]
pub enum StorageError {
    /// Document not found
    #[error("Document not found: {0}")]
    NotFound(String),

    /// Document already exists
    #[error("Document already exists: {0}")]
    AlreadyExists(String),

    /// Missing embedding
    #[error("Document is missing embedding: {0}")]
    MissingEmbedding(String),

    /// Dimension mismatch
    #[error("Embedding dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Storage is empty
    #[error("Storage is empty")]
    Empty,

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Result type for storage operations
pub type Result<T> = std::result::Result<T, StorageError>;

impl From<StorageError> for neuro_core::Error {
    fn from(err: StorageError) -> Self {
        neuro_core::Error::storage(err.to_string())
    }
}
