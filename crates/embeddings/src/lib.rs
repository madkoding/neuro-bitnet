//! # neuro-embeddings
//!
//! Embedding generation for the neuro-bitnet RAG system.
//!
//! This crate provides text embedding functionality using fastembed,
//! supporting various pre-trained models optimized for semantic search.
//!
//! ## Features
//!
//! - `cuda` - Enable CUDA GPU acceleration (requires NVIDIA GPU)
//!
//! ## Example
//!
//! ```no_run
//! use neuro_embeddings::{Embedder, FastEmbedder, EmbeddingModel};
//!
//! let embedder = FastEmbedder::new(EmbeddingModel::AllMiniLmL6V2).unwrap();
//!
//! // Single text
//! let embedding = embedder.embed_single("Hello, world!").unwrap();
//!
//! // Multiple texts (more efficient)
//! let embeddings = embedder.embed_batch(&["Text 1", "Text 2"]).unwrap();
//! ```

mod embedder;
mod models;
mod error;

pub use embedder::{Embedder, FastEmbedder};
pub use models::EmbeddingModel;
pub use error::{EmbeddingError, Result};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{Embedder, FastEmbedder, EmbeddingModel, EmbeddingError, Result};
}
