//! Context pool for concurrent request handling
//!
//! Manages a pool of LlamaContext instances for efficient reuse across requests.

use crate::error::{InferenceError, Result};
use crate::native::{LlamaContext, LlamaModel, ContextParams};
use crossbeam_channel::{bounded, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Configuration for the context pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum number of contexts to maintain (pre-allocated)
    pub min_size: usize,
    /// Maximum number of contexts allowed
    pub max_size: usize,
    /// Context parameters for created contexts
    pub context_params: ContextParams,
    /// Timeout for acquiring a context
    pub acquire_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        // Default to reasonable values based on system resources
        let num_cpus = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);
        
        Self {
            min_size: 2,
            max_size: num_cpus.min(4),
            context_params: ContextParams::default(),
            acquire_timeout: Duration::from_secs(30),
        }
    }
}

impl PoolConfig {
    /// Create config with specific pool sizes
    pub fn with_sizes(min_size: usize, max_size: usize) -> Self {
        Self {
            min_size,
            max_size,
            ..Default::default()
        }
    }

    /// Set context parameters
    pub fn with_context_params(mut self, params: ContextParams) -> Self {
        self.context_params = params;
        self
    }

    /// Set acquire timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = timeout;
        self
    }
}

/// A pool of LlamaContext instances for concurrent request handling
///
/// Contexts are expensive to create, so this pool maintains a set of
/// pre-allocated contexts that can be borrowed and returned.
pub struct ContextPool {
    /// Channel for available contexts
    available: Receiver<LlamaContext>,
    /// Channel to return contexts
    returns: Sender<LlamaContext>,
    /// The model shared across all contexts
    model: Arc<LlamaModel>,
    /// Pool configuration
    config: PoolConfig,
    /// Current number of contexts (including borrowed)
    current_size: std::sync::atomic::AtomicUsize,
}

impl ContextPool {
    /// Create a new context pool
    ///
    /// Pre-allocates `min_size` contexts immediately.
    pub fn new(model: Arc<LlamaModel>, config: PoolConfig) -> Result<Arc<Self>> {
        let (returns_tx, returns_rx) = bounded(config.max_size);
        let (available_tx, available_rx) = bounded(config.max_size);

        // Pre-allocate minimum contexts
        info!(
            "Initializing context pool: min={}, max={}",
            config.min_size, config.max_size
        );

        for i in 0..config.min_size {
            let ctx = LlamaContext::new(Arc::clone(&model), &config.context_params)?;
            available_tx.send(ctx).map_err(|_| {
                InferenceError::Context(format!("Failed to initialize context {}", i))
            })?;
        }

        // Spawn return handler thread
        let returns_rx_clone = returns_rx.clone();
        let available_tx_clone = available_tx.clone();
        std::thread::spawn(move || {
            while let Ok(mut ctx) = returns_rx_clone.recv() {
                // Clear KV cache before returning to pool
                ctx.kv_cache_clear();
                if available_tx_clone.send(ctx).is_err() {
                    break; // Pool was dropped
                }
            }
        });

        Ok(Arc::new(Self {
            available: available_rx,
            returns: returns_tx,
            model,
            config,
            current_size: std::sync::atomic::AtomicUsize::new(config.min_size),
        }))
    }

    /// Try to acquire a context without blocking
    ///
    /// Returns `None` if no context is immediately available.
    pub fn try_acquire(self: &Arc<Self>) -> Option<PooledContext> {
        match self.available.try_recv() {
            Ok(ctx) => Some(PooledContext {
                context: Some(ctx),
                pool: Arc::clone(self),
            }),
            Err(TryRecvError::Empty) => {
                // Try to create a new context if under max
                self.try_grow()
            }
            Err(TryRecvError::Disconnected) => None,
        }
    }

    /// Acquire a context, blocking until one is available
    ///
    /// Times out according to `config.acquire_timeout`.
    pub fn acquire(self: &Arc<Self>) -> Result<PooledContext> {
        // First try non-blocking
        if let Some(ctx) = self.try_acquire() {
            return Ok(ctx);
        }

        // Wait for a context with timeout
        match self.available.recv_timeout(self.config.acquire_timeout) {
            Ok(ctx) => Ok(PooledContext {
                context: Some(ctx),
                pool: Arc::clone(self),
            }),
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                Err(InferenceError::Context(format!(
                    "Timeout waiting for context ({}s)",
                    self.config.acquire_timeout.as_secs()
                )))
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                Err(InferenceError::Context("Context pool closed".to_string()))
            }
        }
    }

    /// Try to grow the pool by one context
    fn try_grow(self: &Arc<Self>) -> Option<PooledContext> {
        use std::sync::atomic::Ordering;

        let current = self.current_size.load(Ordering::SeqCst);
        if current >= self.config.max_size {
            debug!("Pool at max capacity ({}), cannot grow", self.config.max_size);
            return None;
        }

        // Try to increment atomically
        if self
            .current_size
            .compare_exchange(current, current + 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return None; // Another thread won the race
        }

        // Create new context
        match LlamaContext::new(Arc::clone(&self.model), &self.config.context_params) {
            Ok(ctx) => {
                info!("Grew context pool to {} contexts", current + 1);
                Some(PooledContext {
                    context: Some(ctx),
                    pool: Arc::clone(self),
                })
            }
            Err(e) => {
                warn!("Failed to grow pool: {}", e);
                self.current_size.fetch_sub(1, Ordering::SeqCst);
                None
            }
        }
    }

    /// Return a context to the pool
    fn return_context(&self, ctx: LlamaContext) {
        if self.returns.send(ctx).is_err() {
            warn!("Failed to return context to pool (pool closed?)");
        }
    }

    /// Get the model shared by all contexts
    pub fn model(&self) -> &Arc<LlamaModel> {
        &self.model
    }

    /// Get current pool size
    pub fn size(&self) -> usize {
        self.current_size.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Get number of available contexts
    pub fn available(&self) -> usize {
        self.available.len()
    }
}

/// RAII guard for a pooled context
///
/// Automatically returns the context to the pool when dropped.
pub struct PooledContext {
    context: Option<LlamaContext>,
    pool: Arc<ContextPool>,
}

impl PooledContext {
    /// Get a reference to the context
    pub fn get(&self) -> &LlamaContext {
        self.context.as_ref().expect("Context already returned")
    }

    /// Get a mutable reference to the context
    pub fn get_mut(&mut self) -> &mut LlamaContext {
        self.context.as_mut().expect("Context already returned")
    }
}

impl std::ops::Deref for PooledContext {
    type Target = LlamaContext;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl std::ops::DerefMut for PooledContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl Drop for PooledContext {
    fn drop(&mut self) {
        if let Some(ctx) = self.context.take() {
            self.pool.return_context(ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.min_size, 2);
        assert!(config.max_size >= 2);
    }

    #[test]
    fn test_pool_config_builder() {
        let config = PoolConfig::with_sizes(1, 8)
            .with_timeout(Duration::from_secs(60));
        
        assert_eq!(config.min_size, 1);
        assert_eq!(config.max_size, 8);
        assert_eq!(config.acquire_timeout, Duration::from_secs(60));
    }
}
