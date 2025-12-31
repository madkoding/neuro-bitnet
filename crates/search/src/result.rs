//! Web search result types

use serde::{Deserialize, Serialize};

/// Result from a web search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    /// Title of the result
    pub title: String,

    /// URL of the source
    pub url: String,

    /// Short snippet/description
    pub snippet: String,

    /// Full content (if fetched)
    pub content: Option<String>,

    /// Source name (e.g., "Wikipedia")
    pub source: String,
}

impl WebSearchResult {
    /// Create a new search result
    pub fn new(
        title: impl Into<String>,
        url: impl Into<String>,
        snippet: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            url: url.into(),
            snippet: snippet.into(),
            content: None,
            source: source.into(),
        }
    }

    /// Set full content
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Check if full content is available
    pub fn has_content(&self) -> bool {
        self.content.is_some()
    }

    /// Get the best available text (content or snippet)
    pub fn best_text(&self) -> &str {
        self.content.as_deref().unwrap_or(&self.snippet)
    }

    /// Convert to a RAG-friendly string
    pub fn to_rag_context(&self) -> String {
        let mut context = format!("# {}\n", self.title);
        context.push_str(&format!("Source: {} ({})\n\n", self.source, self.url));
        context.push_str(self.best_text());
        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result() {
        let result = WebSearchResult::new(
            "Rust (programming language)",
            "https://en.wikipedia.org/wiki/Rust_(programming_language)",
            "Rust is a multi-paradigm programming language...",
            "Wikipedia",
        );

        assert_eq!(result.title, "Rust (programming language)");
        assert_eq!(result.source, "Wikipedia");
        assert!(!result.has_content());
    }

    #[test]
    fn test_with_content() {
        let result = WebSearchResult::new("Title", "https://example.com", "Short", "Source")
            .with_content("Full content here");

        assert!(result.has_content());
        assert_eq!(result.best_text(), "Full content here");
    }

    #[test]
    fn test_to_rag_context() {
        let result = WebSearchResult::new("Test", "https://test.com", "Content", "TestSource");
        let context = result.to_rag_context();

        assert!(context.contains("# Test"));
        assert!(context.contains("TestSource"));
        assert!(context.contains("Content"));
    }
}
