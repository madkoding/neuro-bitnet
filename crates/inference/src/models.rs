//! BitNet model definitions and metadata
//!
//! Supported models from Microsoft's BitNet family.

use std::fmt;

/// Available BitNet models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitNetModel {
    /// BitNet b1.58 2B-4T (2B params, 4T training tokens) - Recommended
    B1_58_2B_4T,
    /// BitNet b1.58 Large (0.7B params) - Fast/lightweight  
    B1_58_Large,
    /// BitNet b1.58 3B (3.3B params)
    B1_58_3B,
    /// Llama3 8B 1.58-bit (8B params, 100B training tokens)
    Llama3_8B_1_58,
}

impl Default for BitNetModel {
    fn default() -> Self {
        Self::B1_58_2B_4T
    }
}

impl BitNetModel {
    /// Get all available models
    pub fn all() -> &'static [BitNetModel] {
        &[
            BitNetModel::B1_58_2B_4T,
            BitNetModel::B1_58_Large,
            BitNetModel::B1_58_3B,
            BitNetModel::Llama3_8B_1_58,
        ]
    }

    /// Model identifier string
    pub fn id(&self) -> &'static str {
        match self {
            Self::B1_58_2B_4T => "bitnet-b1.58-2b-4t",
            Self::B1_58_Large => "bitnet-b1.58-large",
            Self::B1_58_3B => "bitnet-b1.58-3b",
            Self::Llama3_8B_1_58 => "llama3-8b-1.58",
        }
    }

    /// Human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::B1_58_2B_4T => "BitNet b1.58 2B-4T",
            Self::B1_58_Large => "BitNet b1.58 Large",
            Self::B1_58_3B => "BitNet b1.58 3B",
            Self::Llama3_8B_1_58 => "Llama3 8B 1.58-bit",
        }
    }

    /// Model description
    pub fn description(&self) -> &'static str {
        match self {
            Self::B1_58_2B_4T => "2B parameters, trained on 4 trillion tokens. Best balance of quality and speed.",
            Self::B1_58_Large => "0.7B parameters. Fastest model, good for testing and low-resource systems.",
            Self::B1_58_3B => "3.3B parameters. Higher quality but slower.",
            Self::Llama3_8B_1_58 => "8B parameters, trained on 100B tokens. Highest quality, requires more resources.",
        }
    }

    /// Number of parameters (in billions)
    pub fn params_billions(&self) -> f32 {
        match self {
            Self::B1_58_2B_4T => 2.4,
            Self::B1_58_Large => 0.7,
            Self::B1_58_3B => 3.3,
            Self::Llama3_8B_1_58 => 8.0,
        }
    }

    /// GGUF file size in bytes
    pub fn size_bytes(&self) -> u64 {
        match self {
            Self::B1_58_2B_4T => 1_277_853_696,      // ~1.19 GB
            Self::B1_58_Large => 400_000_000,        // ~400 MB (estimate)
            Self::B1_58_3B => 1_800_000_000,         // ~1.8 GB (estimate)
            Self::Llama3_8B_1_58 => 4_500_000_000,   // ~4.5 GB (estimate)
        }
    }

    /// Human-readable file size
    pub fn size_human(&self) -> &'static str {
        match self {
            Self::B1_58_2B_4T => "1.19 GB",
            Self::B1_58_Large => "~400 MB",
            Self::B1_58_3B => "~1.8 GB",
            Self::Llama3_8B_1_58 => "~4.5 GB",
        }
    }

    /// GGUF filename
    pub fn filename(&self) -> &'static str {
        match self {
            Self::B1_58_2B_4T => "ggml-model-i2_s.gguf",
            Self::B1_58_Large => "bitnet-b1.58-large-i2_s.gguf",
            Self::B1_58_3B => "bitnet-b1.58-3b-i2_s.gguf",
            Self::Llama3_8B_1_58 => "llama3-8b-1.58-i2_s.gguf",
        }
    }

    /// Download URL for the GGUF file
    pub fn download_url(&self) -> &'static str {
        match self {
            Self::B1_58_2B_4T => "https://huggingface.co/microsoft/bitnet-b1.58-2B-4T-gguf/resolve/main/ggml-model-i2_s.gguf",
            Self::B1_58_Large => "https://huggingface.co/1bitLLM/bitnet_b1_58-large/resolve/main/ggml-model-i2_s.gguf",
            Self::B1_58_3B => "https://huggingface.co/1bitLLM/bitnet_b1_58-3B/resolve/main/ggml-model-i2_s.gguf",
            Self::Llama3_8B_1_58 => "https://huggingface.co/HF1BitLLM/Llama3-8B-1.58-100B-tokens/resolve/main/ggml-model-i2_s.gguf",
        }
    }

    /// HuggingFace repository
    pub fn hf_repo(&self) -> &'static str {
        match self {
            Self::B1_58_2B_4T => "microsoft/bitnet-b1.58-2B-4T-gguf",
            Self::B1_58_Large => "1bitLLM/bitnet_b1_58-large",
            Self::B1_58_3B => "1bitLLM/bitnet_b1_58-3B",
            Self::Llama3_8B_1_58 => "HF1BitLLM/Llama3-8B-1.58-100B-tokens",
        }
    }

    /// SHA256 checksum of the GGUF file (None if unknown)
    pub fn sha256(&self) -> Option<&'static str> {
        match self {
            // Verified checksums for official models
            Self::B1_58_2B_4T => Some("4221b252fdd5fd25e15847adfeb5ee88886506ba50b8a34548374492884c2162"),
            _ => None, // Other models need checksum verification
        }
    }

    /// Whether this model is verified and recommended
    pub fn is_verified(&self) -> bool {
        matches!(self, Self::B1_58_2B_4T)
    }

    /// Parse model from string identifier
    pub fn from_str(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "bitnet-b1.58-2b-4t" | "b1.58-2b-4t" | "2b-4t" | "2b" | "default" => Some(Self::B1_58_2B_4T),
            "bitnet-b1.58-large" | "b1.58-large" | "large" | "0.7b" => Some(Self::B1_58_Large),
            "bitnet-b1.58-3b" | "b1.58-3b" | "3b" => Some(Self::B1_58_3B),
            "llama3-8b-1.58" | "llama3-8b" | "8b" => Some(Self::Llama3_8B_1_58),
            _ => None,
        }
    }

    /// Detect model type from file path
    pub fn from_path(path: &str) -> Option<Self> {
        let path_lower = path.to_lowercase();
        
        if path_lower.contains("2b-4t") || path_lower.contains("2b4t") {
            return Some(Self::B1_58_2B_4T);
        }
        if path_lower.contains("large") || path_lower.contains("0.7b") {
            return Some(Self::B1_58_Large);
        }
        if path_lower.contains("3b") && !path_lower.contains("8b") {
            return Some(Self::B1_58_3B);
        }
        if path_lower.contains("8b") || path_lower.contains("llama3") {
            return Some(Self::Llama3_8B_1_58);
        }
        
        // Default for generic bitnet model paths
        if path_lower.contains("bitnet") || path_lower.contains("i2_s") {
            return Some(Self::B1_58_2B_4T);
        }
        
        None
    }
}

impl fmt::Display for BitNetModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_parsing() {
        assert_eq!(BitNetModel::from_str("2b"), Some(BitNetModel::B1_58_2B_4T));
        assert_eq!(BitNetModel::from_str("default"), Some(BitNetModel::B1_58_2B_4T));
        assert_eq!(BitNetModel::from_str("large"), Some(BitNetModel::B1_58_Large));
        assert_eq!(BitNetModel::from_str("8b"), Some(BitNetModel::Llama3_8B_1_58));
        assert_eq!(BitNetModel::from_str("unknown"), None);
    }

    #[test]
    fn test_model_from_path() {
        assert_eq!(
            BitNetModel::from_path("/models/bitnet-b1.58-2B-4T/model.gguf"),
            Some(BitNetModel::B1_58_2B_4T)
        );
        assert_eq!(
            BitNetModel::from_path("ggml-model-i2_s.gguf"),
            Some(BitNetModel::B1_58_2B_4T)
        );
    }

    #[test]
    fn test_default_model() {
        assert_eq!(BitNetModel::default(), BitNetModel::B1_58_2B_4T);
    }
}
