//! Search result types

use serde::{Deserialize, Serialize};
use crate::document::Document;
use crate::classification::ClassificationResult;

/// Result from a similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matched document
    pub document: Document,

    /// Similarity score (0.0 - 1.0, higher is better)
    pub score: f32,

    /// Rank in results (0-indexed)
    #[serde(default)]
    pub rank: usize,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(document: Document, score: f32) -> Self {
        Self {
            document,
            score,
            rank: 0,
        }
    }

    /// Set the rank
    pub fn with_rank(mut self, rank: usize) -> Self {
        self.rank = rank;
        self
    }

    /// Check if this is a high-quality match (score >= 0.7)
    pub fn is_relevant(&self) -> bool {
        self.score >= 0.7
    }

    /// Check if this is a low-quality match (score < 0.4)
    pub fn is_weak_match(&self) -> bool {
        self.score < 0.4
    }
}

/// Complete result from a RAG query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Original query text
    pub query: String,

    /// Classification of the query
    pub classification: ClassificationResult,

    /// Search results from RAG (if applicable)
    #[serde(default)]
    pub search_results: Vec<SearchResult>,

    /// Context assembled for LLM
    #[serde(default)]
    pub context: String,

    /// Whether web search was used
    #[serde(default)]
    pub used_web_search: bool,

    /// Total processing time in milliseconds
    #[serde(default)]
    pub processing_time_ms: u64,
}

impl QueryResult {
    /// Create a new query result
    pub fn new(query: impl Into<String>, classification: ClassificationResult) -> Self {
        Self {
            query: query.into(),
            classification,
            search_results: Vec::new(),
            context: String::new(),
            used_web_search: false,
            processing_time_ms: 0,
        }
    }

    /// Add search results
    pub fn with_search_results(mut self, results: Vec<SearchResult>) -> Self {
        self.search_results = results;
        self
    }

    /// Set the context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = context.into();
        self
    }

    /// Mark that web search was used
    pub fn with_web_search(mut self) -> Self {
        self.used_web_search = true;
        self
    }

    /// Set processing time
    pub fn with_processing_time(mut self, ms: u64) -> Self {
        self.processing_time_ms = ms;
        self
    }

    /// Check if any relevant results were found
    pub fn has_relevant_results(&self) -> bool {
        self.search_results.iter().any(|r| r.is_relevant())
    }

    /// Get the number of search results
    pub fn result_count(&self) -> usize {
        self.search_results.len()
    }

    /// Get the best search result (if any)
    pub fn best_result(&self) -> Option<&SearchResult> {
        self.search_results.first()
    }

    /// Build context string from search results
    pub fn build_context(&mut self, max_length: usize) {
        let mut context = String::new();
        let mut current_length = 0;

        for result in &self.search_results {
            if result.is_weak_match() {
                continue;
            }

            let content = &result.document.content;
            if current_length + content.len() > max_length {
                break;
            }

            if !context.is_empty() {
                context.push_str("\n\n---\n\n");
            }
            context.push_str(content);
            current_length += content.len();
        }

        self.context = context;
    }
}

impl Default for QueryResult {
    fn default() -> Self {
        Self::new("", ClassificationResult::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QueryCategory;
    use crate::QueryStrategy;

    #[test]
    fn test_search_result() {
        let doc = Document::new("Test content");
        let result = SearchResult::new(doc, 0.85).with_rank(0);

        assert!(result.is_relevant());
        assert!(!result.is_weak_match());
        assert_eq!(result.rank, 0);
    }

    #[test]
    fn test_query_result_builder() {
        let classification = ClassificationResult::new(
            QueryCategory::Factual,
            QueryStrategy::RagLocal,
            0.9,
        );

        let result = QueryResult::new("What is Rust?", classification)
            .with_context("Rust is a systems programming language")
            .with_processing_time(150);

        assert_eq!(result.query, "What is Rust?");
        assert!(!result.context.is_empty());
        assert_eq!(result.processing_time_ms, 150);
    }

    #[test]
    fn test_build_context() {
        let classification = ClassificationResult::default();
        let mut result = QueryResult::new("test", classification);

        result.search_results = vec![
            SearchResult::new(Document::new("First document"), 0.9),
            SearchResult::new(Document::new("Second document"), 0.8),
            SearchResult::new(Document::new("Weak match"), 0.3),
        ];

        result.build_context(1000);

        assert!(result.context.contains("First document"));
        assert!(result.context.contains("Second document"));
        assert!(!result.context.contains("Weak match")); // Low score excluded
    }
}
