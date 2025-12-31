//! Application state

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use neuro_classifier::Classifier;
use neuro_embeddings::{Embedder, FastEmbedder, EmbeddingModel};
use neuro_storage::{Storage, MemoryStorage, FileStorage};
use neuro_search::{WebSearcher, WikipediaSearcher};

use crate::config::ServerConfig;
use crate::error::{Result, ServerError};

/// Shared application state
pub struct AppState {
    /// Document storage
    pub storage: RwLock<Box<dyn Storage>>,
    
    /// Embedding generator
    pub embedder: Arc<dyn Embedder>,
    
    /// Query classifier
    pub classifier: Classifier,
    
    /// Web searcher
    pub web_searcher: Arc<dyn WebSearcher>,
    
    /// Server configuration
    pub config: ServerConfig,
    
    /// Server start time
    pub start_time: Instant,
    
    /// Request counter
    pub request_count: RwLock<u64>,
}

impl AppState {
    /// Create new application state
    pub async fn new(config: ServerConfig) -> Result<Self> {
        // Initialize storage
        let storage: Box<dyn Storage> = if let Some(ref path) = config.storage_path {
            Box::new(
                FileStorage::new(path)
                    .await
                    .map_err(|e| ServerError::Internal(e.to_string()))?,
            )
        } else {
            Box::new(MemoryStorage::new())
        };

        // Initialize embedder
        let model: EmbeddingModel = config
            .embedding_model
            .parse()
            .unwrap_or(EmbeddingModel::AllMiniLmL6V2);
        
        let embedder = Arc::new(
            FastEmbedder::new(model)
                .map_err(|e| ServerError::Internal(e.to_string()))?,
        );

        // Initialize classifier
        let classifier = Classifier::new();

        // Initialize web searcher
        let web_searcher = Arc::new(WikipediaSearcher::new());

        Ok(Self {
            storage: RwLock::new(storage),
            embedder,
            classifier,
            web_searcher,
            config,
            start_time: Instant::now(),
            request_count: RwLock::new(0),
        })
    }

    /// Increment request counter
    pub async fn increment_requests(&self) {
        let mut count = self.request_count.write().await;
        *count += 1;
    }

    /// Get uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get request count
    pub async fn get_request_count(&self) -> u64 {
        *self.request_count.read().await
    }
}
