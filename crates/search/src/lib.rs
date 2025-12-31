//! # neuro-search
//!
//! Web search capabilities for the neuro-bitnet RAG system.
//!
//! This crate provides web search functionality to augment RAG
//! with external knowledge when local context is insufficient.
//!
//! ## Features
//!
//! - Wikipedia search and content extraction
//! - Configurable timeouts and result limits
//! - Clean text extraction from HTML
//!
//! ## Example
//!
//! ```no_run
//! use neuro_search::{WebSearcher, WikipediaSearcher};
//!
//! #[tokio::main]
//! async fn main() {
//!     let searcher = WikipediaSearcher::new();
//!     let results = searcher.search("Rust programming language", 3).await.unwrap();
//!     
//!     for result in results {
//!         println!("{}: {}", result.title, result.snippet);
//!     }
//! }
//! ```

mod error;
mod searcher;
mod wikipedia;
mod result;

pub use error::{SearchError, Result};
pub use searcher::WebSearcher;
pub use wikipedia::WikipediaSearcher;
pub use result::WebSearchResult;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{WebSearcher, WikipediaSearcher, WebSearchResult, SearchError, Result};
}
