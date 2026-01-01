//! Neuro MCP - Model Context Protocol Server
//!
//! Provides MCP interface for IDE integration (VS Code, etc.)

mod protocol;
mod server;
mod tools;

pub use protocol::*;
pub use server::McpServer;
pub use tools::*;
