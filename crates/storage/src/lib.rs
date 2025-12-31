//! # neuro-storage
//!
//! Storage backends for the neuro-bitnet RAG system.
//!
//! This crate provides vector storage with similarity search capabilities:
//! - [`MemoryStorage`] - In-memory storage (fast, non-persistent)
//! - [`FileStorage`] - JSON file-based storage (persistent)
//!
//! ## Example
//!
//! ```no_run
//! use neuro_storage::{Storage, MemoryStorage};
//! use neuro_core::Document;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut storage = MemoryStorage::new();
//!     
//!     let doc = Document::new("Hello, world!")
//!         .with_embedding(vec![0.1, 0.2, 0.3]);
//!     
//!     storage.add(doc).await.unwrap();
//!     
//!     let results = storage.search(&[0.1, 0.2, 0.3], 5).await.unwrap();
//! }
//! ```

mod storage;
mod memory;
mod files;
mod similarity;
mod error;

pub use storage::Storage;
pub use memory::MemoryStorage;
pub use files::FileStorage;
pub use similarity::cosine_similarity;
pub use error::{StorageError, Result};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{Storage, MemoryStorage, FileStorage, StorageError, Result};
}
