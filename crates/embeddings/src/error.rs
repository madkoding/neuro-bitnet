//! Error types for embedding operations

use thiserror::Error;

/// Errors that can occur during embedding operations
#[derive(Error, Debug)]
pub enum EmbeddingError {
    /// Model initialization failed
    #[error("Failed to initialize embedding model: {0}")]
    ModelInit(String),

    /// Embedding generation failed
    #[error("Failed to generate embedding: {0}")]
    Generation(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Dimension mismatch
    #[error("Embedding dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
}

/// Result type for embedding operations
pub type Result<T> = std::result::Result<T, EmbeddingError>;

impl From<EmbeddingError> for neuro_core::Error {
    fn from(err: EmbeddingError) -> Self {
        neuro_core::Error::embedding(err.to_string())
    }
}
