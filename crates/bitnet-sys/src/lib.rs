//! Low-level FFI bindings to bitnet.cpp
//!
//! This crate provides raw bindings to bitnet.cpp, Microsoft's optimized
//! inference runtime for BitNet 1.58-bit models.
//!
//! ## Build Requirements
//!
//! - CMake 3.14+
//! - Clang 18+ (recommended) or GCC 11+
//! - For CUDA: CUDA Toolkit 11.0+
//!
//! ## Features
//!
//! - `cuda` - Enable CUDA GPU acceleration
//!
//! ## Usage
//!
//! This crate is typically used through the higher-level `neuro-inference` crate.
//! Direct usage requires careful handling of raw pointers and FFI safety.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]

// Only include bindings if build succeeded
#[cfg(not(bitnet_sys_failed))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Fallback types when build fails
#[cfg(bitnet_sys_failed)]
pub mod fallback {
    //! Placeholder types when native bindings are unavailable
    
    use std::ffi::c_void;
    
    /// Placeholder for llama_model
    pub type llama_model = c_void;
    /// Placeholder for llama_context  
    pub type llama_context = c_void;
    /// Placeholder for llama_sampler
    pub type llama_sampler = c_void;
    /// Placeholder for llama_batch
    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone)]
    pub struct llama_batch {
        pub n_tokens: i32,
    }
    /// Placeholder token type
    pub type llama_token = i32;
    /// Placeholder position type
    pub type llama_pos = i32;
    /// Placeholder sequence ID type
    pub type llama_seq_id = i32;
}

#[cfg(bitnet_sys_failed)]
pub use fallback::*;

/// Check if native bindings are available
pub const fn is_available() -> bool {
    cfg!(not(bitnet_sys_failed))
}

/// Get the backend type string
pub const fn backend_type() -> &'static str {
    if cfg!(bitnet_tl1) {
        "BitNet TL1 (ARM NEON)"
    } else if cfg!(bitnet_tl2) {
        "BitNet TL2 (x86 AVX)"
    } else if cfg!(bitnet_cuda) {
        "BitNet CUDA"
    } else if cfg!(bitnet_sys_failed) {
        "Unavailable (build failed)"
    } else {
        "BitNet Generic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type() {
        let backend = backend_type();
        assert!(!backend.is_empty());
        println!("Backend: {}", backend);
    }

    #[test]
    fn test_is_available() {
        let available = is_available();
        println!("Native bindings available: {}", available);
    }
}
