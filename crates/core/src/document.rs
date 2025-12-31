//! Document types and sources

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Source of a document in the RAG system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentSource {
    /// Manually added by user
    Manual,
    /// Indexed from a file
    File,
    /// Retrieved from web search
    Web,
    /// From conversation history
    Conversation,
    /// From code analysis/indexing
    Code,
}

impl Default for DocumentSource {
    fn default() -> Self {
        Self::Manual
    }
}

impl std::fmt::Display for DocumentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual => write!(f, "manual"),
            Self::File => write!(f, "file"),
            Self::Web => write!(f, "web"),
            Self::Conversation => write!(f, "conversation"),
            Self::Code => write!(f, "code"),
        }
    }
}

/// A document stored in the RAG system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique identifier
    pub id: String,

    /// Text content of the document
    pub content: String,

    /// User identifier (for multi-tenant support)
    #[serde(default)]
    pub user_id: Option<String>,

    /// Source of the document
    #[serde(default)]
    pub source: DocumentSource,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Vector embedding (optional, may be computed lazily)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

impl Document {
    /// Create a new document with auto-generated ID and timestamp
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            user_id: None,
            source: DocumentSource::default(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            embedding: None,
        }
    }

    /// Create a document with a specific ID
    pub fn with_id(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            user_id: None,
            source: DocumentSource::default(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            embedding: None,
        }
    }

    /// Set the user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the document source
    pub fn with_source(mut self, source: DocumentSource) -> Self {
        self.source = source;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Set the embedding vector
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// Get content length in characters
    pub fn content_len(&self) -> usize {
        self.content.len()
    }

    /// Check if document has an embedding
    pub fn has_embedding(&self) -> bool {
        self.embedding.is_some()
    }

    /// Get embedding dimension (if available)
    pub fn embedding_dim(&self) -> Option<usize> {
        self.embedding.as_ref().map(|e| e.len())
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new("Hello, world!");
        assert_eq!(doc.content, "Hello, world!");
        assert!(!doc.id.is_empty());
        assert_eq!(doc.source, DocumentSource::Manual);
        assert!(doc.embedding.is_none());
    }

    #[test]
    fn test_document_builder() {
        let doc = Document::new("Test content")
            .with_user_id("user123")
            .with_source(DocumentSource::File)
            .with_metadata("filename", serde_json::json!("test.txt"))
            .with_embedding(vec![0.1, 0.2, 0.3]);

        assert_eq!(doc.user_id, Some("user123".to_string()));
        assert_eq!(doc.source, DocumentSource::File);
        assert!(doc.metadata.contains_key("filename"));
        assert_eq!(doc.embedding_dim(), Some(3));
    }

    #[test]
    fn test_document_source_display() {
        assert_eq!(DocumentSource::Manual.to_string(), "manual");
        assert_eq!(DocumentSource::File.to_string(), "file");
        assert_eq!(DocumentSource::Web.to_string(), "web");
        assert_eq!(DocumentSource::Code.to_string(), "code");
    }

    #[test]
    fn test_document_serialization() {
        let doc = Document::new("Test")
            .with_source(DocumentSource::Web);
        
        let json = serde_json::to_string(&doc).unwrap();
        let parsed: Document = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.content, doc.content);
        assert_eq!(parsed.source, DocumentSource::Web);
    }
}
