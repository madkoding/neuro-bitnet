//! Sampler configuration for text generation

/// Configuration for token sampling
#[derive(Debug, Clone)]
pub struct SamplerConfig {
    /// Temperature for randomness (0.0 = greedy, higher = more random)
    pub temperature: f32,
    /// Top-K sampling: consider only top K tokens
    pub top_k: i32,
    /// Top-P (nucleus) sampling: consider tokens with cumulative probability <= p
    pub top_p: f32,
    /// Min-P sampling: minimum probability threshold
    pub min_p: f32,
    /// Repetition penalty
    pub repeat_penalty: f32,
    /// Number of tokens to consider for repetition penalty
    pub repeat_last_n: i32,
    /// Random seed (0 = random)
    pub seed: u32,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_k: 40,
            top_p: 0.95,
            min_p: 0.05,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            seed: 0,
        }
    }
}

impl SamplerConfig {
    /// Greedy sampling (deterministic)
    pub fn greedy() -> Self {
        Self {
            temperature: 0.0,
            top_k: 1,
            top_p: 1.0,
            min_p: 0.0,
            repeat_penalty: 1.0,
            repeat_last_n: 0,
            seed: 0,
        }
    }

    /// Creative sampling (more random)
    pub fn creative() -> Self {
        Self {
            temperature: 0.9,
            top_k: 50,
            top_p: 0.95,
            min_p: 0.02,
            repeat_penalty: 1.15,
            repeat_last_n: 128,
            seed: 0,
        }
    }

    /// Balanced sampling (default)
    pub fn balanced() -> Self {
        Self::default()
    }

    /// Set temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }
}
