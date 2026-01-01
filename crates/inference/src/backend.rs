//! Backend trait for inference engines
//!
//! Defines a unified interface for different inference backends:
//! - `NativeBackend` - Direct FFI bindings to bitnet.cpp (fastest)
//! - `SubprocessBackend` - Calls llama-cli binary (fallback)

use crate::error::Result;
use crate::sampler::SamplerConfig;

/// Token callback type for streaming
pub type TokenCallback<'a> = &'a mut dyn FnMut(&str);

/// Unified interface for inference backends
///
/// Both native FFI and subprocess backends implement this trait,
/// allowing seamless switching between them.
pub trait InferenceBackend: Send + Sync {
    /// Generate text from a prompt
    ///
    /// # Arguments
    /// * `prompt` - The input prompt text
    /// * `max_tokens` - Maximum number of tokens to generate
    /// * `sampler` - Sampling configuration (temperature, top_k, etc.)
    ///
    /// # Returns
    /// Generated text string
    fn generate(&self, prompt: &str, max_tokens: u32, sampler: &SamplerConfig) -> Result<String>;

    /// Generate text with streaming callback
    ///
    /// Calls `on_token` for each generated token, allowing real-time output.
    ///
    /// # Arguments
    /// * `prompt` - The input prompt text
    /// * `max_tokens` - Maximum number of tokens to generate
    /// * `sampler` - Sampling configuration
    /// * `on_token` - Callback invoked for each token
    fn generate_streaming(
        &self,
        prompt: &str,
        max_tokens: u32,
        sampler: &SamplerConfig,
        on_token: TokenCallback<'_>,
    ) -> Result<String>;

    /// Chat-style generation with system and user prompts
    ///
    /// Formats the input as a chat conversation and generates a response.
    fn chat(
        &self,
        system_prompt: &str,
        user_message: &str,
        max_tokens: u32,
        sampler: &SamplerConfig,
    ) -> Result<String>;

    /// Get the backend type name
    fn name(&self) -> &'static str;

    /// Check if the backend is operational
    fn is_ready(&self) -> bool;

    /// Get version information (if available)
    fn version(&self) -> Result<String> {
        Ok(self.name().to_string())
    }
}

/// Type of inference backend to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendType {
    /// Native FFI bindings to bitnet.cpp (preferred, fastest)
    Native,
    /// Subprocess backend calling llama-cli binary (fallback)
    Subprocess,
    /// Auto-detect: try native first, fall back to subprocess
    #[default]
    Auto,
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendType::Native => write!(f, "native"),
            BackendType::Subprocess => write!(f, "subprocess"),
            BackendType::Auto => write!(f, "auto"),
        }
    }
}

impl std::str::FromStr for BackendType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "native" | "ffi" => Ok(BackendType::Native),
            "subprocess" | "cli" | "process" => Ok(BackendType::Subprocess),
            "auto" | "default" => Ok(BackendType::Auto),
            _ => Err(format!("Unknown backend type: {}", s)),
        }
    }
}
