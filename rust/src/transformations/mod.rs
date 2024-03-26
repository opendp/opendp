//! Various transformation constructors.
//!
//! The different [`crate::core::Transformation`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Transformation` does.

#[cfg(all(feature = "contrib", feature = "polars"))]
mod make_stable_lazyframe;
#[cfg(all(feature = "contrib", feature = "polars"))]
pub use make_stable_lazyframe::*;

#[cfg(all(feature = "contrib", feature = "polars"))]
mod make_stable_expr;
#[cfg(all(feature = "contrib", feature = "polars"))]
pub use make_stable_expr::*;

#[cfg(feature = "contrib")]
mod covariance;
#[cfg(feature = "contrib")]
pub use crate::transformations::covariance::*;

#[cfg(feature = "contrib")]
mod b_ary_tree;
#[cfg(feature = "contrib")]
pub use crate::transformations::b_ary_tree::*;

#[cfg(feature = "contrib")]
mod dataframe;
#[cfg(feature = "contrib")]
pub use crate::transformations::dataframe::*;

#[cfg(feature = "contrib")]
pub mod quantile_score_candidates;
#[cfg(feature = "contrib")]
pub use crate::transformations::quantile_score_candidates::*;

#[cfg(feature = "contrib")]
mod manipulation;
#[cfg(feature = "contrib")]
pub use crate::transformations::manipulation::*;

#[cfg(feature = "contrib")]
mod sum;
#[cfg(feature = "contrib")]
pub use crate::transformations::sum::*;

#[cfg(feature = "contrib")]
mod sum_of_squared_deviations;
#[cfg(feature = "contrib")]
pub use crate::transformations::sum_of_squared_deviations::*;

#[cfg(feature = "contrib")]
mod count;
#[cfg(feature = "contrib")]
pub use crate::transformations::count::*;

#[cfg(feature = "contrib")]
mod count_cdf;
#[cfg(feature = "contrib")]
pub use crate::transformations::count_cdf::*;

#[cfg(feature = "contrib")]
mod mean;
#[cfg(feature = "contrib")]
pub use crate::transformations::mean::*;

#[cfg(feature = "contrib")]
mod variance;
#[cfg(feature = "contrib")]
pub use crate::transformations::variance::*;

#[cfg(feature = "contrib")]
mod impute;
#[cfg(feature = "contrib")]
pub use crate::transformations::impute::*;

#[cfg(feature = "contrib")]
mod index;
#[cfg(feature = "contrib")]
pub use crate::transformations::index::*;

#[cfg(feature = "contrib")]
mod lipschitz_mul;
#[cfg(feature = "contrib")]
pub use crate::transformations::lipschitz_mul::*;

#[cfg(feature = "ffi")]
mod make_user_transformation;
#[cfg(feature = "ffi")]
pub use crate::transformations::make_user_transformation::*;

#[cfg(feature = "contrib")]
mod clamp;
#[cfg(feature = "contrib")]
pub use crate::transformations::clamp::*;

#[cfg(feature = "contrib")]
mod cast;
#[cfg(feature = "contrib")]
pub use crate::transformations::cast::*;

#[cfg(feature = "contrib")]
mod cast_metric;
#[cfg(feature = "contrib")]
pub use crate::transformations::cast_metric::*;

#[cfg(feature = "contrib")]
mod resize;
#[cfg(feature = "contrib")]
pub use crate::transformations::resize::*;
