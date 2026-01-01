//! Daemon server implementation

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::{handlers, AppState};

/// Daemon server configuration
pub struct DaemonConfig {
    /// Host to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// Path to the model
    pub model_path: String,
    /// Auto-translate non-English queries
    pub auto_translate: bool,
    /// Maximum tokens
    pub max_tokens: u32,
    /// Temperature
    pub temperature: f32,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 11435,
            model_path: String::new(),
            auto_translate: true,
            max_tokens: 512,
            temperature: 0.7,
        }
    }
}

/// The daemon server
pub struct DaemonServer {
    config: DaemonConfig,
    state: Arc<AppState>,
}

impl DaemonServer {
    /// Create a new daemon server
    pub fn new(config: DaemonConfig) -> Self {
        let state = Arc::new(AppState {
            model: Arc::new(tokio::sync::RwLock::new(None)),
            model_path: config.model_path.clone(),
            auto_translate: config.auto_translate,
            max_tokens: config.max_tokens,
            temperature: config.temperature,
        });

        Self { config, state }
    }

    /// Build the router
    fn router(&self) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Router::new()
            // Health check
            .route("/health", get(handlers::health))
            .route("/v1/health", get(handlers::health))
            // Generate endpoint
            .route("/generate", post(handlers::generate))
            .route("/v1/generate", post(handlers::generate))
            // OpenAI-compatible chat endpoint
            .route("/v1/chat/completions", post(handlers::chat))
            // Legacy endpoint
            .route("/api/generate", post(handlers::generate))
            .layer(cors)
            .layer(TraceLayer::new_for_http())
            .with_state(self.state.clone())
    }

    /// Run the server
    pub async fn run(self) -> anyhow::Result<()> {
        // Load model in background
        let state = self.state.clone();
        let model_path = self.config.model_path.clone();
        
        tokio::spawn(async move {
            info!("Loading model: {}", model_path);
            if let Err(e) = state.load_model().await {
                tracing::error!("Failed to load model: {}", e);
            } else {
                info!("Model loaded successfully");
            }
        });

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()?;

        info!("Starting daemon on {}", addr);
        info!("Auto-translate: {}", self.config.auto_translate);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.router()).await?;

        Ok(())
    }

    /// Run with graceful shutdown
    pub async fn run_with_shutdown<F>(self, shutdown: F) -> anyhow::Result<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        // Load model in background
        let state = self.state.clone();
        let model_path = self.config.model_path.clone();
        
        tokio::spawn(async move {
            info!("Loading model: {}", model_path);
            if let Err(e) = state.load_model().await {
                tracing::error!("Failed to load model: {}", e);
            } else {
                info!("Model loaded successfully");
            }
        });

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()?;

        info!("Starting daemon on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.router())
            .with_graceful_shutdown(shutdown)
            .await?;

        Ok(())
    }
}
