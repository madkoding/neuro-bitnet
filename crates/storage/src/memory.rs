//! In-memory storage implementation

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use tracing::debug;

use neuro_core::{Document, SearchResult};
use crate::error::{Result, StorageError};
use crate::similarity::top_k_similar;
use crate::storage::{Storage, StorageStats};

/// In-memory document storage
///
/// Fast but non-persistent. Ideal for testing or ephemeral use cases.
pub struct MemoryStorage {
    documents: HashMap<String, Document>,
    embeddings: Vec<Vec<f32>>,
    id_to_index: HashMap<String, usize>,
    dimension: Option<usize>,
}

impl MemoryStorage {
    /// Create a new empty memory storage
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            embeddings: Vec::new(),
            id_to_index: HashMap::new(),
            dimension: None,
        }
    }

    /// Create with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            documents: HashMap::with_capacity(capacity),
            embeddings: Vec::with_capacity(capacity),
            id_to_index: HashMap::with_capacity(capacity),
            dimension: None,
        }
    }

    /// Get the embedding dimension (if any documents exist)
    pub fn dimension(&self) -> Option<usize> {
        self.dimension
    }

    fn validate_embedding(&self, embedding: &[f32]) -> Result<()> {
        if let Some(dim) = self.dimension {
            if embedding.len() != dim {
                return Err(StorageError::DimensionMismatch {
                    expected: dim,
                    actual: embedding.len(),
                });
            }
        }
        Ok(())
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn add(&mut self, document: Document) -> Result<()> {
        let embedding = document
            .embedding
            .as_ref()
            .ok_or_else(|| StorageError::MissingEmbedding(document.id.clone()))?;

        if self.documents.contains_key(&document.id) {
            return Err(StorageError::AlreadyExists(document.id.clone()));
        }

        // Set or validate dimension
        if self.dimension.is_none() {
            self.dimension = Some(embedding.len());
        }
        self.validate_embedding(embedding)?;

        debug!("Adding document {} ({} chars)", document.id, document.content.len());

        let index = self.embeddings.len();
        self.embeddings.push(embedding.clone());
        self.id_to_index.insert(document.id.clone(), index);
        self.documents.insert(document.id.clone(), document);

        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Document> {
        self.documents
            .get(id)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(id.to_string()))
    }

    async fn delete(&mut self, id: &str) -> Result<()> {
        if !self.documents.contains_key(id) {
            return Err(StorageError::NotFound(id.to_string()));
        }

        debug!("Deleting document {}", id);

        // Note: This leaves a "hole" in embeddings vec
        // For simplicity, we keep the index mapping consistent
        // A production system might compact periodically
        self.documents.remove(id);
        self.id_to_index.remove(id);

        Ok(())
    }

    async fn exists(&self, id: &str) -> bool {
        self.documents.contains_key(id)
    }

    async fn search(&self, embedding: &[f32], top_k: usize) -> Result<Vec<SearchResult>> {
        if self.documents.is_empty() {
            return Ok(Vec::new());
        }

        self.validate_embedding(embedding)?;

        // Get valid embeddings with their document IDs
        let valid_docs: Vec<(&String, &Vec<f32>)> = self
            .id_to_index
            .iter()
            .filter_map(|(id, &idx)| {
                if self.documents.contains_key(id) {
                    Some((id, &self.embeddings[idx]))
                } else {
                    None
                }
            })
            .collect();

        if valid_docs.is_empty() {
            return Ok(Vec::new());
        }

        let doc_embeddings: Vec<Vec<f32>> = valid_docs.iter().map(|(_, e)| (*e).clone()).collect();
        let doc_ids: Vec<&String> = valid_docs.iter().map(|(id, _)| *id).collect();

        let top_results = top_k_similar(embedding, &doc_embeddings, top_k);

        let results: Vec<SearchResult> = top_results
            .into_iter()
            .enumerate()
            .filter_map(|(rank, (idx, score))| {
                let id = doc_ids.get(idx)?;
                let document = self.documents.get(*id)?.clone();
                Some(SearchResult::new(document, score).with_rank(rank))
            })
            .collect();

        Ok(results)
    }

    async fn search_by_user(
        &self,
        embedding: &[f32],
        user_id: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        if self.documents.is_empty() {
            return Ok(Vec::new());
        }

        self.validate_embedding(embedding)?;

        // Filter by user
        let valid_docs: Vec<(&String, &Vec<f32>)> = self
            .id_to_index
            .iter()
            .filter_map(|(id, &idx)| {
                let doc = self.documents.get(id)?;
                if doc.user_id.as_deref() == Some(user_id) {
                    Some((id, &self.embeddings[idx]))
                } else {
                    None
                }
            })
            .collect();

        if valid_docs.is_empty() {
            return Ok(Vec::new());
        }

        let doc_embeddings: Vec<Vec<f32>> = valid_docs.iter().map(|(_, e)| (*e).clone()).collect();
        let doc_ids: Vec<&String> = valid_docs.iter().map(|(id, _)| *id).collect();

        let top_results = top_k_similar(embedding, &doc_embeddings, top_k);

        let results: Vec<SearchResult> = top_results
            .into_iter()
            .enumerate()
            .filter_map(|(rank, (idx, score))| {
                let id = doc_ids.get(idx)?;
                let document = self.documents.get(*id)?.clone();
                Some(SearchResult::new(document, score).with_rank(rank))
            })
            .collect();

        Ok(results)
    }

    async fn list(&self) -> Result<Vec<Document>> {
        Ok(self.documents.values().cloned().collect())
    }

    async fn list_by_user(&self, user_id: &str) -> Result<Vec<Document>> {
        Ok(self
            .documents
            .values()
            .filter(|d| d.user_id.as_deref() == Some(user_id))
            .cloned()
            .collect())
    }

    async fn count(&self) -> usize {
        self.documents.len()
    }

    async fn clear(&mut self) -> Result<()> {
        self.documents.clear();
        self.embeddings.clear();
        self.id_to_index.clear();
        self.dimension = None;
        Ok(())
    }

    async fn stats(&self) -> StorageStats {
        let unique_users: HashSet<&str> = self
            .documents
            .values()
            .filter_map(|d| d.user_id.as_deref())
            .collect();

        let total_content_bytes: usize = self.documents.values().map(|d| d.content.len()).sum();

        StorageStats {
            document_count: self.documents.len(),
            embedding_dimension: self.dimension,
            total_content_bytes,
            unique_users: unique_users.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc(id: &str, content: &str, embedding: Vec<f32>) -> Document {
        Document::with_id(id, content).with_embedding(embedding)
    }

    #[tokio::test]
    async fn test_add_and_get() {
        let mut storage = MemoryStorage::new();
        let doc = make_doc("doc1", "Hello, world!", vec![1.0, 0.0, 0.0]);

        storage.add(doc.clone()).await.unwrap();

        let retrieved = storage.get("doc1").await.unwrap();
        assert_eq!(retrieved.content, "Hello, world!");
    }

    #[tokio::test]
    async fn test_add_duplicate() {
        let mut storage = MemoryStorage::new();
        let doc1 = make_doc("doc1", "First", vec![1.0, 0.0, 0.0]);
        let doc2 = make_doc("doc1", "Second", vec![0.0, 1.0, 0.0]);

        storage.add(doc1).await.unwrap();
        let result = storage.add(doc2).await;

        assert!(matches!(result, Err(StorageError::AlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_dimension_mismatch() {
        let mut storage = MemoryStorage::new();
        let doc1 = make_doc("doc1", "First", vec![1.0, 0.0, 0.0]);
        let doc2 = make_doc("doc2", "Second", vec![1.0, 0.0]); // Wrong dimension

        storage.add(doc1).await.unwrap();
        let result = storage.add(doc2).await;

        assert!(matches!(result, Err(StorageError::DimensionMismatch { .. })));
    }

    #[tokio::test]
    async fn test_search() {
        let mut storage = MemoryStorage::new();
        storage
            .add(make_doc("doc1", "Similar", vec![1.0, 0.0, 0.0]))
            .await
            .unwrap();
        storage
            .add(make_doc("doc2", "Different", vec![0.0, 1.0, 0.0]))
            .await
            .unwrap();
        storage
            .add(make_doc("doc3", "Also similar", vec![0.9, 0.1, 0.0]))
            .await
            .unwrap();

        let results = storage.search(&[1.0, 0.0, 0.0], 2).await.unwrap();

        assert_eq!(results.len(), 2);
        // Most similar should be doc1 (identical)
        assert!(results[0].score > 0.99);
    }

    #[tokio::test]
    async fn test_search_by_user() {
        let mut storage = MemoryStorage::new();
        storage
            .add(
                make_doc("doc1", "User A doc", vec![1.0, 0.0, 0.0])
                    .with_user_id("user_a"),
            )
            .await
            .unwrap();
        storage
            .add(
                make_doc("doc2", "User B doc", vec![0.9, 0.1, 0.0])
                    .with_user_id("user_b"),
            )
            .await
            .unwrap();

        let results = storage
            .search_by_user(&[1.0, 0.0, 0.0], "user_a", 10)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document.content, "User A doc");
    }

    #[tokio::test]
    async fn test_delete() {
        let mut storage = MemoryStorage::new();
        storage
            .add(make_doc("doc1", "Hello", vec![1.0, 0.0, 0.0]))
            .await
            .unwrap();

        assert!(storage.exists("doc1").await);
        storage.delete("doc1").await.unwrap();
        assert!(!storage.exists("doc1").await);
    }

    #[tokio::test]
    async fn test_stats() {
        let mut storage = MemoryStorage::new();
        storage
            .add(
                make_doc("doc1", "Hello", vec![1.0, 0.0, 0.0])
                    .with_user_id("user_a"),
            )
            .await
            .unwrap();
        storage
            .add(
                make_doc("doc2", "World", vec![0.0, 1.0, 0.0])
                    .with_user_id("user_b"),
            )
            .await
            .unwrap();

        let stats = storage.stats().await;
        assert_eq!(stats.document_count, 2);
        assert_eq!(stats.embedding_dimension, Some(3));
        assert_eq!(stats.unique_users, 2);
    }
}
