//! # neuro-core
//!
//! Core types and models for the neuro-bitnet RAG (Retrieval Augmented Generation) system.
//!
//! This crate provides the foundational data structures used across all other crates:
//! - [`Document`] - Represents a stored document with embeddings
//! - [`SearchResult`] - Result from similarity search
//! - [`QueryResult`] - Complete result from RAG query
//! - [`ClassificationResult`] - Query classification output
//! - [`QueryCategory`] - Categories for query classification
//! - [`QueryStrategy`] - Strategies for handling queries
//! - [`DocumentSource`] - Source types for documents

mod document;
mod error;
mod classification;
mod search;

pub use document::{Document, DocumentSource};
pub use error::{Error, Result};
pub use classification::{ClassificationResult, QueryCategory, QueryStrategy};
pub use search::{SearchResult, QueryResult};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        Document, DocumentSource,
        ClassificationResult, QueryCategory, QueryStrategy,
        SearchResult, QueryResult,
        Error, Result,
    };
}
