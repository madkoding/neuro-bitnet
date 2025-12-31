//! # neuro-classifier
//!
//! Query classification for the neuro-bitnet RAG system.
//!
//! Classifies user queries into categories to determine the optimal
//! handling strategy (direct LLM, RAG search, web search, etc.).
//!
//! ## Categories
//!
//! - **Math** - Mathematical expressions and calculations
//! - **Code** - Programming and code-related queries
//! - **Reasoning** - Logical analysis and reasoning tasks
//! - **Tools** - Tool usage and function calls
//! - **Greeting** - Social interactions and greetings
//! - **Factual** - Knowledge and fact-based queries
//! - **Conversational** - General conversation
//!
//! ## Example
//!
//! ```
//! use neuro_classifier::Classifier;
//!
//! let classifier = Classifier::new();
//! let result = classifier.classify("What is 2 + 2?");
//!
//! assert_eq!(result.category, neuro_core::QueryCategory::Math);
//! ```

mod classifier;
mod patterns;

pub use classifier::Classifier;
pub use patterns::{QueryPatterns, WeightedPattern, CompiledPattern};

/// Re-export core types
pub use neuro_core::{ClassificationResult, QueryCategory, QueryStrategy};
