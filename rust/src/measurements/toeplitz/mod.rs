//! Toeplitz mechanism for differentially private continual release
//! 
//! This module provides two APIs for the Toeplitz mechanism:
//! 
//! ## One-shot API
//! Functions that return standard OpenDP measurements for single computations:
//! - `make_toeplitz` - Base function with monotonicity flag
//! - `make_baseline_toeplitz` - Without monotonicity enforcement  
//! - `make_monotonic_toeplitz` - With isotonic regression for monotonicity
//! 
//! ## Continual Release API
//! Stateful structs implementing the `ContinualRelease` trait for incremental releases:
//! - `BaselineContinualToeplitz` - Without monotonicity enforcement
//! - `MonotonicContinualToeplitz` - With isotonic regression for monotonicity

// Internal modules
pub(crate) mod core;
pub(crate) mod isotonic;
pub(crate) mod noise_generation;
pub(crate) mod type_conversion;

// Public API exports for one-shot measurements
#[cfg(feature = "contrib-continual")]
pub use one_shot::{
    make_toeplitz,
    make_baseline_toeplitz,
    make_monotonic_toeplitz,
};

// Public API exports for continual release
#[cfg(feature = "contrib-continual")]
pub use continual::{
    BaselineContinualToeplitz,
    MonotonicContinualToeplitz,
    ContinualRelease,
};

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
