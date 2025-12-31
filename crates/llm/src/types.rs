//! Types for OpenAI-compatible API requests and responses.

use serde::{Deserialize, Serialize};

/// Role in a chat conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System message (instructions)
    System,
    /// User message (input)
    User,
    /// Assistant message (model response)
    Assistant,
}

/// A message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message author
    pub role: Role,
    /// The content of the message
    pub content: String,
}

impl Message {
    /// Create a new message.
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Role::System, content)
    }

    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Role::User, content)
    }

    /// Create an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(Role::Assistant, content)
    }
}

/// Request for chat completion (OpenAI-compatible).
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    /// Model identifier
    pub model: String,
    /// List of messages in the conversation
    pub messages: Vec<Message>,
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Temperature for sampling (0.0 = deterministic, 1.0 = creative)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Top-p sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

impl ChatRequest {
    /// Create a new chat request.
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            model: "bitnet".to_string(),
            messages,
            max_tokens: Some(512),
            temperature: Some(0.7),
            top_p: None,
            stream: Some(false),
            stop: None,
        }
    }

    /// Set the model name.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
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

    /// Set top-p.
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Enable streaming.
    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }
}

/// Response from chat completion.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    /// Unique identifier
    pub id: Option<String>,
    /// Object type
    pub object: Option<String>,
    /// Unix timestamp
    pub created: Option<u64>,
    /// Model used
    pub model: Option<String>,
    /// Generated choices
    pub choices: Vec<Choice>,
    /// Token usage statistics
    pub usage: Option<Usage>,
}

impl ChatResponse {
    /// Get the content of the first choice.
    pub fn content(&self) -> Option<&str> {
        self.choices.first().and_then(|c| c.message.as_ref()).map(|m| m.content.as_str())
    }
}

/// A choice in the response.
#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    /// Index of this choice
    pub index: Option<u32>,
    /// The generated message
    pub message: Option<Message>,
    /// Delta for streaming
    pub delta: Option<Message>,
    /// Reason for finishing
    pub finish_reason: Option<String>,
}

/// Token usage statistics.
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    /// Tokens in the prompt
    pub prompt_tokens: u32,
    /// Tokens in the completion
    pub completion_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}

/// Request for text generation (llama.cpp native).
#[derive(Debug, Clone, Serialize)]
pub struct GenerateRequest {
    /// The prompt to generate from
    pub prompt: String,
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n_predict: Option<u32>,
    /// Temperature for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Top-p sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Top-k sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// Whether to stream
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl GenerateRequest {
    /// Create a new generate request.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            n_predict: Some(512),
            temperature: Some(0.7),
            top_p: None,
            top_k: None,
            stop: None,
            stream: Some(false),
        }
    }

    /// Set max tokens.
    pub fn n_predict(mut self, n: u32) -> Self {
        self.n_predict = Some(n);
        self
    }

    /// Set temperature.
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }
}

/// Response from text generation (llama.cpp native).
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateResponse {
    /// Generated text
    pub content: String,
    /// Whether generation stopped
    pub stop: Option<bool>,
    /// Number of tokens generated
    pub tokens_predicted: Option<u32>,
    /// Number of tokens evaluated
    pub tokens_evaluated: Option<u32>,
    /// Generation time in milliseconds
    pub generation_time_ms: Option<f64>,
}
