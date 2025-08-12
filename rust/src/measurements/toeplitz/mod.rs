//! Toeplitz mechanism for differentially private continual release
//! 
//! This module provides two APIs for the Toeplitz mechanism:
//! - One-shot API: `make_toeplitz` for single computations
//! - Continual API: `BaselineContinualToeplitz` and `MonotonicContinualToeplitz` for stateful, incremental releases

// Internal modules
mod utils;

// Public API exports
/// One-shot Toeplitz measurement
#[cfg(feature = "contrib-continual")]
pub use one_shot::make_toeplitz;
/// Continual release API (when feature enabled)
#[cfg(feature = "contrib-continual")]
pub use continual::{BaselineContinualToeplitz, MonotonicContinualToeplitz, ContinualRelease};

// API modules
#[cfg(feature = "contrib-continual")]
mod one_shot;
#[cfg(feature = "contrib-continual")]
mod continual;

// FFI module (when feature enabled)
#[cfg(feature = "ffi")]
mod ffi;

// Test module
#[cfg(test)]
mod test;
