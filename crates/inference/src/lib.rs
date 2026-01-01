//! Local LLM inference using bitnet.cpp
//!
//! This crate provides CPU-optimized inference for BitNet 1.58-bit models,
//! using Microsoft's bitnet.cpp runtime for maximum performance.
//!
//! ## Features
//!
//! - `subprocess` - Use subprocess backend (calls llama-cli binary)
//! - `native` - Use native FFI bindings to bitnet.cpp (fastest)
//! - `cuda` - Enable CUDA GPU acceleration (requires `native`)
//! - `download` - Enable model downloading with progress bars
//!
//! ## Backends
//!
//! Two inference backends are available:
//!
//! 1. **Native FFI** (`native` feature) - Direct bindings to bitnet.cpp for
//!    maximum performance. Requires bitnet.cpp to be compiled from source.
//!
//! 2. **Subprocess** (`subprocess` feature, default) - Calls the llama-cli
//!    binary from bitnet.cpp. Works out of the box if binary is installed.
//!
//! ## Example
//!
//! ```ignore
//! use neuro_inference::{InferenceModel, InferenceConfig, GenerateOptions};
//!
//! let config = InferenceConfig::new("path/to/bitnet-model.gguf");
//! let model = InferenceModel::load(config)?;
//!
//! let response = model.generate("Hello, how are you?", &GenerateOptions::default())?;
//! println!("{}", response);
//! ```

mod backend;
mod error;
mod model;
mod sampler;
pub mod models;
pub mod cache;
pub mod translation;

#[cfg(feature = "subprocess")]
pub mod subprocess;

#[cfg(feature = "native")]
pub mod native;

pub use backend::{InferenceBackend, BackendType};
pub use error::InferenceError;
pub use model::{InferenceModel, InferenceConfig, GenerateOptions};
pub use sampler::SamplerConfig;
pub use translation::{Language, detect_language, build_translation_prompt, build_multilingual_prompt, translate_to_english};
pub use models::BitNetModel;
pub use cache::ModelCache;

#[cfg(feature = "native")]
pub use native::{
    NativeBackend, LlamaModel, LlamaContext, LlamaSampler, LlamaBatch,
    ContextPool, PooledContext, PoolConfig, ModelParams, ContextParams,
};

#[cfg(feature = "download")]
pub use cache::download::{download_model, get_or_download, DownloadOptions};

/// Check if native bindings are available
pub fn native_available() -> bool {
    #[cfg(feature = "native")]
    {
        native::is_available()
    }
    #[cfg(not(feature = "native"))]
    {
        false
    }
}

/// Get the default backend type based on availability
pub fn default_backend() -> BackendType {
    if native_available() {
        BackendType::Native
    } else {
        BackendType::Subprocess
    }
}


