//! Web searcher trait

use async_trait::async_trait;
use crate::error::Result;
use crate::result::WebSearchResult;

/// Trait for web search implementations
#[async_trait]
pub trait WebSearcher: Send + Sync {
    /// Get the name of this search provider
    fn name(&self) -> &str;

    /// Search for results
    ///
    /// # Arguments
    /// * `query` - Search query string
    /// * `max_results` - Maximum number of results to return
    async fn search(&self, query: &str, max_results: usize) -> Result<Vec<WebSearchResult>>;

    /// Fetch full content for a search result
    async fn fetch_content(&self, result: &WebSearchResult) -> Result<String>;
}
