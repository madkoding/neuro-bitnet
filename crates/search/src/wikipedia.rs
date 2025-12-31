//! Wikipedia search implementation

use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::time::Duration;
use tracing::{debug, warn};
use url::Url;

use crate::error::{Result, SearchError};
use crate::result::WebSearchResult;
use crate::searcher::WebSearcher;

/// Wikipedia search configuration
#[derive(Debug, Clone)]
pub struct WikipediaConfig {
    /// Request timeout
    pub timeout: Duration,
    /// Language code (e.g., "en", "es")
    pub language: String,
    /// Maximum content length to fetch
    pub max_content_length: usize,
}

impl Default for WikipediaConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            language: "en".to_string(),
            max_content_length: 10000,
        }
    }
}

/// Wikipedia search provider
pub struct WikipediaSearcher {
    client: Client,
    config: WikipediaConfig,
}

impl WikipediaSearcher {
    /// Create a new Wikipedia searcher with default config
    pub fn new() -> Self {
        Self::with_config(WikipediaConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: WikipediaConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent("neuro-bitnet/0.1 (RAG system)")
            .build()
            .expect("Failed to build HTTP client");

        Self { client, config }
    }

    /// Create with specific language
    pub fn with_language(language: impl Into<String>) -> Self {
        let mut config = WikipediaConfig::default();
        config.language = language.into();
        Self::with_config(config)
    }

    fn api_url(&self) -> String {
        format!(
            "https://{}.wikipedia.org/w/api.php",
            self.config.language
        )
    }

    fn article_url(&self, title: &str) -> String {
        format!(
            "https://{}.wikipedia.org/wiki/{}",
            self.config.language,
            urlencoding::encode(title)
        )
    }
}

impl Default for WikipediaSearcher {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct WikiSearchResponse {
    query: Option<WikiQuery>,
}

#[derive(Debug, Deserialize)]
struct WikiQuery {
    search: Option<Vec<WikiSearchResult>>,
    pages: Option<std::collections::HashMap<String, WikiPage>>,
}

#[derive(Debug, Deserialize)]
struct WikiSearchResult {
    title: String,
    snippet: String,
    pageid: u64,
}

#[derive(Debug, Deserialize)]
struct WikiPage {
    title: String,
    extract: Option<String>,
}

#[async_trait]
impl WebSearcher for WikipediaSearcher {
    fn name(&self) -> &str {
        "Wikipedia"
    }

    async fn search(&self, query: &str, max_results: usize) -> Result<Vec<WebSearchResult>> {
        if query.trim().is_empty() {
            return Err(SearchError::InvalidQuery("Empty query".into()));
        }

        debug!("Searching Wikipedia for: {}", query);

        let url = Url::parse_with_params(
            &self.api_url(),
            &[
                ("action", "query"),
                ("list", "search"),
                ("srsearch", query),
                ("srlimit", &max_results.to_string()),
                ("format", "json"),
                ("utf8", "1"),
            ],
        )
        .map_err(|e| SearchError::Parse(e.to_string()))?;

        let response = self
            .client
            .get(url)
            .send()
            .await?
            .json::<WikiSearchResponse>()
            .await?;

        let search_results = response
            .query
            .and_then(|q| q.search)
            .unwrap_or_default();

        if search_results.is_empty() {
            return Err(SearchError::NoResults(query.to_string()));
        }

        let results: Vec<WebSearchResult> = search_results
            .into_iter()
            .map(|r| {
                // Clean HTML from snippet
                let snippet = clean_html(&r.snippet);
                
                WebSearchResult::new(
                    r.title.clone(),
                    self.article_url(&r.title),
                    snippet,
                    "Wikipedia",
                )
            })
            .collect();

        debug!("Found {} Wikipedia results", results.len());
        Ok(results)
    }

    async fn fetch_content(&self, result: &WebSearchResult) -> Result<String> {
        debug!("Fetching Wikipedia content for: {}", result.title);

        // Use the extracts API for clean text
        let url = Url::parse_with_params(
            &self.api_url(),
            &[
                ("action", "query"),
                ("titles", &result.title),
                ("prop", "extracts"),
                ("exintro", "false"),
                ("explaintext", "true"),
                ("exsectionformat", "plain"),
                ("format", "json"),
                ("utf8", "1"),
            ],
        )
        .map_err(|e| SearchError::Parse(e.to_string()))?;

        let response = self
            .client
            .get(url)
            .send()
            .await?
            .json::<WikiSearchResponse>()
            .await?;

        let pages = response
            .query
            .and_then(|q| q.pages)
            .ok_or_else(|| SearchError::Parse("No pages in response".into()))?;

        // Get the first (and usually only) page
        let page = pages
            .values()
            .next()
            .ok_or_else(|| SearchError::NoResults(result.title.clone()))?;

        let content = page
            .extract
            .as_ref()
            .ok_or_else(|| SearchError::Parse("No extract available".into()))?;

        // Truncate if too long
        let content = if content.len() > self.config.max_content_length {
            let truncated = &content[..self.config.max_content_length];
            // Try to truncate at a sentence boundary
            if let Some(pos) = truncated.rfind(". ") {
                format!("{}...", &truncated[..=pos])
            } else {
                format!("{}...", truncated)
            }
        } else {
            content.clone()
        };

        Ok(content)
    }
}

/// Clean HTML tags from text
fn clean_html(html: &str) -> String {
    let fragment = Html::parse_fragment(html);
    let text: String = fragment.root_element().text().collect();
    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_html() {
        assert_eq!(
            clean_html("<span class=\"searchmatch\">Rust</span> is a language"),
            "Rust is a language"
        );
        assert_eq!(
            clean_html("Plain text"),
            "Plain text"
        );
    }

    #[test]
    fn test_config_default() {
        let config = WikipediaConfig::default();
        assert_eq!(config.language, "en");
        assert_eq!(config.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_article_url() {
        let searcher = WikipediaSearcher::new();
        let url = searcher.article_url("Rust (programming language)");
        assert!(url.contains("wikipedia.org/wiki/"));
        assert!(url.contains("Rust"));
    }

    #[test]
    fn test_api_url() {
        let searcher = WikipediaSearcher::with_language("es");
        assert!(searcher.api_url().contains("es.wikipedia.org"));
    }

    // Integration test - requires network
    #[tokio::test]
    #[ignore = "Requires network"]
    async fn test_search_integration() {
        let searcher = WikipediaSearcher::new();
        let results = searcher.search("Rust programming language", 3).await.unwrap();
        
        assert!(!results.is_empty());
        assert!(results[0].title.contains("Rust"));
    }
}
