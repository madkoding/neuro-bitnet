//! HTTP request handlers

use axum::extract::{Json, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

use neuro_core::{Document, DocumentSource, QueryResult};
use neuro_search::WebSearcher;
use neuro_storage::Storage;

use crate::error::{Result, ServerError};
use crate::state::AppState;

// ============================================================================
// Request/Response types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub query: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    5
}

#[derive(Debug, Deserialize)]
pub struct AddDocumentRequest {
    pub content: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_secs: u64,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub uptime_secs: u64,
    pub request_count: u64,
    pub document_count: usize,
    pub embedding_dimension: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct AddDocumentResponse {
    pub id: String,
    pub message: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// Health check endpoint
pub async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        uptime_secs: state.uptime_secs(),
    })
}

/// Statistics endpoint
pub async fn stats(State(state): State<Arc<AppState>>) -> Result<Json<StatsResponse>> {
    let storage = state.storage.read().await;
    let stats = storage.stats().await;

    Ok(Json(StatsResponse {
        uptime_secs: state.uptime_secs(),
        request_count: state.get_request_count().await,
        document_count: stats.document_count,
        embedding_dimension: stats.embedding_dimension,
    }))
}

/// Intelligent query endpoint
pub async fn query(
    State(state): State<Arc<AppState>>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<QueryResult>> {
    state.increment_requests().await;
    let start = Instant::now();

    if req.query.trim().is_empty() {
        return Err(ServerError::BadRequest("Empty query".to_string()));
    }

    info!("Processing query: {}", req.query);

    // Classify the query
    let classification = state.classifier.classify(&req.query);
    debug!("Classification: {:?}", classification);

    // Generate embedding for search
    let embedding = state
        .embedder
        .embed_single(&req.query)
        .map_err(ServerError::Embedding)?;

    // Search storage
    let storage = state.storage.read().await;
    let search_results = if let Some(ref user_id) = req.user_id {
        storage
            .search_by_user(&embedding, user_id, req.top_k)
            .await
            .map_err(ServerError::Storage)?
    } else {
        storage
            .search(&embedding, req.top_k)
            .await
            .map_err(ServerError::Storage)?
    };
    drop(storage);

    // Build result
    let mut result = QueryResult::new(&req.query, classification);
    result = result.with_search_results(search_results);
    result.build_context(state.config.max_search_results * 1000);
    
    // Check if we need web search
    let needs_web = matches!(
        result.classification.strategy,
        neuro_core::QueryStrategy::RagThenWeb | neuro_core::QueryStrategy::WebSearch
    ) && !result.has_relevant_results();

    if needs_web {
        debug!("Attempting web search for: {}", req.query);
        match state.web_searcher.search(&req.query, 3).await {
            Ok(web_results) => {
                let mut context = result.context.clone();
                for web_result in web_results {
                    if !context.is_empty() {
                        context.push_str("\n\n---\n\n");
                    }
                    context.push_str(&web_result.to_rag_context());
                }
                result = result.with_context(context).with_web_search();
            }
            Err(e) => {
                debug!("Web search failed: {}", e);
            }
        }
    }

    result = result.with_processing_time(start.elapsed().as_millis() as u64);

    Ok(Json(result))
}

/// Classify query without execution
pub async fn classify(
    State(state): State<Arc<AppState>>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<neuro_core::ClassificationResult>> {
    state.increment_requests().await;

    if req.query.trim().is_empty() {
        return Err(ServerError::BadRequest("Empty query".to_string()));
    }

    let result = state.classifier.classify(&req.query);
    Ok(Json(result))
}

/// Add document endpoint
pub async fn add_document(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddDocumentRequest>,
) -> Result<(StatusCode, Json<AddDocumentResponse>)> {
    state.increment_requests().await;

    if req.content.trim().is_empty() {
        return Err(ServerError::BadRequest("Empty content".to_string()));
    }

    info!("Adding document ({} chars)", req.content.len());

    // Generate embedding
    let embedding = state
        .embedder
        .embed_single(&req.content)
        .map_err(ServerError::Embedding)?;

    // Build document
    let mut doc = Document::new(&req.content).with_embedding(embedding);

    if let Some(user_id) = req.user_id {
        doc = doc.with_user_id(user_id);
    }

    if let Some(source) = req.source {
        let source = match source.as_str() {
            "manual" => DocumentSource::Manual,
            "file" => DocumentSource::File,
            "web" => DocumentSource::Web,
            "conversation" => DocumentSource::Conversation,
            "code" => DocumentSource::Code,
            _ => DocumentSource::Manual,
        };
        doc = doc.with_source(source);
    }

    if let Some(metadata) = req.metadata {
        if let Some(obj) = metadata.as_object() {
            for (key, value) in obj {
                doc = doc.with_metadata(key, value.clone());
            }
        }
    }

    let id = doc.id.clone();

    // Add to storage
    let mut storage = state.storage.write().await;
    storage.add(doc).await.map_err(ServerError::Storage)?;

    Ok((
        StatusCode::CREATED,
        Json(AddDocumentResponse {
            id,
            message: "Document added successfully".to_string(),
        }),
    ))
}

/// Search endpoint
pub async fn search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<Vec<neuro_core::SearchResult>>> {
    state.increment_requests().await;

    if req.query.trim().is_empty() {
        return Err(ServerError::BadRequest("Empty query".to_string()));
    }

    debug!("Searching for: {}", req.query);

    // Generate embedding
    let embedding = state
        .embedder
        .embed_single(&req.query)
        .map_err(ServerError::Embedding)?;

    // Search
    let storage = state.storage.read().await;
    let results = if let Some(ref user_id) = req.user_id {
        storage
            .search_by_user(&embedding, user_id, req.top_k)
            .await
            .map_err(ServerError::Storage)?
    } else {
        storage
            .search(&embedding, req.top_k)
            .await
            .map_err(ServerError::Storage)?
    };

    Ok(Json(results))
}

/// List documents endpoint
pub async fn list_documents(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Document>>> {
    state.increment_requests().await;

    let storage = state.storage.read().await;
    let documents = storage.list().await.map_err(ServerError::Storage)?;

    Ok(Json(documents))
}
