//! Server error types

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

/// Server errors
#[derive(Error, Debug)]
pub enum ServerError {
    /// Bad request
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(#[from] neuro_storage::StorageError),

    /// Embedding error
    #[error("Embedding error: {0}")]
    Embedding(#[from] neuro_embeddings::EmbeddingError),

    /// Search error
    #[error("Search error: {0}")]
    Search(#[from] neuro_search::SearchError),

    /// Core error
    #[error("{0}")]
    Core(#[from] neuro_core::Error),
}

/// Result type for server operations
pub type Result<T> = std::result::Result<T, ServerError>;

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ServerError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ServerError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ServerError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ServerError::Storage(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ServerError::Embedding(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ServerError::Search(e) => (StatusCode::BAD_GATEWAY, e.to_string()),
            ServerError::Core(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
