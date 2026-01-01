//! Application state for the daemon

use neuro_inference::{InferenceModel, InferenceConfig};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state
pub struct AppState {
    /// The loaded inference model
    pub model: Arc<RwLock<Option<InferenceModel>>>,
    /// Model path
    pub model_path: String,
    /// Whether to auto-translate non-English queries
    pub auto_translate: bool,
    /// Maximum tokens for generation
    pub max_tokens: u32,
    /// Temperature for sampling
    pub temperature: f32,
}

impl AppState {
    pub fn new(model_path: String, auto_translate: bool) -> Self {
        Self {
            model: Arc::new(RwLock::new(None)),
            model_path,
            auto_translate,
            max_tokens: 512,
            temperature: 0.7,
        }
    }

    /// Load the model
    pub async fn load_model(&self) -> anyhow::Result<()> {
        let config = InferenceConfig::new(&self.model_path);
        let model = tokio::task::spawn_blocking(move || InferenceModel::load(config)).await??;
        
        let mut guard = self.model.write().await;
        *guard = Some(model);
        Ok(())
    }

    /// Check if model is loaded
    pub async fn is_model_loaded(&self) -> bool {
        self.model.read().await.is_some()
    }
}
