//! # neuro-cli
//!
//! Command-line interface for the neuro-bitnet RAG system.
//!
//! ## Commands
//!
//! - `serve` - Start the HTTP server
//! - `index` - Index files or directories
//! - `query` - Execute a query
//! - `stats` - Show storage statistics
//! - `embed` - Generate embeddings for text
//!
//! ## Usage
//!
//! ```bash
//! # Start server
//! neuro serve --port 8080
//!
//! # Index a directory
//! neuro index ./src --recursive
//!
//! # Query the system
//! neuro query "What is Rust?"
//!
//! # Show statistics
//! neuro stats
//! ```

pub mod cli;
pub mod commands;

pub use cli::Cli;
