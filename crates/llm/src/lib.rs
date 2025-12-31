//! LLM client for neuro-bitnet.
//!
//! This crate provides a client for communicating with BitNet/llama.cpp servers
//! that expose an OpenAI-compatible API.
//!
//! # Example
//!
//! ```rust,no_run
//! use neuro_llm::{LlmClient, Message, Role};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = LlmClient::new("http://localhost:11435");
//!     
//!     let messages = vec![
//!         Message::system("You are a helpful assistant."),
//!         Message::user("What is 2 + 2?"),
//!     ];
//!     
//!     let response = client.chat(&messages, None).await?;
//!     println!("Response: {}", response);
//!     
//!     Ok(())
//! }
//! ```

mod client;
mod error;
mod types;

pub use client::{LlmClient, LlmConfig, ChatOptions, GenerateOptions};
pub use error::{LlmError, Result};
pub use types::{
    ChatRequest, ChatResponse, Choice, Message, Role, Usage,
    GenerateRequest, GenerateResponse,
};
