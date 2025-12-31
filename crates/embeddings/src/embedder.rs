//! Embedding generation trait and implementations

use crate::error::{EmbeddingError, Result};
use crate::models::EmbeddingModel;
use fastembed::{InitOptions, TextEmbedding};
use std::sync::Mutex;
use tracing::{debug, info};

/// Trait for text embedding generation
pub trait Embedder: Send + Sync {
    /// Get the model being used
    fn model(&self) -> EmbeddingModel;

    /// Get the embedding dimension
    fn dimension(&self) -> usize;

    /// Generate embedding for a single text
    fn embed_single(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts (more efficient)
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
}

/// FastEmbed-based embedder implementation
pub struct FastEmbedder {
    model: Mutex<TextEmbedding>,
    model_type: EmbeddingModel,
}

impl FastEmbedder {
    /// Create a new FastEmbedder with the specified model
    pub fn new(model_type: EmbeddingModel) -> Result<Self> {
        info!(
            "Initializing FastEmbedder with model: {} ({}D)",
            model_type.model_name(),
            model_type.dimension()
        );

        let fastembed_model = match model_type {
            EmbeddingModel::AllMiniLmL6V2 => fastembed::EmbeddingModel::AllMiniLML6V2,
            EmbeddingModel::AllMiniLmL12V2 => fastembed::EmbeddingModel::AllMiniLML12V2,
            EmbeddingModel::AllMpnetBaseV2 => fastembed::EmbeddingModel::AllMpnetBaseV2,
            EmbeddingModel::BgeSmallEnV15 => fastembed::EmbeddingModel::BGESmallENV15,
            EmbeddingModel::BgeBaseEnV15 => fastembed::EmbeddingModel::BGEBaseENV15,
            EmbeddingModel::BgeLargeEnV15 => fastembed::EmbeddingModel::BGELargeENV15,
            EmbeddingModel::MultilingualE5Small => fastembed::EmbeddingModel::MultilingualE5Small,
            EmbeddingModel::MultilingualE5Base => fastembed::EmbeddingModel::MultilingualE5Base,
            EmbeddingModel::MultilingualE5Large => fastembed::EmbeddingModel::MultilingualE5Large,
        };

        let model = TextEmbedding::try_new(
            InitOptions::new(fastembed_model).with_show_download_progress(true),
        )
        .map_err(|e| EmbeddingError::ModelInit(e.to_string()))?;

        info!("FastEmbedder initialized successfully");

        Ok(Self {
            model: Mutex::new(model),
            model_type,
        })
    }

    /// Create with default model (AllMiniLmL6V2)
    pub fn default_model() -> Result<Self> {
        Self::new(EmbeddingModel::default())
    }

    /// Create with a model specified by name
    pub fn from_model_name(name: &str) -> Result<Self> {
        let model_type: EmbeddingModel = name
            .parse()
            .map_err(|e: String| EmbeddingError::ModelNotFound(e))?;
        Self::new(model_type)
    }
}

impl Embedder for FastEmbedder {
    fn model(&self) -> EmbeddingModel {
        self.model_type
    }

    fn dimension(&self) -> usize {
        self.model_type.dimension()
    }

    fn embed_single(&self, text: &str) -> Result<Vec<f32>> {
        if text.is_empty() {
            return Err(EmbeddingError::InvalidInput("Empty text provided".into()));
        }

        debug!("Embedding single text ({} chars)", text.len());

        let mut model = self
            .model
            .lock()
            .map_err(|_| EmbeddingError::Generation("Lock poisoned".to_string()))?;
        
        let embeddings = model
            .embed(vec![text], None)
            .map_err(|e| EmbeddingError::Generation(e.to_string()))?;

        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::Generation("No embedding returned".into()))
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Validate inputs
        for (i, text) in texts.iter().enumerate() {
            if text.is_empty() {
                return Err(EmbeddingError::InvalidInput(format!(
                    "Empty text at index {}",
                    i
                )));
            }
        }

        debug!("Embedding batch of {} texts", texts.len());

        let mut model = self
            .model
            .lock()
            .map_err(|_| EmbeddingError::Generation("Lock poisoned".to_string()))?;

        model
            .embed(texts.to_vec(), None)
            .map_err(|e| EmbeddingError::Generation(e.to_string()))
    }
}

/// A mock embedder for testing
#[cfg(test)]
pub struct MockEmbedder {
    model_type: EmbeddingModel,
}

#[cfg(test)]
impl MockEmbedder {
    pub fn new(model_type: EmbeddingModel) -> Self {
        Self { model_type }
    }
}

#[cfg(test)]
impl Embedder for MockEmbedder {
    fn model(&self) -> EmbeddingModel {
        self.model_type
    }

    fn dimension(&self) -> usize {
        self.model_type.dimension()
    }

    fn embed_single(&self, text: &str) -> Result<Vec<f32>> {
        if text.is_empty() {
            return Err(EmbeddingError::InvalidInput("Empty text".into()));
        }
        // Return deterministic fake embedding based on text hash
        let hash = text.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
        let dim = self.dimension();
        Ok((0..dim)
            .map(|i| ((hash.wrapping_add(i as u32)) % 1000) as f32 / 1000.0)
            .collect())
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|t| self.embed_single(t)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_embedder() {
        let embedder = MockEmbedder::new(EmbeddingModel::AllMiniLmL6V2);
        
        let embedding = embedder.embed_single("test").unwrap();
        assert_eq!(embedding.len(), 384);
        
        // Same text should give same embedding
        let embedding2 = embedder.embed_single("test").unwrap();
        assert_eq!(embedding, embedding2);
    }

    #[test]
    fn test_mock_embedder_batch() {
        let embedder = MockEmbedder::new(EmbeddingModel::AllMiniLmL6V2);
        
        let embeddings = embedder.embed_batch(&["hello", "world"]).unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 384);
    }

    #[test]
    fn test_mock_embedder_empty_input() {
        let embedder = MockEmbedder::new(EmbeddingModel::AllMiniLmL6V2);
        
        let result = embedder.embed_single("");
        assert!(result.is_err());
    }

    // Integration test - only runs when fastembed can download models
    #[test]
    #[ignore = "Requires model download"]
    fn test_fastembed_real() {
        let embedder = FastEmbedder::new(EmbeddingModel::AllMiniLmL6V2).unwrap();
        
        let embedding = embedder.embed_single("Hello, world!").unwrap();
        assert_eq!(embedding.len(), 384);
        
        // Verify embedding is normalized (approximately unit length)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.1);
    }
}
