//! Query classifier implementation

use neuro_core::{ClassificationResult, QueryCategory, QueryStrategy};
use tracing::debug;

use crate::patterns::{QueryPatterns, PATTERNS};

/// Query classifier using regex pattern matching
pub struct Classifier {
    /// Minimum confidence threshold for a match
    confidence_threshold: f32,
}

impl Classifier {
    /// Create a new classifier with default settings
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.3,
        }
    }

    /// Create a classifier with custom confidence threshold
    pub fn with_threshold(confidence_threshold: f32) -> Self {
        Self {
            confidence_threshold: confidence_threshold.clamp(0.0, 1.0),
        }
    }

    /// Classify a query into a category with recommended strategy
    pub fn classify(&self, query: &str) -> ClassificationResult {
        let query = query.trim();
        
        if query.is_empty() {
            return ClassificationResult::new(
                QueryCategory::Conversational,
                QueryStrategy::LlmDirect,
                0.0,
            )
            .with_query(query);
        }

        debug!("Classifying query: {}", query);

        // Count matches for each category
        let scores = self.score_categories(query);

        // Find the best category
        let (category, score, reasons) = self.select_best_category(&scores);

        // Determine strategy based on category
        let strategy = self.determine_strategy(category, score);

        let confidence = self.normalize_confidence(score);

        debug!(
            "Classification: {:?} (confidence: {:.2}, strategy: {:?})",
            category, confidence, strategy
        );

        ClassificationResult::new(category, strategy, confidence)
            .with_reasons(reasons)
            .with_query(query)
    }

    fn score_categories(&self, query: &str) -> CategoryScores {
        CategoryScores {
            math: QueryPatterns::score_category(&PATTERNS.math, query),
            code: QueryPatterns::score_category(&PATTERNS.code, query),
            reasoning: QueryPatterns::score_category(&PATTERNS.reasoning, query),
            tools: QueryPatterns::score_category(&PATTERNS.tools, query),
            greeting: QueryPatterns::score_category(&PATTERNS.greeting, query),
            factual: QueryPatterns::score_category(&PATTERNS.factual, query),
        }
    }

    fn select_best_category(&self, scores: &CategoryScores) -> (QueryCategory, f32, Vec<String>) {
        let mut best = (QueryCategory::Conversational, 0.0_f32, vec!["Default category".to_string()]);

        // Priority order matters for tie-breaking
        let categories = [
            (QueryCategory::Greeting, scores.greeting, "Greeting patterns matched"),
            (QueryCategory::Math, scores.math, "Mathematical patterns matched"),
            (QueryCategory::Code, scores.code, "Programming patterns matched"),
            (QueryCategory::Tools, scores.tools, "Tool usage patterns matched"),
            (QueryCategory::Reasoning, scores.reasoning, "Reasoning patterns matched"),
            (QueryCategory::Factual, scores.factual, "Factual query patterns matched"),
        ];

        for (category, score, reason) in categories {
            if score > best.1 {
                best = (category, score, vec![reason.to_string()]);
            } else if (score - best.1).abs() < 0.01 && score > 0.0 {
                // Add to reasons if tie (within floating point tolerance)
                best.2.push(reason.to_string());
            }
        }

        best
    }

    fn determine_strategy(&self, category: QueryCategory, score: f32) -> QueryStrategy {
        match category {
            // Direct to LLM (no RAG needed)
            QueryCategory::Math => QueryStrategy::LlmDirect,
            QueryCategory::Greeting => QueryStrategy::LlmDirect,
            
            // Code often benefits from RAG (documentation, examples)
            QueryCategory::Code => {
                if score >= 3.0 {
                    QueryStrategy::RagLocal
                } else {
                    QueryStrategy::LlmDirect
                }
            }
            
            // Reasoning might need context
            QueryCategory::Reasoning => QueryStrategy::RagLocal,
            
            // Tools might need web search
            QueryCategory::Tools => QueryStrategy::RagThenWeb,
            
            // Factual queries benefit from RAG + web
            QueryCategory::Factual => QueryStrategy::RagThenWeb,
            
            // Default: try local RAG first
            QueryCategory::Conversational => QueryStrategy::RagLocal,
        }
    }

    fn normalize_confidence(&self, score: f32) -> f32 {
        // Map weighted score to 0.0-1.0 range
        // Score is now weighted, so we use thresholds
        if score <= 0.0 {
            0.3 // Default category
        } else if score < 1.5 {
            0.5
        } else if score < 3.0 {
            0.7
        } else if score < 5.0 {
            0.85
        } else {
            0.95
        }
    }
}

impl Default for Classifier {
    fn default() -> Self {
        Self::new()
    }
}

struct CategoryScores {
    math: f32,
    code: f32,
    reasoning: f32,
    tools: f32,
    greeting: f32,
    factual: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify(query: &str) -> ClassificationResult {
        Classifier::new().classify(query)
    }

    #[test]
    fn test_math_classification() {
        let result = classify("What is 2 + 2?");
        assert_eq!(result.category, QueryCategory::Math);
        assert_eq!(result.strategy, QueryStrategy::LlmDirect);
    }

    #[test]
    fn test_math_calculation() {
        let result = classify("Calculate the derivative of x^2");
        assert_eq!(result.category, QueryCategory::Math);
    }

    #[test]
    fn test_code_classification() {
        let result = classify("Write a Python function to sort a list");
        assert_eq!(result.category, QueryCategory::Code);
    }

    #[test]
    fn test_code_debug() {
        let result = classify("Fix the bug in my JavaScript code");
        assert_eq!(result.category, QueryCategory::Code);
    }

    #[test]
    fn test_greeting_hello() {
        let result = classify("Hello!");
        assert_eq!(result.category, QueryCategory::Greeting);
        assert_eq!(result.strategy, QueryStrategy::LlmDirect);
    }

    #[test]
    fn test_greeting_how_are_you() {
        let result = classify("How are you doing?");
        assert_eq!(result.category, QueryCategory::Greeting);
    }

    #[test]
    fn test_factual_capital() {
        let result = classify("What is the capital of France?");
        assert_eq!(result.category, QueryCategory::Factual);
        assert_eq!(result.strategy, QueryStrategy::RagThenWeb);
    }

    #[test]
    fn test_factual_who() {
        let result = classify("Who was Albert Einstein?");
        assert_eq!(result.category, QueryCategory::Factual);
    }

    #[test]
    fn test_tools_search() {
        let result = classify("Search the web for latest news");
        assert_eq!(result.category, QueryCategory::Tools);
        assert_eq!(result.strategy, QueryStrategy::RagThenWeb);
    }

    #[test]
    fn test_tools_translate() {
        let result = classify("Translate 'hello' to Spanish");
        assert_eq!(result.category, QueryCategory::Tools);
    }

    #[test]
    fn test_reasoning_analysis() {
        let result = classify("Analyze the pros and cons of remote work");
        assert_eq!(result.category, QueryCategory::Reasoning);
    }

    #[test]
    fn test_conversational_default() {
        let result = classify("I like pizza");
        assert_eq!(result.category, QueryCategory::Conversational);
    }

    #[test]
    fn test_empty_query() {
        let result = classify("");
        assert_eq!(result.category, QueryCategory::Conversational);
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn test_confidence_levels() {
        // Strong match (multiple patterns)
        let math_result = classify("Calculate the sum of 1 + 2 + 3, what is the average?");
        assert!(math_result.confidence >= 0.7);

        // Weak match
        let weak_result = classify("interesting topic");
        assert!(weak_result.confidence <= 0.5);
    }

    #[test]
    fn test_classification_result_fields() {
        let result = classify("What is Rust programming language?");
        
        assert!(!result.query.is_empty());
        assert!(!result.reasons.is_empty());
    }
}
