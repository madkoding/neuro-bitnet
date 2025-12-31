//! Server implementation

use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

use crate::config::ServerConfig;
use crate::error::{Result, ServerError};
use crate::routes::build_router;
use crate::state::AppState;

/// The neuro-bitnet server
pub struct Server {
    state: Arc<AppState>,
}

impl Server {
    /// Create a new server with the given configuration
    pub async fn new(config: ServerConfig) -> Result<Self> {
        let state = Arc::new(AppState::new(config).await?);
        Ok(Self { state })
    }

    /// Get a reference to the application state
    pub fn state(&self) -> Arc<AppState> {
        self.state.clone()
    }

    /// Run the server
    pub async fn run(self) -> Result<()> {
        let addr = self.state.config.bind_address();
        let router = build_router(self.state.clone());

        info!("Starting neuro-bitnet server on {}", addr);
        info!(
            "Embedding model: {}",
            self.state.embedder.model()
        );

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to bind to {}: {}", addr, e)))?;

        info!("Server listening on http://{}", addr);

        axum::serve(listener, router)
            .await
            .map_err(|e| ServerError::Internal(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Run the server until a shutdown signal is received
    pub async fn run_with_shutdown(self, shutdown: impl std::future::Future<Output = ()> + Send + 'static) -> Result<()> {
        let addr = self.state.config.bind_address();
        let router = build_router(self.state.clone());

        info!("Starting neuro-bitnet server on {}", addr);

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to bind to {}: {}", addr, e)))?;

        info!("Server listening on http://{}", addr);

        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown)
            .await
            .map_err(|e| ServerError::Internal(format!("Server error: {}", e)))?;

        info!("Server shutdown complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use serde_json::json;

    async fn test_server() -> TestServer {
        let config = ServerConfig {
            storage_path: None,
            ..ServerConfig::development()
        };
        let state = Arc::new(AppState::new(config).await.unwrap());
        let router = build_router(state);
        TestServer::new(router).unwrap()
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let server = test_server().await;
        let response = server.get("/health").await;
        
        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], "healthy");
    }

    #[tokio::test]
    async fn test_stats_endpoint() {
        let server = test_server().await;
        let response = server.get("/stats").await;
        
        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert!(body["document_count"].is_number());
    }

    #[tokio::test]
    #[ignore = "Requires embedding model download"]
    async fn test_add_and_search() {
        let server = test_server().await;

        // Add a document
        let add_response = server
            .post("/add")
            .json(&json!({
                "content": "Rust is a systems programming language"
            }))
            .await;
        
        add_response.assert_status(axum::http::StatusCode::CREATED);

        // Search for it
        let search_response = server
            .post("/search")
            .json(&json!({
                "query": "programming language",
                "top_k": 5
            }))
            .await;
        
        search_response.assert_status_ok();
        let results: Vec<serde_json::Value> = search_response.json();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_classify() {
        let server = test_server().await;

        let response = server
            .post("/classify")
            .json(&json!({
                "query": "What is 2 + 2?"
            }))
            .await;
        
        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["category"], "math");
    }

    #[tokio::test]
    #[ignore = "Requires embedding model download"]
    async fn test_query() {
        let server = test_server().await;

        // Add some context first
        server
            .post("/add")
            .json(&json!({
                "content": "The capital of France is Paris."
            }))
            .await;

        // Query
        let response = server
            .post("/query")
            .json(&json!({
                "query": "What is the capital of France?"
            }))
            .await;
        
        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert!(body["classification"].is_object());
    }
}
