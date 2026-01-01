//! Native FFI backend for bitnet.cpp
//!
//! This module provides direct bindings to bitnet.cpp for maximum performance.
//! Falls back gracefully to subprocess backend if native bindings are unavailable.

#[cfg(feature = "native")]
mod model;
#[cfg(feature = "native")]
mod context;
#[cfg(feature = "native")]
mod sampler;
#[cfg(feature = "native")]
mod batch;
#[cfg(feature = "native")]
mod pool;
#[cfg(feature = "native")]
mod backend;

#[cfg(feature = "native")]
pub use self::model::{LlamaModel, ModelParams};
#[cfg(feature = "native")]
pub use self::context::{LlamaContext, ContextParams};
#[cfg(feature = "native")]
pub use self::sampler::LlamaSampler;
#[cfg(feature = "native")]
pub use self::batch::LlamaBatch;
#[cfg(feature = "native")]
pub use self::pool::{ContextPool, PooledContext, PoolConfig};
#[cfg(feature = "native")]
pub use self::backend::NativeBackend;

/// Check if native bindings are available and functional
pub fn is_available() -> bool {
    #[cfg(feature = "native")]
    {
        bitnet_sys::is_available()
    }
    #[cfg(not(feature = "native"))]
    {
        false
    }
}

/// Get the native backend type description
pub fn backend_type() -> &'static str {
    #[cfg(feature = "native")]
    {
        bitnet_sys::backend_type()
    }
    #[cfg(not(feature = "native"))]
    {
        "Native bindings not compiled (feature 'native' disabled)"
    }
}
