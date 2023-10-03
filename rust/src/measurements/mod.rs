//! Various measurement constructors.
//!
//! The different [`crate::core::Measurement`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Measurement` does.

#[cfg(feature = "contrib")]
mod gumbel_max;
#[cfg(feature = "contrib")]
pub use gumbel_max::*;

#[cfg(feature = "contrib")]
mod exponential_max;
#[cfg(feature = "contrib")]
pub use exponential_max::*;

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

#[cfg(feature = "contrib")]
mod private_quantile;
#[cfg(feature = "contrib")]
pub use private_quantile::*;

#[cfg(feature = "contrib")]
mod noise;
#[cfg(feature = "contrib")]
pub use noise::*;

#[cfg(feature = "contrib")]
mod noise_threshold;
#[cfg(feature = "contrib")]
pub use noise_threshold::*;

#[cfg(feature = "contrib")]
mod randomized_response;
#[cfg(feature = "contrib")]
pub use randomized_response::*;

#[cfg(feature = "contrib")]
mod randomized_response_bitvec;
#[cfg(feature = "contrib")]
pub use randomized_response_bitvec::*;

#[cfg(feature = "contrib")]
mod canonical_noise;
#[cfg(feature = "contrib")]
pub use canonical_noise::*;

#[cfg(all(feature = "floating-point", feature = "contrib"))]
mod alp;
#[cfg(all(feature = "floating-point", feature = "contrib"))]
pub use alp::*;
