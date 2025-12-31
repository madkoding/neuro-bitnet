//! # neuro-server
//!
//! HTTP API server for the neuro-bitnet RAG system.
//!
//! This crate provides a REST API using Axum for:
//! - Query classification and execution
//! - Document management (add, search, list)
//! - Health checks and statistics
//!
//! ## Endpoints
//!
//! - `GET /health` - Health check
//! - `GET /stats` - Server statistics
//! - `POST /query` - Intelligent query (classify + execute)
//! - `POST /classify` - Classify query without execution
//! - `POST /add` - Add document
//! - `POST /search` - Similarity search
//! - `GET /documents` - List documents
//!
//! ## Example
//!
//! ```no_run
//! use neuro_server::{Server, ServerConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = ServerConfig::default();
//!     let server = Server::new(config).await.unwrap();
//!     server.run().await.unwrap();
//! }
//! ```

mod config;
mod error;
mod handlers;
mod routes;
mod state;
mod server;

pub use config::ServerConfig;
pub use error::{ServerError, Result};
pub use server::Server;
pub use state::AppState;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{Server, ServerConfig, ServerError, Result, AppState};
}
