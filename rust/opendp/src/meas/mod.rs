//! Various implementations of Measurement.
//!
//! The different [`Measurement`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Measurement` does.

#[cfg(feature="floating-point")]
pub mod laplace;
#[cfg(feature="floating-point")]
pub mod gaussian;
pub mod geometric;
#[cfg(feature="floating-point")]
pub mod stability;

#[cfg(feature="floating-point")]
pub use crate::meas::laplace::*;
#[cfg(feature="floating-point")]
pub use crate::meas::gaussian::*;
pub use crate::meas::geometric::*;
#[cfg(feature="floating-point")]
pub use crate::meas::stability::*;
