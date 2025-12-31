//! Main inference model implementation for BitNet 1.58-bit models
//!
//! Uses subprocess backend (llama-cli from bitnet.cpp) for inference.

use crate::error::{InferenceError, Result};
use crate::sampler::SamplerConfig;
use crate::subprocess::SubprocessBackend;
use std::io::{self, Write};
use std::path::Path;
use tracing::info;

/// Configuration for loading inference models
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    /// Path to the GGUF model file
    pub model_path: String,
    /// Context size (default: 2048)
    pub n_ctx: u32,
    /// Number of CPU threads for generation
    pub n_threads: Option<i32>,
    /// Number of CPU threads for batch processing
    pub n_threads_batch: Option<i32>,
    /// Use memory mapping (default: true)
    pub use_mmap: bool,
    /// Use memory locking (default: false)
    pub use_mlock: bool,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            n_ctx: 2048,
            n_threads: None,
            n_threads_batch: None,
            use_mmap: true,
            use_mlock: false,
        }
    }
}

impl InferenceConfig {
    /// Create a new config with model path
    pub fn new<P: AsRef<Path>>(model_path: P) -> Self {
        Self {
            model_path: model_path.as_ref().to_string_lossy().into_owned(),
            ..Default::default()
        }
    }

    /// Set context size
    pub fn with_context_size(mut self, n_ctx: u32) -> Self {
        self.n_ctx = n_ctx;
        self
    }

    /// Set number of threads
    pub fn with_threads(mut self, threads: i32) -> Self {
        self.n_threads = Some(threads);
        self.n_threads_batch = Some(threads);
        self
    }
}

/// Options for text generation
#[derive(Debug, Clone)]
pub struct GenerateOptions {
    /// Maximum number of tokens to generate
    pub max_tokens: u32,
    /// Sampler configuration
    pub sampler: SamplerConfig,
    /// Stop sequences
    pub stop_sequences: Vec<String>,
    /// Whether to stream output
    pub stream: bool,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            sampler: SamplerConfig::default(),
            stop_sequences: vec![],
            stream: false,
        }
    }
}

impl GenerateOptions {
    /// Create with max tokens
    pub fn new(max_tokens: u32) -> Self {
        Self {
            max_tokens,
            ..Default::default()
        }
    }

    /// Enable streaming
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Set sampler config
    pub fn with_sampler(mut self, sampler: SamplerConfig) -> Self {
        self.sampler = sampler;
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.sampler.temperature = temp;
        self
    }

    /// Add stop sequence
    pub fn with_stop_sequence(mut self, stop: impl Into<String>) -> Self {
        self.stop_sequences.push(stop.into());
        self
    }
}

/// High-level inference model wrapper for BitNet
/// 
/// Uses llama-cli from bitnet.cpp as the inference backend.
pub struct InferenceModel {
    backend: SubprocessBackend,
    #[allow(dead_code)]
    config: InferenceConfig,
}

impl InferenceModel {
    /// Load a model from a GGUF file
    /// 
    /// Requires the bitnet.cpp llama-cli binary to be installed.
    /// Set BITNET_CLI_PATH environment variable or run scripts/setup_bitnet.sh.
    pub fn load(config: InferenceConfig) -> Result<Self> {
        info!("Initializing BitNet inference backend...");
        
        let mut backend = SubprocessBackend::new(&config.model_path)?
            .with_context_size(config.n_ctx);
        
        if let Some(threads) = config.n_threads {
            backend = backend.with_threads(threads);
        }

        info!("BitNet model ready: {}", config.model_path);
        Ok(Self { backend, config })
    }

    /// Load with a specific binary path
    pub fn load_with_binary<P1: AsRef<Path>, P2: AsRef<Path>>(
        binary_path: P1,
        model_path: P2,
        config: InferenceConfig,
    ) -> Result<Self> {
        info!("Initializing BitNet inference backend with custom binary...");
        
        let mut backend = SubprocessBackend::with_binary(binary_path, model_path)?
            .with_context_size(config.n_ctx);
        
        if let Some(threads) = config.n_threads {
            backend = backend.with_threads(threads);
        }

        Ok(Self { backend, config })
    }

    /// Generate text from a prompt
    pub fn generate(&self, prompt: &str, options: &GenerateOptions) -> Result<String> {
        if options.stream {
            let mut output = String::new();
            self.backend.generate_streaming(
                prompt,
                options.max_tokens,
                &options.sampler,
                |token| {
                    print!("{}", token);
                    io::stdout().flush().ok();
                    output.push_str(token);
                },
            )?;
            println!();
            
            // Apply stop sequences
            let final_output = self.apply_stop_sequences(&output, &options.stop_sequences);
            Ok(final_output)
        } else {
            let output = self.backend.generate(prompt, options.max_tokens, &options.sampler)?;
            let final_output = self.apply_stop_sequences(&output, &options.stop_sequences);
            Ok(final_output)
        }
    }

    /// Generate with a system prompt and user message
    pub fn chat(
        &self,
        system_prompt: &str,
        user_message: &str,
        options: &GenerateOptions,
    ) -> Result<String> {
        self.backend.chat(system_prompt, user_message, options.max_tokens, &options.sampler)
    }

    /// Get the backend type being used
    pub fn backend_name(&self) -> &'static str {
        "bitnet.cpp (subprocess)"
    }

    /// Check if the backend is available
    pub fn is_available() -> bool {
        SubprocessBackend::is_available()
    }

    /// Get binary version
    pub fn version(&self) -> Result<String> {
        self.backend.version()
    }

    /// Apply stop sequences to output
    fn apply_stop_sequences(&self, output: &str, stop_sequences: &[String]) -> String {
        let mut result = output.to_string();
        for stop in stop_sequences {
            if let Some(pos) = result.find(stop) {
                result.truncate(pos);
                break;
            }
        }
        result.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_config_builder() {
        let config = InferenceConfig::new("/path/to/model.gguf")
            .with_context_size(4096)
            .with_threads(8);

        assert_eq!(config.model_path, "/path/to/model.gguf");
        assert_eq!(config.n_ctx, 4096);
        assert_eq!(config.n_threads, Some(8));
    }

    #[test]
    fn test_generate_options() {
        let options = GenerateOptions::new(256)
            .with_temperature(0.8)
            .with_stop_sequence("</s>");

        assert_eq!(options.max_tokens, 256);
        assert_eq!(options.sampler.temperature, 0.8);
        assert!(options.stop_sequences.contains(&"</s>".to_string()));
    }

    #[test]
    fn test_apply_stop_sequences() {
        // Direct test of stop sequence logic
        let output = "Hello world</s>more text";
        let stop_seqs = vec!["</s>".to_string()];
        
        let mut result = output.to_string();
        for stop in &stop_seqs {
            if let Some(pos) = result.find(stop) {
                result.truncate(pos);
                break;
            }
        }
        assert_eq!(result.trim(), "Hello world");
    }
}
