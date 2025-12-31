//! LLM client implementation.

use std::time::Duration;
use reqwest::Client;
use tracing::{debug, info, warn};

use crate::error::{LlmError, Result};
use crate::types::{
    ChatRequest, ChatResponse, GenerateRequest, GenerateResponse, Message,
};

/// Default timeout for requests in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Configuration for the LLM client.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Base URL of the LLM server
    pub base_url: String,
    /// Model name to use
    pub model: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Default max tokens
    pub max_tokens: u32,
    /// Default temperature
    pub temperature: f32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11435".to_string(),
            model: "bitnet".to_string(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            max_tokens: 512,
            temperature: 0.7,
        }
    }
}

impl LlmConfig {
    /// Create a new config with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            ..Default::default()
        }
    }
}

/// Client for communicating with BitNet/llama.cpp servers.
///
/// Supports both OpenAI-compatible API and native llama.cpp API.
#[derive(Debug, Clone)]
pub struct LlmClient {
    client: Client,
    config: LlmConfig,
}

impl LlmClient {
    /// Create a new LLM client with default settings.
    pub fn new(base_url: impl Into<String>) -> Self {
        let config = LlmConfig::new(base_url);
        Self::with_config(config)
    }

    /// Create a new LLM client with custom configuration.
    pub fn with_config(config: LlmConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Check if the server is available.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.config.base_url);
        debug!("Health check: {}", url);

        match self.client.get(&url).timeout(Duration::from_secs(5)).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Wait for the server to become available.
    pub async fn wait_for_server(&self, max_wait_secs: u64) -> Result<()> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(max_wait_secs);

        info!("Waiting for LLM server at {}...", self.config.base_url);

        while start.elapsed() < timeout {
            if self.health_check().await? {
                info!("LLM server is ready");
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Err(LlmError::ServerUnavailable {
            url: self.config.base_url.clone(),
        })
    }

    /// Send a chat completion request (OpenAI-compatible API).
    pub async fn chat(&self, messages: &[Message], options: Option<ChatOptions>) -> Result<String> {
        let options = options.unwrap_or_default();
        
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            max_tokens: Some(options.max_tokens.unwrap_or(self.config.max_tokens)),
            temperature: Some(options.temperature.unwrap_or(self.config.temperature)),
            top_p: options.top_p,
            stream: Some(false),
            stop: options.stop,
        };

        let url = format!("{}/v1/chat/completions", self.config.base_url);
        debug!("Chat request to {}", url);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(LlmError::ServerError { status, message });
        }

        let chat_response: ChatResponse = response.json().await?;
        
        chat_response
            .content()
            .map(|s| s.to_string())
            .ok_or(LlmError::EmptyResponse)
    }

    /// Generate text using native llama.cpp API.
    pub async fn generate(&self, prompt: &str, options: Option<GenerateOptions>) -> Result<GenerateResponse> {
        let options = options.unwrap_or_default();

        let request = GenerateRequest {
            prompt: prompt.to_string(),
            n_predict: Some(options.max_tokens.unwrap_or(self.config.max_tokens)),
            temperature: Some(options.temperature.unwrap_or(self.config.temperature)),
            top_p: options.top_p,
            top_k: options.top_k,
            stop: options.stop,
            stream: Some(false),
        };

        let url = format!("{}/completion", self.config.base_url);
        debug!("Generate request to {}", url);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(LlmError::ServerError { status, message });
        }

        let gen_response: GenerateResponse = response.json().await?;
        Ok(gen_response)
    }

    /// Simple question-answering with context.
    pub async fn ask_with_context(
        &self,
        question: &str,
        context: &str,
        system_prompt: Option<&str>,
    ) -> Result<String> {
        let system = system_prompt.unwrap_or(
            "You are a helpful assistant. Use the provided context to answer questions. \
             If the information is not in the context, say so. \
             Respond in the same language as the question."
        );

        let user_prompt = format!(
            "Context:\n{}\n\nQuestion: {}\n\nAnswer:",
            context, question
        );

        let messages = vec![
            Message::system(system),
            Message::user(user_prompt),
        ];

        self.chat(&messages, None).await
    }

    /// Simple question-answering without context.
    pub async fn ask(&self, question: &str) -> Result<String> {
        let messages = vec![
            Message::system("You are a helpful assistant. Be concise and accurate."),
            Message::user(question),
        ];

        self.chat(&messages, None).await
    }
}

/// Options for chat completion.
#[derive(Debug, Clone, Default)]
pub struct ChatOptions {
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature (0.0 = deterministic, 1.0 = creative)
    pub temperature: Option<f32>,
    /// Top-p sampling
    pub top_p: Option<f32>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
}

impl ChatOptions {
    /// Create new chat options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set max tokens.
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set temperature.
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

/// Options for text generation.
#[derive(Debug, Clone, Default)]
pub struct GenerateOptions {
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature
    pub temperature: Option<f32>,
    /// Top-p sampling
    pub top_p: Option<f32>,
    /// Top-k sampling
    pub top_k: Option<u32>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = LlmConfig::default();
        assert_eq!(config.base_url, "http://localhost:11435");
        assert_eq!(config.model, "bitnet");
    }

    #[test]
    fn test_client_creation() {
        let client = LlmClient::new("http://localhost:8080");
        assert_eq!(client.base_url(), "http://localhost:8080");
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.content, "Hello");
    }
}
