//! LlamaContext wrapper for bitnet.cpp
//!
//! Safe Rust wrapper around the llama_context FFI type.

use crate::error::{InferenceError, Result};
use crate::native::LlamaModel;
use bitnet_sys::*;
use std::ptr::NonNull;
use std::sync::Arc;

/// Safe wrapper around llama_context
///
/// Holds the inference state for a model. Each context can process
/// one sequence at a time, but multiple contexts can share a model.
pub struct LlamaContext {
    ptr: NonNull<llama_context>,
    model: Arc<LlamaModel>,
    n_ctx: u32,
}

// SAFETY: Context is not thread-safe, but we enforce single-threaded access via pool
unsafe impl Send for LlamaContext {}

/// Context creation parameters
#[derive(Debug, Clone)]
pub struct ContextParams {
    /// Context size (number of tokens that can be processed)
    pub n_ctx: u32,
    /// Batch size for prompt processing
    pub n_batch: u32,
    /// Physical batch size (for VRAM optimization)
    pub n_ubatch: u32,
    /// Number of threads for generation
    pub n_threads: i32,
    /// Number of threads for batch processing
    pub n_threads_batch: i32,
    /// RoPE frequency base
    pub rope_freq_base: f32,
    /// RoPE frequency scale
    pub rope_freq_scale: f32,
    /// Seed for random number generation (0 = random)
    pub seed: u32,
    /// Enable embeddings mode
    pub embeddings: bool,
    /// Use flash attention (faster but may use more memory)
    pub flash_attn: bool,
}

impl Default for ContextParams {
    fn default() -> Self {
        Self {
            n_ctx: 2048,
            n_batch: 512,
            n_ubatch: 512,
            n_threads: 4,
            n_threads_batch: 4,
            rope_freq_base: 0.0,  // Use model default
            rope_freq_scale: 0.0, // Use model default
            seed: 0,              // Random
            embeddings: false,
            flash_attn: false,
        }
    }
}

impl ContextParams {
    /// Create params with specific context size
    pub fn with_context_size(mut self, n_ctx: u32) -> Self {
        self.n_ctx = n_ctx;
        self
    }

    /// Set number of threads
    pub fn with_threads(mut self, threads: i32) -> Self {
        self.n_threads = threads;
        self.n_threads_batch = threads;
        self
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }
}

impl LlamaContext {
    /// Create a new context for a model
    ///
    /// # Arguments
    /// * `model` - The model to create a context for
    /// * `params` - Context parameters
    pub fn new(model: Arc<LlamaModel>, params: &ContextParams) -> Result<Self> {
        let mut ctx_params = unsafe { llama_context_default_params() };
        
        ctx_params.n_ctx = params.n_ctx;
        ctx_params.n_batch = params.n_batch;
        ctx_params.n_ubatch = params.n_ubatch;
        ctx_params.n_threads = params.n_threads as u32;
        ctx_params.n_threads_batch = params.n_threads_batch as u32;
        ctx_params.seed = params.seed;
        ctx_params.embeddings = params.embeddings;
        ctx_params.flash_attn = params.flash_attn;

        if params.rope_freq_base > 0.0 {
            ctx_params.rope_freq_base = params.rope_freq_base;
        }
        if params.rope_freq_scale > 0.0 {
            ctx_params.rope_freq_scale = params.rope_freq_scale;
        }

        let ptr = unsafe { llama_new_context_with_model(model.as_ptr(), ctx_params) };

        let ptr = NonNull::new(ptr).ok_or_else(|| {
            InferenceError::Context("Failed to create context".to_string())
        })?;

        Ok(Self {
            ptr,
            model,
            n_ctx: params.n_ctx,
        })
    }

    /// Get the raw pointer (for FFI calls)
    pub(crate) fn as_ptr(&self) -> *mut llama_context {
        self.ptr.as_ptr()
    }

    /// Get the associated model
    pub fn model(&self) -> &Arc<LlamaModel> {
        &self.model
    }

    /// Get the context size
    pub fn n_ctx(&self) -> u32 {
        self.n_ctx
    }

    /// Decode a batch of tokens
    ///
    /// This is the main inference step - processes tokens and updates KV cache.
    pub fn decode(&mut self, batch: &mut super::LlamaBatch) -> Result<()> {
        let ret = unsafe { llama_decode(self.ptr.as_ptr(), batch.as_raw()) };

        if ret != 0 {
            return Err(InferenceError::Decode(format!(
                "llama_decode failed with code {}",
                ret
            )));
        }

        Ok(())
    }

    /// Get logits for the last token
    ///
    /// Returns a slice of logits with length equal to vocab size.
    pub fn get_logits(&self) -> &[f32] {
        unsafe {
            let ptr = llama_get_logits(self.ptr.as_ptr());
            let vocab_size = self.model.vocab_size() as usize;
            std::slice::from_raw_parts(ptr, vocab_size)
        }
    }

    /// Get logits for a specific token index in the batch
    pub fn get_logits_ith(&self, i: i32) -> &[f32] {
        unsafe {
            let ptr = llama_get_logits_ith(self.ptr.as_ptr(), i);
            let vocab_size = self.model.vocab_size() as usize;
            std::slice::from_raw_parts(ptr, vocab_size)
        }
    }

    /// Clear the KV cache
    pub fn kv_cache_clear(&mut self) {
        unsafe {
            llama_kv_cache_clear(self.ptr.as_ptr());
        }
    }

    /// Get the number of tokens in the KV cache
    pub fn kv_cache_used_cells(&self) -> i32 {
        unsafe { llama_get_kv_cache_used_cells(self.ptr.as_ptr()) }
    }

    /// Remove tokens from the KV cache
    ///
    /// # Arguments
    /// * `seq_id` - Sequence ID (-1 for all sequences)
    /// * `p0` - Start position (inclusive)
    /// * `p1` - End position (exclusive, -1 for end)
    pub fn kv_cache_seq_rm(&mut self, seq_id: i32, p0: i32, p1: i32) -> bool {
        unsafe { llama_kv_cache_seq_rm(self.ptr.as_ptr(), seq_id, p0, p1) }
    }

    /// Shift KV cache positions
    pub fn kv_cache_seq_add(&mut self, seq_id: i32, p0: i32, p1: i32, delta: i32) {
        unsafe {
            llama_kv_cache_seq_add(self.ptr.as_ptr(), seq_id, p0, p1, delta);
        }
    }

    /// Get embeddings for the last token (if embeddings mode is enabled)
    pub fn get_embeddings(&self) -> Option<&[f32]> {
        unsafe {
            let ptr = llama_get_embeddings(self.ptr.as_ptr());
            if ptr.is_null() {
                return None;
            }
            let n_embd = self.model.n_embd() as usize;
            Some(std::slice::from_raw_parts(ptr, n_embd))
        }
    }
}

impl Drop for LlamaContext {
    fn drop(&mut self) {
        unsafe {
            llama_free(self.ptr.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_params_default() {
        let params = ContextParams::default();
        assert_eq!(params.n_ctx, 2048);
        assert_eq!(params.n_threads, 4);
    }

    #[test]
    fn test_context_params_builder() {
        let params = ContextParams::default()
            .with_context_size(4096)
            .with_threads(8)
            .with_seed(42);
        
        assert_eq!(params.n_ctx, 4096);
        assert_eq!(params.n_threads, 8);
        assert_eq!(params.seed, 42);
    }
}
