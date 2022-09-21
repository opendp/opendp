//! Various implementations of Transformations.
//!
//! The different [`crate::core::Transformation`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Transformation` does.

#[cfg(feature="contrib")]
pub mod covariance;
#[cfg(feature="contrib")]
pub use crate::transformations::covariance::*;

#[cfg(feature="contrib")]
pub mod b_ary_tree;
#[cfg(feature="contrib")]
pub use crate::transformations::b_ary_tree::*;

#[cfg(feature="contrib")]
pub mod dataframe;
#[cfg(feature="contrib")]
pub use crate::transformations::dataframe::*;

#[cfg(feature="contrib")]
pub(crate) mod postprocess;
#[cfg(feature="contrib")]
pub(crate) use postprocess::*;

#[cfg(feature="contrib")]
pub mod manipulation;
#[cfg(feature="contrib")]
pub use crate::transformations::manipulation::*;

#[cfg(feature="contrib")]
pub mod sum;
#[cfg(feature="contrib")]
pub use crate::transformations::sum::*;

#[cfg(feature="contrib")]
pub mod sum_of_squared_deviations;
#[cfg(feature="contrib")]
pub use crate::transformations::sum_of_squared_deviations::*;

#[cfg(feature="contrib")]
pub mod count;
#[cfg(feature="contrib")]
pub use crate::transformations::count::*;

#[cfg(feature="contrib")]
pub mod count_cdf;
#[cfg(feature="contrib")]
pub use crate::transformations::count_cdf::*;

#[cfg(feature="contrib")]
pub mod mean;
#[cfg(feature="contrib")]
pub use crate::transformations::mean::*;

#[cfg(feature="contrib")]
pub mod variance;
#[cfg(feature="contrib")]
pub use crate::transformations::variance::*;

#[cfg(feature="contrib")]
pub mod impute;
#[cfg(feature="contrib")]
pub use crate::transformations::impute::*;

#[cfg(feature="contrib")]
pub mod index;
#[cfg(feature="contrib")]
pub use crate::transformations::index::*;

#[cfg(feature="contrib")]
pub mod lipschitz_mul;
#[cfg(feature="contrib")]
pub use crate::transformations::lipschitz_mul::*;

#[cfg(feature="contrib")]
pub mod clamp;
#[cfg(feature="contrib")]
pub use crate::transformations::clamp::*;

#[cfg(feature="contrib")]
pub mod cast;
#[cfg(feature="contrib")]
pub use crate::transformations::cast::*;

#[cfg(feature="contrib")]
pub mod cast_metric;
#[cfg(feature="contrib")]
pub use crate::transformations::cast_metric::*;

#[cfg(feature="contrib")]
pub mod resize;
#[cfg(feature="contrib")]
pub use crate::transformations::resize::*;

