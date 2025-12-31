//! Local LLM inference using bitnet.cpp
//!
//! This crate provides CPU-optimized inference for BitNet 1.58-bit models,
//! using Microsoft's bitnet.cpp runtime for maximum performance.
//!
//! ## Features
//!
//! - `download` - Enable model downloading with progress bars
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

mod error;
mod model;
mod sampler;
pub mod models;
pub mod cache;
pub mod subprocess;

pub use error::InferenceError;
pub use model::{InferenceModel, InferenceConfig, GenerateOptions};
pub use sampler::SamplerConfig;
pub use models::BitNetModel;
pub use cache::ModelCache;

#[cfg(feature = "download")]
pub use cache::download::{download_model, get_or_download, DownloadOptions};

