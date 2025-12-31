//! # neuro-indexer
//!
//! Code indexing for the neuro-bitnet RAG system.
//!
//! This crate provides code analysis capabilities using tree-sitter
//! for multiple programming languages:
//!
//! - Python
//! - JavaScript
//! - TypeScript
//! - Rust
//!
//! ## Example
//!
//! ```no_run
//! use std::path::Path;
//! use neuro_indexer::{CodeIndexer, Language};
//!
//! let indexer = CodeIndexer::new();
//! let chunks = indexer.index_file(Path::new("src/main.rs"), Language::Rust).unwrap();
//!
//! for chunk in chunks {
//!     println!("{}: {}", chunk.symbol_type, chunk.name);
//! }
//! ```

mod analyzer;
mod chunk;
mod error;
mod indexer;
mod languages;

pub use analyzer::CodeAnalyzer;
pub use chunk::{CodeChunk, SymbolType};
pub use error::{IndexerError, Result};
pub use indexer::CodeIndexer;
pub use languages::Language;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{CodeAnalyzer, CodeChunk, CodeIndexer, IndexerError, Language, Result, SymbolType};
}
