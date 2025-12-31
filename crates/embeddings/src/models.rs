//! Embedding model definitions

use serde::{Deserialize, Serialize};

/// Available embedding models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingModel {
    /// all-MiniLM-L6-v2 (384 dimensions, fast, good quality)
    AllMiniLmL6V2,
    
    /// all-MiniLM-L12-v2 (384 dimensions, slightly better than L6)
    AllMiniLmL12V2,
    
    /// all-mpnet-base-v2 (768 dimensions, high quality)
    AllMpnetBaseV2,
    
    /// BGE-small-en-v1.5 (384 dimensions, optimized for English)
    BgeSmallEnV15,
    
    /// BGE-base-en-v1.5 (768 dimensions, balanced)
    BgeBaseEnV15,
    
    /// BGE-large-en-v1.5 (1024 dimensions, highest quality)
    BgeLargeEnV15,
    
    /// Multilingual-e5-small (384 dimensions, multilingual)
    MultilingualE5Small,
    
    /// Multilingual-e5-base (768 dimensions, multilingual)
    MultilingualE5Base,
    
    /// Multilingual-e5-large (1024 dimensions, multilingual, highest quality)
    MultilingualE5Large,
}

impl EmbeddingModel {
    /// Get the embedding dimension for this model
    pub fn dimension(&self) -> usize {
        match self {
            Self::AllMiniLmL6V2 => 384,
            Self::AllMiniLmL12V2 => 384,
            Self::AllMpnetBaseV2 => 768,
            Self::BgeSmallEnV15 => 384,
            Self::BgeBaseEnV15 => 768,
            Self::BgeLargeEnV15 => 1024,
            Self::MultilingualE5Small => 384,
            Self::MultilingualE5Base => 768,
            Self::MultilingualE5Large => 1024,
        }
    }

    /// Get the model name as it appears in fastembed
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::AllMiniLmL6V2 => "all-MiniLM-L6-v2",
            Self::AllMiniLmL12V2 => "all-MiniLM-L12-v2",
            Self::AllMpnetBaseV2 => "all-mpnet-base-v2",
            Self::BgeSmallEnV15 => "BGE-small-en-v1.5",
            Self::BgeBaseEnV15 => "BGE-base-en-v1.5",
            Self::BgeLargeEnV15 => "BGE-large-en-v1.5",
            Self::MultilingualE5Small => "multilingual-e5-small",
            Self::MultilingualE5Base => "multilingual-e5-base",
            Self::MultilingualE5Large => "multilingual-e5-large",
        }
    }

    /// Check if this model supports multiple languages
    pub fn is_multilingual(&self) -> bool {
        matches!(
            self,
            Self::MultilingualE5Small | Self::MultilingualE5Base | Self::MultilingualE5Large
        )
    }

    /// Get relative speed (1-5, higher is faster)
    pub fn speed_rating(&self) -> u8 {
        match self {
            Self::AllMiniLmL6V2 => 5,
            Self::AllMiniLmL12V2 => 4,
            Self::BgeSmallEnV15 => 5,
            Self::MultilingualE5Small => 4,
            Self::AllMpnetBaseV2 => 3,
            Self::BgeBaseEnV15 => 3,
            Self::MultilingualE5Base => 3,
            Self::BgeLargeEnV15 => 2,
            Self::MultilingualE5Large => 1,
        }
    }

    /// Get relative quality (1-5, higher is better)
    pub fn quality_rating(&self) -> u8 {
        match self {
            Self::AllMiniLmL6V2 => 3,
            Self::AllMiniLmL12V2 => 3,
            Self::BgeSmallEnV15 => 3,
            Self::MultilingualE5Small => 3,
            Self::AllMpnetBaseV2 => 4,
            Self::BgeBaseEnV15 => 4,
            Self::MultilingualE5Base => 4,
            Self::BgeLargeEnV15 => 5,
            Self::MultilingualE5Large => 5,
        }
    }
}

impl Default for EmbeddingModel {
    fn default() -> Self {
        Self::AllMiniLmL6V2
    }
}

impl std::fmt::Display for EmbeddingModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.model_name())
    }
}

impl std::str::FromStr for EmbeddingModel {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all-minilm-l6-v2" | "minilm" | "minilm-l6" => Ok(Self::AllMiniLmL6V2),
            "all-minilm-l12-v2" | "minilm-l12" => Ok(Self::AllMiniLmL12V2),
            "all-mpnet-base-v2" | "mpnet" => Ok(Self::AllMpnetBaseV2),
            "bge-small-en-v1.5" | "bge-small" => Ok(Self::BgeSmallEnV15),
            "bge-base-en-v1.5" | "bge-base" => Ok(Self::BgeBaseEnV15),
            "bge-large-en-v1.5" | "bge-large" | "bge" => Ok(Self::BgeLargeEnV15),
            "multilingual-e5-small" | "e5-small" => Ok(Self::MultilingualE5Small),
            "multilingual-e5-base" | "e5-base" => Ok(Self::MultilingualE5Base),
            "multilingual-e5-large" | "e5-large" | "e5" => Ok(Self::MultilingualE5Large),
            _ => Err(format!("Unknown model: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_dimensions() {
        assert_eq!(EmbeddingModel::AllMiniLmL6V2.dimension(), 384);
        assert_eq!(EmbeddingModel::AllMpnetBaseV2.dimension(), 768);
        assert_eq!(EmbeddingModel::BgeLargeEnV15.dimension(), 1024);
    }

    #[test]
    fn test_model_parsing() {
        assert_eq!("minilm".parse::<EmbeddingModel>().unwrap(), EmbeddingModel::AllMiniLmL6V2);
        assert_eq!("bge".parse::<EmbeddingModel>().unwrap(), EmbeddingModel::BgeLargeEnV15);
        assert_eq!("e5".parse::<EmbeddingModel>().unwrap(), EmbeddingModel::MultilingualE5Large);
    }

    #[test]
    fn test_multilingual() {
        assert!(EmbeddingModel::MultilingualE5Large.is_multilingual());
        assert!(!EmbeddingModel::AllMiniLmL6V2.is_multilingual());
    }
}
