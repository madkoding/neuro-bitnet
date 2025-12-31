//! Error types for inference operations

use thiserror::Error;

/// Errors that can occur during inference
#[derive(Error, Debug)]
pub enum InferenceError {
    #[error("Failed to initialize backend: {0}")]
    BackendInit(String),

    #[error("Failed to load model from '{path}': {message}")]
    ModelLoad { path: String, message: String },

    #[error("Failed to create context: {0}")]
    ContextCreation(String),

    #[error("Tokenization error: {0}")]
    Tokenization(String),

    #[error("Decode error: {0}")]
    Decode(String),

    #[error("Sampling error: {0}")]
    Sampling(String),

    #[error("Model not loaded")]
    ModelNotLoaded,

    #[error("Generation interrupted")]
    Interrupted,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, InferenceError>;
