//! Query classification types

use serde::{Deserialize, Serialize};

/// Categories for classifying user queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryCategory {
    /// Mathematical calculations or expressions
    Math,
    /// Programming/code related queries
    Code,
    /// Logical reasoning or analysis
    Reasoning,
    /// Tool usage or function calls
    Tools,
    /// Greetings or social interactions
    Greeting,
    /// Factual knowledge queries
    Factual,
    /// General conversation
    Conversational,
}

impl Default for QueryCategory {
    fn default() -> Self {
        Self::Conversational
    }
}

impl std::fmt::Display for QueryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Math => write!(f, "math"),
            Self::Code => write!(f, "code"),
            Self::Reasoning => write!(f, "reasoning"),
            Self::Tools => write!(f, "tools"),
            Self::Greeting => write!(f, "greeting"),
            Self::Factual => write!(f, "factual"),
            Self::Conversational => write!(f, "conversational"),
        }
    }
}

/// Strategy for handling a query based on classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryStrategy {
    /// Send directly to LLM without RAG
    LlmDirect,
    /// Use local RAG storage
    RagLocal,
    /// Use RAG, then fall back to web search
    RagThenWeb,
    /// Use web search directly
    WebSearch,
}

impl Default for QueryStrategy {
    fn default() -> Self {
        Self::RagLocal
    }
}

impl std::fmt::Display for QueryStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LlmDirect => write!(f, "llm_direct"),
            Self::RagLocal => write!(f, "rag_local"),
            Self::RagThenWeb => write!(f, "rag_then_web"),
            Self::WebSearch => write!(f, "web_search"),
        }
    }
}

/// Result of query classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// The determined category
    pub category: QueryCategory,

    /// The recommended strategy
    pub strategy: QueryStrategy,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Reasons for the classification
    #[serde(default)]
    pub reasons: Vec<String>,

    /// Original query text
    #[serde(default)]
    pub query: String,
}

impl ClassificationResult {
    /// Create a new classification result
    pub fn new(category: QueryCategory, strategy: QueryStrategy, confidence: f32) -> Self {
        Self {
            category,
            strategy,
            confidence,
            reasons: Vec::new(),
            query: String::new(),
        }
    }

    /// Add a reason for the classification
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reasons.push(reason.into());
        self
    }

    /// Add multiple reasons
    pub fn with_reasons(mut self, reasons: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.reasons.extend(reasons.into_iter().map(|r| r.into()));
        self
    }

    /// Set the original query
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }

    /// Check if classification is high confidence (>= 0.7)
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.7
    }

    /// Check if classification is low confidence (< 0.4)
    pub fn is_low_confidence(&self) -> bool {
        self.confidence < 0.4
    }
}

impl Default for ClassificationResult {
    fn default() -> Self {
        Self::new(
            QueryCategory::default(),
            QueryStrategy::default(),
            0.5,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_category_display() {
        assert_eq!(QueryCategory::Math.to_string(), "math");
        assert_eq!(QueryCategory::Code.to_string(), "code");
        assert_eq!(QueryCategory::Greeting.to_string(), "greeting");
    }

    #[test]
    fn test_query_strategy_display() {
        assert_eq!(QueryStrategy::LlmDirect.to_string(), "llm_direct");
        assert_eq!(QueryStrategy::RagLocal.to_string(), "rag_local");
    }

    #[test]
    fn test_classification_result_builder() {
        let result = ClassificationResult::new(QueryCategory::Math, QueryStrategy::LlmDirect, 0.95)
            .with_reason("Contains mathematical expression")
            .with_query("What is 2 + 2?");

        assert_eq!(result.category, QueryCategory::Math);
        assert!(result.is_high_confidence());
        assert_eq!(result.reasons.len(), 1);
        assert_eq!(result.query, "What is 2 + 2?");
    }

    #[test]
    fn test_confidence_thresholds() {
        let high = ClassificationResult::new(QueryCategory::Code, QueryStrategy::RagLocal, 0.85);
        let low = ClassificationResult::new(QueryCategory::Code, QueryStrategy::RagLocal, 0.3);

        assert!(high.is_high_confidence());
        assert!(!high.is_low_confidence());
        assert!(!low.is_high_confidence());
        assert!(low.is_low_confidence());
    }

    #[test]
    fn test_serialization() {
        let result = ClassificationResult::new(QueryCategory::Factual, QueryStrategy::RagThenWeb, 0.75);
        let json = serde_json::to_string(&result).unwrap();
        let parsed: ClassificationResult = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.category, QueryCategory::Factual);
        assert_eq!(parsed.strategy, QueryStrategy::RagThenWeb);
    }
}
