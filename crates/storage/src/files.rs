//! File-based persistent storage implementation

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

use neuro_core::{Document, SearchResult};
use crate::error::{Result, StorageError};
use crate::similarity::top_k_similar;
use crate::storage::{Storage, StorageStats};

/// File-based document storage
///
/// Persists documents as JSON files. Each save operation writes
/// the entire storage to disk for consistency.
pub struct FileStorage {
    path: PathBuf,
    documents: HashMap<String, Document>,
    embeddings: Vec<Vec<f32>>,
    id_to_index: HashMap<String, usize>,
    dimension: Option<usize>,
    auto_save: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StorageData {
    documents: Vec<Document>,
    dimension: Option<usize>,
}

impl FileStorage {
    /// Create a new file storage at the given path
    ///
    /// If the file exists, it will be loaded. Otherwise, an empty storage is created.
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        let mut storage = Self {
            path,
            documents: HashMap::new(),
            embeddings: Vec::new(),
            id_to_index: HashMap::new(),
            dimension: None,
            auto_save: true,
        };

        // Try to load existing data
        if storage.path.exists() {
            storage.load().await?;
        }

        Ok(storage)
    }

    /// Create with auto-save disabled (must call save() manually)
    pub async fn new_manual_save(path: impl AsRef<Path>) -> Result<Self> {
        let mut storage = Self::new(path).await?;
        storage.auto_save = false;
        Ok(storage)
    }

    /// Enable or disable auto-save
    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save = enabled;
    }

    /// Get the storage file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the embedding dimension
    pub fn dimension(&self) -> Option<usize> {
        self.dimension
    }

    /// Manually save storage to disk
    pub async fn save(&self) -> Result<()> {
        let data = StorageData {
            documents: self.documents.values().cloned().collect(),
            dimension: self.dimension,
        };

        let json = serde_json::to_string_pretty(&data)?;

        // Write to temp file first, then rename for atomicity
        let temp_path = self.path.with_extension("tmp");
        fs::write(&temp_path, &json).await?;
        fs::rename(&temp_path, &self.path).await?;

        debug!("Saved {} documents to {:?}", self.documents.len(), self.path);
        Ok(())
    }

    /// Load storage from disk
    pub async fn load(&mut self) -> Result<()> {
        if !self.path.exists() {
            info!("Storage file does not exist, starting empty: {:?}", self.path);
            return Ok(());
        }

        let json = fs::read_to_string(&self.path).await?;
        let data: StorageData = serde_json::from_str(&json)?;

        self.documents.clear();
        self.embeddings.clear();
        self.id_to_index.clear();
        self.dimension = data.dimension;

        for doc in data.documents {
            if let Some(ref embedding) = doc.embedding {
                let index = self.embeddings.len();
                self.embeddings.push(embedding.clone());
                self.id_to_index.insert(doc.id.clone(), index);
            }
            self.documents.insert(doc.id.clone(), doc);
        }

        info!(
            "Loaded {} documents from {:?}",
            self.documents.len(),
            self.path
        );
        Ok(())
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

    async fn maybe_save(&self) -> Result<()> {
        if self.auto_save {
            self.save().await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Storage for FileStorage {
    async fn add(&mut self, document: Document) -> Result<()> {
        let embedding = document
            .embedding
            .as_ref()
            .ok_or_else(|| StorageError::MissingEmbedding(document.id.clone()))?;

        if self.documents.contains_key(&document.id) {
            return Err(StorageError::AlreadyExists(document.id.clone()));
        }

        if self.dimension.is_none() {
            self.dimension = Some(embedding.len());
        }
        self.validate_embedding(embedding)?;

        debug!("Adding document {} ({} chars)", document.id, document.content.len());

        let index = self.embeddings.len();
        self.embeddings.push(embedding.clone());
        self.id_to_index.insert(document.id.clone(), index);
        self.documents.insert(document.id.clone(), document);

        self.maybe_save().await?;
        Ok(())
    }

    async fn add_batch(&mut self, documents: Vec<Document>) -> Result<()> {
        // Temporarily disable auto-save for batch
        let was_auto_save = self.auto_save;
        self.auto_save = false;

        for doc in documents {
            self.add(doc).await?;
        }

        self.auto_save = was_auto_save;
        self.maybe_save().await?;
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

        self.documents.remove(id);
        self.id_to_index.remove(id);

        self.maybe_save().await?;
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

        self.maybe_save().await?;
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
    use tempfile::tempdir;

    fn make_doc(id: &str, content: &str, embedding: Vec<f32>) -> Document {
        Document::with_id(id, content).with_embedding(embedding)
    }

    #[tokio::test]
    async fn test_file_storage_basic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("storage.json");

        let mut storage = FileStorage::new(&path).await.unwrap();
        let doc = make_doc("doc1", "Hello, world!", vec![1.0, 0.0, 0.0]);

        storage.add(doc).await.unwrap();
        assert_eq!(storage.count().await, 1);

        // File should exist
        assert!(path.exists());
    }

    #[tokio::test]
    async fn test_file_storage_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("storage.json");

        // Create and add document
        {
            let mut storage = FileStorage::new(&path).await.unwrap();
            storage
                .add(make_doc("doc1", "Hello", vec![1.0, 0.0, 0.0]))
                .await
                .unwrap();
        }

        // Load from file
        {
            let storage = FileStorage::new(&path).await.unwrap();
            assert_eq!(storage.count().await, 1);
            let doc = storage.get("doc1").await.unwrap();
            assert_eq!(doc.content, "Hello");
        }
    }

    #[tokio::test]
    async fn test_file_storage_search() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("storage.json");

        let mut storage = FileStorage::new(&path).await.unwrap();
        storage
            .add(make_doc("doc1", "Similar", vec![1.0, 0.0, 0.0]))
            .await
            .unwrap();
        storage
            .add(make_doc("doc2", "Different", vec![0.0, 1.0, 0.0]))
            .await
            .unwrap();

        let results = storage.search(&[1.0, 0.0, 0.0], 1).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document.content, "Similar");
    }

    #[tokio::test]
    async fn test_file_storage_manual_save() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("storage.json");

        let mut storage = FileStorage::new_manual_save(&path).await.unwrap();
        storage
            .add(make_doc("doc1", "Hello", vec![1.0, 0.0, 0.0]))
            .await
            .unwrap();

        // File should NOT exist yet (auto-save disabled)
        assert!(!path.exists());

        // Manual save
        storage.save().await.unwrap();
        assert!(path.exists());
    }
}
