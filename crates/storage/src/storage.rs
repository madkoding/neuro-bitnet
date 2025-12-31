//! Storage trait definition

use async_trait::async_trait;
use neuro_core::{Document, SearchResult};
use crate::error::Result;

/// Statistics about the storage
#[derive(Debug, Clone, Default)]
pub struct StorageStats {
    /// Total number of documents
    pub document_count: usize,
    /// Embedding dimension (if documents exist)
    pub embedding_dimension: Option<usize>,
    /// Total content size in bytes
    pub total_content_bytes: usize,
    /// Number of unique users
    pub unique_users: usize,
}

/// Trait for document storage with vector similarity search
#[async_trait]
pub trait Storage: Send + Sync {
    /// Add a document to storage
    ///
    /// The document must have an embedding. Returns error if document
    /// with same ID already exists.
    async fn add(&mut self, document: Document) -> Result<()>;

    /// Add multiple documents to storage
    async fn add_batch(&mut self, documents: Vec<Document>) -> Result<()> {
        for doc in documents {
            self.add(doc).await?;
        }
        Ok(())
    }

    /// Get a document by ID
    async fn get(&self, id: &str) -> Result<Document>;

    /// Delete a document by ID
    async fn delete(&mut self, id: &str) -> Result<()>;

    /// Check if a document exists
    async fn exists(&self, id: &str) -> bool;

    /// Search for similar documents
    ///
    /// # Arguments
    /// * `embedding` - Query embedding vector
    /// * `top_k` - Maximum number of results to return
    ///
    /// # Returns
    /// Vector of search results sorted by similarity descending
    async fn search(&self, embedding: &[f32], top_k: usize) -> Result<Vec<SearchResult>>;

    /// Search with filtering by user ID
    async fn search_by_user(
        &self,
        embedding: &[f32],
        user_id: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>>;

    /// List all documents
    async fn list(&self) -> Result<Vec<Document>>;

    /// List documents for a specific user
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<Document>>;

    /// Get the number of documents
    async fn count(&self) -> usize;

    /// Clear all documents
    async fn clear(&mut self) -> Result<()>;

    /// Get storage statistics
    async fn stats(&self) -> StorageStats;
}
