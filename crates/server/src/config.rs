//! Server configuration

use std::path::PathBuf;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,
    
    /// Port to listen on
    pub port: u16,
    
    /// Path for file storage (if using FileStorage)
    pub storage_path: Option<PathBuf>,
    
    /// Embedding model to use
    pub embedding_model: String,
    
    /// Maximum number of search results
    pub max_search_results: usize,
    
    /// Enable CORS
    pub enable_cors: bool,
    
    /// Request timeout in seconds
    pub timeout_secs: u64,
    
    /// Log level
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            storage_path: None,
            embedding_model: "minilm".to_string(),
            max_search_results: 10,
            enable_cors: true,
            timeout_secs: 30,
            log_level: "info".to_string(),
        }
    }
}

impl ServerConfig {
    /// Create config for development
    pub fn development() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            log_level: "debug".to_string(),
            ..Default::default()
        }
    }

    /// Create config for production
    pub fn production() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            log_level: "info".to_string(),
            timeout_secs: 60,
            ..Default::default()
        }
    }

    /// Get the bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
