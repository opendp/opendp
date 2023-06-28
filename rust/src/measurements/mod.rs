//! Various measurement constructors.
//!
//! The different [`crate::core::Measurement`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Measurement` does.
#[cfg(all(feature = "contrib", feature = "use-mpfr"))]
mod gaussian;
#[cfg(all(feature = "contrib", feature = "use-mpfr"))]
pub use gaussian::*;

#[cfg(feature = "contrib")]
mod laplace;
#[cfg(feature = "contrib")]
pub use laplace::*;

#[cfg(all(feature = "floating-point", feature = "contrib"))]
mod laplace_threshold;
#[cfg(all(feature = "floating-point", feature = "contrib"))]
pub use laplace_threshold::*;

#[cfg(feature = "contrib")]
mod randomized_response;
#[cfg(feature = "contrib")]
pub use randomized_response::*;

#[cfg(all(feature = "use-mpfr", feature = "floating-point", feature = "contrib"))]
mod alp;
#[cfg(all(feature = "use-mpfr", feature = "floating-point", feature = "contrib"))]
pub use alp::*;
