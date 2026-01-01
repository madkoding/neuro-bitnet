//! LlamaModel wrapper for bitnet.cpp
//!
//! Safe Rust wrapper around the llama_model FFI type.

use crate::error::{InferenceError, Result};
use bitnet_sys::*;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr::NonNull;
use std::sync::Arc;

/// Safe wrapper around llama_model
///
/// Handles model loading, tokenization, and cleanup.
/// Thread-safe via Arc for sharing across contexts.
pub struct LlamaModel {
    ptr: NonNull<llama_model>,
    vocab_size: i32,
}

// SAFETY: llama_model is thread-safe for read operations (tokenization, etc.)
unsafe impl Send for LlamaModel {}
unsafe impl Sync for LlamaModel {}

/// Model loading parameters
#[derive(Debug, Clone)]
pub struct ModelParams {
    /// Number of GPU layers to offload (0 = CPU only)
    pub n_gpu_layers: i32,
    /// Use memory mapping for model file
    pub use_mmap: bool,
    /// Lock model in memory (prevent swapping)
    pub use_mlock: bool,
    /// Check model tensors for NaN/Inf
    pub check_tensors: bool,
}

impl Default for ModelParams {
    fn default() -> Self {
        Self {
            n_gpu_layers: 0, // CPU only by default for BitNet
            use_mmap: true,
            use_mlock: false,
            check_tensors: false,
        }
    }
}

impl LlamaModel {
    /// Load a model from a GGUF file
    ///
    /// # Arguments
    /// * `path` - Path to the GGUF model file
    /// * `params` - Model loading parameters
    ///
    /// # Returns
    /// Arc-wrapped model for sharing across contexts
    pub fn load<P: AsRef<Path>>(path: P, params: &ModelParams) -> Result<Arc<Self>> {
        let path_str = path.as_ref().to_string_lossy();
        let c_path = CString::new(path_str.as_ref())
            .map_err(|_| InferenceError::InvalidConfig("Invalid path encoding".to_string()))?;

        // Initialize model params
        let mut model_params = unsafe { llama_model_default_params() };
        model_params.n_gpu_layers = params.n_gpu_layers;
        model_params.use_mmap = params.use_mmap;
        model_params.use_mlock = params.use_mlock;
        model_params.check_tensors = params.check_tensors;

        // Load the model
        let ptr = unsafe { llama_load_model_from_file(c_path.as_ptr(), model_params) };

        let ptr = NonNull::new(ptr).ok_or_else(|| {
            InferenceError::LoadModel(format!("Failed to load model from: {}", path_str))
        })?;

        // Get vocab size
        let vocab_size = unsafe { llama_n_vocab(ptr.as_ptr()) };

        Ok(Arc::new(Self { ptr, vocab_size }))
    }

    /// Get the raw pointer (for FFI calls)
    pub(crate) fn as_ptr(&self) -> *mut llama_model {
        self.ptr.as_ptr()
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> i32 {
        self.vocab_size
    }

    /// Get context size the model was trained with
    pub fn n_ctx_train(&self) -> i32 {
        unsafe { llama_n_ctx_train(self.ptr.as_ptr()) }
    }

    /// Get embedding dimension
    pub fn n_embd(&self) -> i32 {
        unsafe { llama_n_embd(self.ptr.as_ptr()) }
    }

    /// Tokenize a string
    ///
    /// # Arguments
    /// * `text` - Text to tokenize
    /// * `add_bos` - Whether to add beginning-of-sequence token
    /// * `special` - Whether to parse special tokens
    ///
    /// # Returns
    /// Vector of token IDs
    pub fn tokenize(&self, text: &str, add_bos: bool, special: bool) -> Result<Vec<llama_token>> {
        let c_text = CString::new(text)
            .map_err(|_| InferenceError::Tokenize("Invalid text encoding".to_string()))?;

        // First call to get required size
        let n_tokens = unsafe {
            llama_tokenize(
                self.ptr.as_ptr(),
                c_text.as_ptr(),
                text.len() as i32,
                std::ptr::null_mut(),
                0,
                add_bos,
                special,
            )
        };

        if n_tokens < 0 {
            return Err(InferenceError::Tokenize("Tokenization failed".to_string()));
        }

        // Allocate buffer and tokenize
        let mut tokens = vec![0i32; n_tokens as usize];
        let actual = unsafe {
            llama_tokenize(
                self.ptr.as_ptr(),
                c_text.as_ptr(),
                text.len() as i32,
                tokens.as_mut_ptr(),
                tokens.len() as i32,
                add_bos,
                special,
            )
        };

        if actual < 0 {
            return Err(InferenceError::Tokenize(
                "Tokenization buffer too small".to_string(),
            ));
        }

        tokens.truncate(actual as usize);
        Ok(tokens)
    }

    /// Convert a token to its string representation
    pub fn token_to_str(&self, token: llama_token) -> Result<String> {
        let mut buf = vec![0u8; 128];
        
        let len = unsafe {
            llama_token_to_piece(
                self.ptr.as_ptr(),
                token,
                buf.as_mut_ptr() as *mut i8,
                buf.len() as i32,
                0,     // lstrip
                false, // special
            )
        };

        if len < 0 {
            // Need larger buffer
            buf.resize((-len) as usize, 0);
            let len = unsafe {
                llama_token_to_piece(
                    self.ptr.as_ptr(),
                    token,
                    buf.as_mut_ptr() as *mut i8,
                    buf.len() as i32,
                    0,
                    false,
                )
            };
            if len < 0 {
                return Err(InferenceError::Decode("Token decode failed".to_string()));
            }
            buf.truncate(len as usize);
        } else {
            buf.truncate(len as usize);
        }

        String::from_utf8(buf)
            .map_err(|_| InferenceError::Decode("Invalid UTF-8 in token".to_string()))
    }

    /// Check if token is end-of-generation
    pub fn is_eog_token(&self, token: llama_token) -> bool {
        unsafe { llama_token_is_eog(self.ptr.as_ptr(), token) }
    }

    /// Get the BOS (beginning of sequence) token
    pub fn token_bos(&self) -> llama_token {
        unsafe { llama_token_bos(self.ptr.as_ptr()) }
    }

    /// Get the EOS (end of sequence) token
    pub fn token_eos(&self) -> llama_token {
        unsafe { llama_token_eos(self.ptr.as_ptr()) }
    }

    /// Get the newline token
    pub fn token_nl(&self) -> llama_token {
        unsafe { llama_token_nl(self.ptr.as_ptr()) }
    }
}

impl Drop for LlamaModel {
    fn drop(&mut self) {
        unsafe {
            llama_free_model(self.ptr.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_params_default() {
        let params = ModelParams::default();
        assert_eq!(params.n_gpu_layers, 0);
        assert!(params.use_mmap);
        assert!(!params.use_mlock);
    }
}
