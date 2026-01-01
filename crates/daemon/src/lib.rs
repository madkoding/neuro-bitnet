//! Neuro-BitNet Daemon
//!
//! Background server that provides BitNet inference via HTTP API.
//! Supports automatic translation for non-English queries.

pub mod server;
pub mod handlers;
pub mod state;

pub use server::{DaemonServer, DaemonConfig};
pub use state::AppState;
