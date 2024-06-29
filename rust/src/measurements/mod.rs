//! Various measurement constructors.
//!
//! The different [`crate::core::Measurement`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Measurement` does.

#[cfg(all(feature = "contrib"))]
mod gaussian;
#[cfg(all(feature = "contrib"))]
pub use gaussian::*;

#[cfg(feature = "contrib")]
mod geometric;
#[cfg(feature = "contrib")]
pub use geometric::*;

#[cfg(feature = "contrib")]
mod gumbel_max;
#[cfg(feature = "contrib")]
pub use gumbel_max::*;

#[cfg(feature = "contrib")]
mod laplace;
#[cfg(feature = "contrib")]
pub use laplace::*;

#[cfg(all(feature = "contrib", feature = "polars"))]
mod make_private_expr;
#[cfg(all(feature = "contrib", feature = "polars"))]
pub use make_private_expr::*;

#[cfg(all(feature = "contrib", feature = "polars"))]
mod make_private_lazyframe;
#[cfg(all(feature = "contrib", feature = "polars"))]
pub use make_private_lazyframe::*;

#[cfg(all(feature = "honest-but-curious", feature = "ffi"))]
mod make_user_measurement;
#[cfg(all(feature = "honest-but-curious", feature = "ffi"))]
pub use crate::measurements::make_user_measurement::*;

#[cfg(all(feature = "floating-point", feature = "contrib"))]
mod laplace_threshold;
#[cfg(all(feature = "floating-point", feature = "contrib"))]
pub use laplace_threshold::*;

#[cfg(feature = "contrib")]
mod randomized_response;
#[cfg(feature = "contrib")]
pub use randomized_response::*;

#[cfg(feature = "contrib")]
mod randomized_response_bitvec;
#[cfg(feature = "contrib")]
pub use randomized_response_bitvec::*;

#[cfg(all(feature = "floating-point", feature = "contrib"))]
mod alp;
#[cfg(all(feature = "floating-point", feature = "contrib"))]
pub use alp::*;
