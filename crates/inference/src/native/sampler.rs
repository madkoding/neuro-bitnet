//! LlamaSampler wrapper for bitnet.cpp
//!
//! Safe Rust wrapper around the llama_sampler FFI type.

use crate::error::{InferenceError, Result};
use crate::sampler::SamplerConfig;
use bitnet_sys::*;
use std::ptr::NonNull;

/// Safe wrapper around llama_sampler
///
/// Handles token sampling with various strategies (temperature, top-k, top-p, etc.)
pub struct LlamaSampler {
    ptr: NonNull<llama_sampler>,
}

// SAFETY: Sampler state is independent per instance
unsafe impl Send for LlamaSampler {}

impl LlamaSampler {
    /// Create a new sampler chain from configuration
    ///
    /// Sets up a chain of samplers in the recommended order:
    /// 1. Repetition penalty
    /// 2. Top-K
    /// 3. Top-P (nucleus)
    /// 4. Min-P
    /// 5. Temperature
    /// 6. Distribution sampling
    pub fn from_config(config: &SamplerConfig, vocab_size: i32) -> Result<Self> {
        // Initialize the sampler chain
        let params = llama_sampler_chain_params {
            no_perf: false,
        };
        
        let chain = unsafe { llama_sampler_chain_init(params) };
        
        if chain.is_null() {
            return Err(InferenceError::Sampling("Failed to create sampler chain".to_string()));
        }

        // Add repetition penalty sampler
        if config.repeat_penalty != 1.0 {
            let repeat_sampler = unsafe {
                llama_sampler_init_penalties(
                    vocab_size,
                    llama_token_eos(std::ptr::null_mut()), // Will be ignored if penalty_last_n is 0
                    llama_token_nl(std::ptr::null_mut()),
                    config.repeat_last_n,
                    config.repeat_penalty,
                    0.0,  // frequency_penalty
                    0.0,  // presence_penalty
                    false, // penalize_nl
                    false, // ignore_eos
                )
            };
            if !repeat_sampler.is_null() {
                unsafe { llama_sampler_chain_add(chain, repeat_sampler) };
            }
        }

        // Add top-k sampler
        if config.top_k > 0 {
            let top_k_sampler = unsafe { llama_sampler_init_top_k(config.top_k) };
            if !top_k_sampler.is_null() {
                unsafe { llama_sampler_chain_add(chain, top_k_sampler) };
            }
        }

        // Add top-p (nucleus) sampler
        if config.top_p < 1.0 {
            let top_p_sampler = unsafe { llama_sampler_init_top_p(config.top_p, 1) };
            if !top_p_sampler.is_null() {
                unsafe { llama_sampler_chain_add(chain, top_p_sampler) };
            }
        }

        // Add min-p sampler
        if config.min_p > 0.0 {
            let min_p_sampler = unsafe { llama_sampler_init_min_p(config.min_p, 1) };
            if !min_p_sampler.is_null() {
                unsafe { llama_sampler_chain_add(chain, min_p_sampler) };
            }
        }

        // Add temperature sampler
        if config.temperature > 0.0 {
            let temp_sampler = unsafe { llama_sampler_init_temp(config.temperature) };
            if !temp_sampler.is_null() {
                unsafe { llama_sampler_chain_add(chain, temp_sampler) };
            }
        }

        // Add distribution sampler (final step)
        let seed = if config.seed == 0 {
            rand::random::<u32>()
        } else {
            config.seed as u32
        };
        
        let dist_sampler = unsafe { llama_sampler_init_dist(seed) };
        if !dist_sampler.is_null() {
            unsafe { llama_sampler_chain_add(chain, dist_sampler) };
        }

        let ptr = NonNull::new(chain).ok_or_else(|| {
            InferenceError::Sampling("Sampler chain creation failed".to_string())
        })?;

        Ok(Self { ptr })
    }

    /// Create a greedy sampler (always picks highest probability token)
    pub fn greedy() -> Result<Self> {
        let params = llama_sampler_chain_params { no_perf: false };
        let chain = unsafe { llama_sampler_chain_init(params) };
        
        if chain.is_null() {
            return Err(InferenceError::Sampling("Failed to create greedy sampler".to_string()));
        }

        let greedy = unsafe { llama_sampler_init_greedy() };
        if !greedy.is_null() {
            unsafe { llama_sampler_chain_add(chain, greedy) };
        }

        let ptr = NonNull::new(chain).ok_or_else(|| {
            InferenceError::Sampling("Greedy sampler creation failed".to_string())
        })?;

        Ok(Self { ptr })
    }

    /// Sample the next token from logits
    ///
    /// # Arguments
    /// * `ctx` - The context to sample from (uses last logits)
    /// * `idx` - Index of the token in the batch (-1 for last)
    pub fn sample(&mut self, ctx: &super::LlamaContext, idx: i32) -> llama_token {
        unsafe { llama_sampler_sample(self.ptr.as_ptr(), ctx.as_ptr(), idx) }
    }

    /// Reset the sampler state
    pub fn reset(&mut self) {
        unsafe {
            llama_sampler_reset(self.ptr.as_ptr());
        }
    }

    /// Accept a token (update repetition penalty state)
    pub fn accept(&mut self, token: llama_token) {
        unsafe {
            llama_sampler_accept(self.ptr.as_ptr(), token);
        }
    }
}

impl Drop for LlamaSampler {
    fn drop(&mut self) {
        unsafe {
            llama_sampler_free(self.ptr.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampler_from_default_config() {
        // This test requires native bindings to be available
        if !bitnet_sys::is_available() {
            return;
        }
        
        let config = SamplerConfig::default();
        let result = LlamaSampler::from_config(&config, 32000);
        assert!(result.is_ok());
    }
}
