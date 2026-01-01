//! Main inference model implementation for BitNet 1.58-bit models
//!
//! Supports multiple backends: native FFI (fastest) and subprocess (fallback).

use crate::backend::{BackendType, InferenceBackend};
use crate::error::{InferenceError, Result};
use crate::sampler::SamplerConfig;
use crate::translation::{detect_language, build_translation_prompt, Language};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn, debug};

#[cfg(feature = "subprocess")]
use crate::subprocess::SubprocessBackend;

#[cfg(feature = "native")]
use crate::native::{NativeBackend, ModelParams, PoolConfig, ContextParams};

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
    /// Backend type to use
    pub backend: BackendType,
    /// Context pool size (for native backend)
    pub pool_size: Option<usize>,
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
            backend: BackendType::Auto,
            pool_size: None,
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

    /// Set backend type
    pub fn with_backend(mut self, backend: BackendType) -> Self {
        self.backend = backend;
        self
    }

    /// Set context pool size (for native backend)
    pub fn with_pool_size(mut self, size: usize) -> Self {
        self.pool_size = Some(size);
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
/// Supports multiple backends: native FFI (fastest) and subprocess (fallback).
pub struct InferenceModel {
    backend: Arc<dyn InferenceBackend>,
    #[allow(dead_code)]
    config: InferenceConfig,
}

impl InferenceModel {
    /// Load a model from a GGUF file
    /// 
    /// Automatically selects the best available backend:
    /// 1. Native FFI (if compiled with `native` feature)
    /// 2. Subprocess (fallback, requires llama-cli binary)
    pub fn load(config: InferenceConfig) -> Result<Self> {
        Self::load_with_backend(config.clone(), config.backend)
    }

    /// Load with a specific backend type
    pub fn load_with_backend(config: InferenceConfig, backend_type: BackendType) -> Result<Self> {
        let backend: Arc<dyn InferenceBackend> = match backend_type {
            BackendType::Native => {
                #[cfg(feature = "native")]
                {
                    Self::create_native_backend(&config)?
                }
                #[cfg(not(feature = "native"))]
                {
                    return Err(InferenceError::InvalidConfig(
                        "Native backend not available (compile with --features native)".to_string()
                    ));
                }
            }
            BackendType::Subprocess => {
                #[cfg(feature = "subprocess")]
                {
                    Self::create_subprocess_backend(&config)?
                }
                #[cfg(not(feature = "subprocess"))]
                {
                    return Err(InferenceError::InvalidConfig(
                        "Subprocess backend not available (compile with --features subprocess)".to_string()
                    ));
                }
            }
            BackendType::Auto => {
                // Try native first, fall back to subprocess
                #[cfg(feature = "native")]
                {
                    match Self::create_native_backend(&config) {
                        Ok(backend) => backend,
                        Err(e) => {
                            warn!("Native backend failed: {}, trying subprocess...", e);
                            #[cfg(feature = "subprocess")]
                            {
                                Self::create_subprocess_backend(&config)?
                            }
                            #[cfg(not(feature = "subprocess"))]
                            {
                                return Err(e);
                            }
                        }
                    }
                }
                #[cfg(not(feature = "native"))]
                {
                    #[cfg(feature = "subprocess")]
                    {
                        Self::create_subprocess_backend(&config)?
                    }
                    #[cfg(not(feature = "subprocess"))]
                    {
                        return Err(InferenceError::InvalidConfig(
                            "No backend available (compile with --features subprocess or --features native)".to_string()
                        ));
                    }
                }
            }
        };

        info!("Loaded model with backend: {}", backend.name());
        Ok(Self { backend, config })
    }

    /// Create native backend
    #[cfg(feature = "native")]
    fn create_native_backend(config: &InferenceConfig) -> Result<Arc<dyn InferenceBackend>> {
        use crate::native;
        
        if !native::is_available() {
            return Err(InferenceError::InvalidConfig(
                "Native bindings not available (bitnet-sys build failed)".to_string()
            ));
        }

        let model_params = ModelParams {
            use_mmap: config.use_mmap,
            use_mlock: config.use_mlock,
            ..Default::default()
        };

        let n_threads = config.n_threads.unwrap_or(4);
        let ctx_params = ContextParams::default()
            .with_context_size(config.n_ctx)
            .with_threads(n_threads);

        let pool_config = if let Some(pool_size) = config.pool_size {
            PoolConfig::with_sizes(pool_size.min(2), pool_size).with_context_params(ctx_params)
        } else {
            PoolConfig::default().with_context_params(ctx_params)
        };

        let backend = NativeBackend::new(&config.model_path, model_params, pool_config)?;
        Ok(Arc::new(backend))
    }

    /// Create subprocess backend
    #[cfg(feature = "subprocess")]
    fn create_subprocess_backend(config: &InferenceConfig) -> Result<Arc<dyn InferenceBackend>> {
        info!("Initializing BitNet subprocess backend...");
        
        let mut backend = SubprocessBackend::new(&config.model_path)?
            .with_context_size(config.n_ctx);
        
        if let Some(threads) = config.n_threads {
            backend = backend.with_threads(threads);
        }

        info!("BitNet model ready: {}", config.model_path);
        Ok(Arc::new(backend))
    }

    /// Load with a specific binary path (subprocess only)
    #[cfg(feature = "subprocess")]
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

        Ok(Self { 
            backend: Arc::new(backend), 
            config 
        })
    }

    /// Generate text from a prompt
    pub fn generate(&self, prompt: &str, options: &GenerateOptions) -> Result<String> {
        if options.stream {
            let mut output = String::new();
            let mut callback = |token: &str| {
                print!("{}", token);
                io::stdout().flush().ok();
                output.push_str(token);
            };
            self.backend.generate_streaming(
                prompt,
                options.max_tokens,
                &options.sampler,
                &mut callback,
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
        self.backend.name()
    }

    /// Check if the backend is available
    #[cfg(feature = "subprocess")]
    pub fn is_available() -> bool {
        SubprocessBackend::is_available()
    }

    /// Check if the backend is available
    #[cfg(not(feature = "subprocess"))]
    pub fn is_available() -> bool {
        #[cfg(feature = "native")]
        {
            crate::native::is_available()
        }
        #[cfg(not(feature = "native"))]
        {
            false
        }
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

    /// Generate text with automatic translation for non-English queries
    /// 
    /// Flow:
    /// 1. Detect language
    /// 2. If not English, translate to English using BitNet
    /// 3. Get answer in English
    /// 4. Return answer (optionally translate back)
    /// 
    /// Returns (response, original_language, was_translated)
    pub fn generate_translated(&self, prompt: &str, options: &GenerateOptions) -> Result<(String, Language, bool)> {
        let lang = detect_language(prompt);
        
        if lang == Language::English {
            let response = self.generate(prompt, options)?;
            return Ok((response, Language::English, false));
        }

        // Step 1: Translate question to English using BitNet
        let translate_prompt = build_translation_prompt(prompt);
        let translate_options = GenerateOptions::new(100)
            .with_temperature(0.1);  // Low temp for accurate translation
        
        let english_question = self.generate(&translate_prompt, &translate_options)?;
        let english_question = english_question.trim().to_string();
        
        debug!("Translated '{}' -> '{}'", prompt, english_question);

        // Step 2: Get answer in English (more accurate for factual questions)
        let response = self.generate(&english_question, options)?;
        
        Ok((response, lang, true))
    }

    /// Chat with automatic translation for non-English queries
    pub fn chat_translated(
        &self,
        system_prompt: &str,
        user_message: &str,
        options: &GenerateOptions,
    ) -> Result<(String, Language, bool)> {
        let lang = detect_language(user_message);
        
        if lang == Language::English {
            let response = self.chat(system_prompt, user_message, options)?;
            return Ok((response, Language::English, false));
        }

        // Translate user message to English
        let translate_prompt = build_translation_prompt(user_message);
        let translate_options = GenerateOptions::new(100)
            .with_temperature(0.1);
        
        let english_message = self.generate(&translate_prompt, &translate_options)?;
        let english_message = english_message.trim().to_string();
        
        debug!("Translated '{}' -> '{}'", user_message, english_message);

        // Get response with translated message
        let response = self.chat(system_prompt, &english_message, options)?;
        
        Ok((response, lang, true))
    }

    /// Check if a query is in Spanish
    pub fn is_spanish(&self, query: &str) -> bool {
        detect_language(query) == Language::Spanish
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
