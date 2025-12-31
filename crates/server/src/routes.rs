//! Route definitions

use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use std::time::Duration;

use crate::handlers;
use crate::state::AppState;

/// Build the application router
pub fn build_router(state: Arc<AppState>) -> Router {
    let mut app = Router::new()
        // Health and stats
        .route("/health", get(handlers::health))
        .route("/stats", get(handlers::stats))
        // Query endpoints
        .route("/query", post(handlers::query))
        .route("/classify", post(handlers::classify))
        // Document endpoints
        .route("/add", post(handlers::add_document))
        .route("/search", post(handlers::search))
        .route("/documents", get(handlers::list_documents))
        // State
        .with_state(state.clone());

    // Add middleware
    app = app.layer(TraceLayer::new_for_http());
    app = app.layer(TimeoutLayer::new(Duration::from_secs(state.config.timeout_secs)));

    if state.config.enable_cors {
        app = app.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    }

    app
}
