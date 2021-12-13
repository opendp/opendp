//! Various implementations of Measurement.
//!
//! The different [`Measurement`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Measurement` does.

#[cfg(all(feature="floating-point", feature="contrib"))]
pub mod laplace;
#[cfg(all(feature="floating-point", feature="contrib"))]
pub use crate::meas::laplace::*;

#[cfg(all(feature="floating-point", feature="contrib"))]
pub mod gaussian;
#[cfg(all(feature="floating-point", feature="contrib"))]
pub use crate::meas::gaussian::*;

#[cfg(feature="contrib")]
pub mod discrete_gaussian;
#[cfg(feature="contrib")]
pub use crate::meas::discrete_gaussian::*;

#[cfg(feature="contrib")]
pub mod discrete_laplace;
#[cfg(feature="contrib")]
pub use crate::meas::discrete_laplace::*;


#[cfg(all(feature="floating-point", feature="contrib"))]
pub mod ptr;
#[cfg(all(feature="floating-point", feature="contrib"))]
pub use crate::meas::ptr::*;

#[cfg(feature="contrib")]
pub mod randomized_response;
#[cfg(feature="contrib")]
pub use crate::meas::randomized_response::*;

#[cfg(all(feature="floating-point", feature="contrib"))]
pub mod alp;
#[cfg(all(feature="floating-point", feature="contrib"))]
pub use crate::meas::alp::*;
