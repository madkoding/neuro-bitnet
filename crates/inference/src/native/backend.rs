//! Native backend implementation
//!
//! High-performance inference using direct FFI bindings to bitnet.cpp.

use crate::backend::{InferenceBackend, TokenCallback};
use crate::error::{InferenceError, Result};
use crate::native::{
    ContextPool, LlamaBatch, LlamaModel, LlamaSampler, ModelParams, PoolConfig, ContextParams,
};
use crate::sampler::SamplerConfig;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// Native FFI backend for bitnet.cpp
///
/// Uses direct bindings to bitnet.cpp for maximum performance.
/// Maintains a pool of contexts for concurrent request handling.
pub struct NativeBackend {
    /// Model shared across all contexts
    model: Arc<LlamaModel>,
    /// Pool of contexts for concurrent requests
    pool: Arc<ContextPool>,
}

impl NativeBackend {
    /// Create a new native backend
    ///
    /// # Arguments
    /// * `model_path` - Path to the GGUF model file
    /// * `model_params` - Model loading parameters
    /// * `pool_config` - Context pool configuration
    pub fn new<P: AsRef<Path>>(
        model_path: P,
        model_params: ModelParams,
        pool_config: PoolConfig,
    ) -> Result<Self> {
        info!("Initializing native BitNet backend...");
        
        // Initialize llama.cpp backend
        unsafe {
            bitnet_sys::llama_backend_init();
        }

        // Load model
        let model = LlamaModel::load(model_path, &model_params)?;
        info!(
            "Model loaded: vocab_size={}, n_ctx_train={}, n_embd={}",
            model.vocab_size(),
            model.n_ctx_train(),
            model.n_embd()
        );

        // Create context pool
        let pool = ContextPool::new(Arc::clone(&model), pool_config)?;
        info!("Context pool initialized: {} contexts", pool.size());

        Ok(Self { model, pool })
    }

    /// Create with default parameters
    pub fn with_defaults<P: AsRef<Path>>(model_path: P, n_ctx: u32, n_threads: i32) -> Result<Self> {
        let model_params = ModelParams::default();
        let pool_config = PoolConfig::default().with_context_params(
            ContextParams::default()
                .with_context_size(n_ctx)
                .with_threads(n_threads),
        );
        Self::new(model_path, model_params, pool_config)
    }

    /// Get the context pool
    pub fn pool(&self) -> &Arc<ContextPool> {
        &self.pool
    }

    /// Generate tokens with full control
    fn generate_tokens(
        &self,
        tokens: &[i32],
        max_new_tokens: u32,
        sampler_config: &SamplerConfig,
        mut on_token: Option<&mut dyn FnMut(&str)>,
    ) -> Result<String> {
        // Acquire context from pool
        let mut ctx = self.pool.acquire()?;
        
        // Create batch for prompt
        let n_ctx = ctx.n_ctx() as usize;
        let mut batch = LlamaBatch::new(n_ctx, 1)?;
        
        // Add prompt tokens
        batch.add_sequence(tokens, 0, 0, true)?;
        
        // Process prompt
        ctx.decode(&mut batch)?;
        
        // Create sampler
        let mut sampler = LlamaSampler::from_config(sampler_config, self.model.vocab_size())?;
        
        // Generate tokens
        let mut output = String::with_capacity(max_new_tokens as usize * 4); // Estimate 4 chars per token
        let mut n_decoded = tokens.len();
        
        for _ in 0..max_new_tokens {
            // Sample next token
            let new_token = sampler.sample(&ctx, -1);
            sampler.accept(new_token);
            
            // Check for end of generation
            if self.model.is_eog_token(new_token) {
                debug!("End of generation token");
                break;
            }
            
            // Decode token to string
            let piece = self.model.token_to_str(new_token)?;
            
            // Stream callback
            if let Some(ref mut callback) = on_token {
                callback(&piece);
            }
            
            output.push_str(&piece);
            
            // Prepare next batch
            batch.clear();
            batch.add(new_token, n_decoded as i32, &[0], true)?;
            ctx.decode(&mut batch)?;
            
            n_decoded += 1;
            
            // Check context limit
            if n_decoded >= n_ctx - 4 {
                debug!("Approaching context limit, stopping");
                break;
            }
        }
        
        Ok(output)
    }
}

impl InferenceBackend for NativeBackend {
    fn generate(&self, prompt: &str, max_tokens: u32, sampler: &SamplerConfig) -> Result<String> {
        // Tokenize prompt
        let tokens = self.model.tokenize(prompt, true, true)?;
        debug!("Tokenized {} chars -> {} tokens", prompt.len(), tokens.len());
        
        // Generate
        self.generate_tokens(&tokens, max_tokens, sampler, None)
    }

    fn generate_streaming(
        &self,
        prompt: &str,
        max_tokens: u32,
        sampler: &SamplerConfig,
        on_token: TokenCallback<'_>,
    ) -> Result<String> {
        // Tokenize prompt
        let tokens = self.model.tokenize(prompt, true, true)?;
        debug!("Tokenized {} chars -> {} tokens", prompt.len(), tokens.len());
        
        // We need to convert the reference to a mutable one
        let mut callback = on_token;
        self.generate_tokens(&tokens, max_tokens, sampler, Some(&mut callback))
    }

    fn chat(
        &self,
        system_prompt: &str,
        user_message: &str,
        max_tokens: u32,
        sampler: &SamplerConfig,
    ) -> Result<String> {
        // Format as chat prompt
        let prompt = format!(
            "<|system|>\n{}</s>\n<|user|>\n{}</s>\n<|assistant|>\n",
            system_prompt, user_message
        );
        
        self.generate(&prompt, max_tokens, sampler)
    }

    fn name(&self) -> &'static str {
        bitnet_sys::backend_type()
    }

    fn is_ready(&self) -> bool {
        self.pool.available() > 0 || self.pool.size() < 4 // Can grow
    }

    fn version(&self) -> Result<String> {
        Ok(format!(
            "Native FFI ({}) - vocab:{}, embd:{}",
            self.name(),
            self.model.vocab_size(),
            self.model.n_embd()
        ))
    }
}

impl Drop for NativeBackend {
    fn drop(&mut self) {
        info!("Shutting down native backend...");
        // llama_backend_free is called automatically
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_backend_type() {
        let backend_type = bitnet_sys::backend_type();
        assert!(!backend_type.is_empty());
    }
}
