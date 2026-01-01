//! LlamaBatch wrapper for bitnet.cpp
//!
//! Safe Rust wrapper around the llama_batch FFI type.

use crate::error::{InferenceError, Result};
use bitnet_sys::*;

/// Safe wrapper around llama_batch
///
/// Manages a batch of tokens for processing. Batches are used to
/// efficiently process multiple tokens at once during inference.
pub struct LlamaBatch {
    batch: llama_batch,
    capacity: usize,
    n_tokens: usize,
}

// SAFETY: Batch data is owned and managed by this struct
unsafe impl Send for LlamaBatch {}

impl LlamaBatch {
    /// Create a new batch with specified capacity
    ///
    /// # Arguments
    /// * `n_tokens` - Maximum number of tokens the batch can hold
    /// * `n_seq_max` - Maximum number of sequences (usually 1)
    pub fn new(n_tokens: usize, n_seq_max: i32) -> Result<Self> {
        let batch = unsafe { llama_batch_init(n_tokens as i32, 0, n_seq_max) };

        // Check if allocation succeeded
        if batch.token.is_null() {
            return Err(InferenceError::Context(
                "Failed to allocate batch memory".to_string(),
            ));
        }

        Ok(Self {
            batch,
            capacity: n_tokens,
            n_tokens: 0,
        })
    }

    /// Add a token to the batch
    ///
    /// # Arguments
    /// * `token` - Token ID to add
    /// * `pos` - Position in the sequence
    /// * `seq_ids` - Sequence IDs this token belongs to
    /// * `logits` - Whether to compute logits for this token
    pub fn add(
        &mut self,
        token: llama_token,
        pos: llama_pos,
        seq_ids: &[llama_seq_id],
        logits: bool,
    ) -> Result<()> {
        if self.n_tokens >= self.capacity {
            return Err(InferenceError::Context(format!(
                "Batch full: {} tokens (capacity: {})",
                self.n_tokens, self.capacity
            )));
        }

        let i = self.n_tokens;

        unsafe {
            // Set token
            *self.batch.token.add(i) = token;
            
            // Set position
            *self.batch.pos.add(i) = pos;
            
            // Set sequence count and IDs
            *self.batch.n_seq_id.add(i) = seq_ids.len() as i32;
            
            let seq_id_ptr = *self.batch.seq_id.add(i);
            for (j, &seq_id) in seq_ids.iter().enumerate() {
                *seq_id_ptr.add(j) = seq_id;
            }
            
            // Set logits flag
            *self.batch.logits.add(i) = logits as i8;
        }

        self.n_tokens += 1;
        self.batch.n_tokens = self.n_tokens as i32;

        Ok(())
    }

    /// Clear all tokens from the batch
    pub fn clear(&mut self) {
        self.n_tokens = 0;
        self.batch.n_tokens = 0;
    }

    /// Get the number of tokens in the batch
    pub fn len(&self) -> usize {
        self.n_tokens
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.n_tokens == 0
    }

    /// Get the batch capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the raw batch struct (for FFI calls)
    pub(crate) fn as_raw(&self) -> llama_batch {
        self.batch
    }

    /// Add multiple tokens at once (for prompt processing)
    ///
    /// # Arguments
    /// * `tokens` - Slice of token IDs
    /// * `start_pos` - Starting position for the first token
    /// * `seq_id` - Sequence ID for all tokens
    /// * `logits_last_only` - Only compute logits for the last token
    pub fn add_sequence(
        &mut self,
        tokens: &[llama_token],
        start_pos: llama_pos,
        seq_id: llama_seq_id,
        logits_last_only: bool,
    ) -> Result<()> {
        if self.n_tokens + tokens.len() > self.capacity {
            return Err(InferenceError::Context(format!(
                "Cannot add {} tokens: would exceed capacity ({} + {} > {})",
                tokens.len(),
                self.n_tokens,
                tokens.len(),
                self.capacity
            )));
        }

        let seq_ids = [seq_id];
        let last_idx = tokens.len().saturating_sub(1);

        for (i, &token) in tokens.iter().enumerate() {
            let pos = start_pos + i as i32;
            let logits = if logits_last_only {
                i == last_idx
            } else {
                true
            };
            self.add(token, pos, &seq_ids, logits)?;
        }

        Ok(())
    }
}

impl Drop for LlamaBatch {
    fn drop(&mut self) {
        unsafe {
            llama_batch_free(self.batch);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_operations() {
        // Skip if native bindings not available
        if !bitnet_sys::is_available() {
            return;
        }

        let mut batch = LlamaBatch::new(512, 1).unwrap();
        assert!(batch.is_empty());
        assert_eq!(batch.capacity(), 512);

        // Add a token
        batch.add(1, 0, &[0], true).unwrap();
        assert_eq!(batch.len(), 1);

        // Clear
        batch.clear();
        assert!(batch.is_empty());
    }
}
