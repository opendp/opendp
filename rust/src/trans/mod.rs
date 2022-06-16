//! Various implementations of Transformations.
//!
//! The different [`Transformation`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Transformation` does.

#[cfg(feature="contrib")]
pub mod dataframe;
#[cfg(feature="contrib")]
pub use crate::trans::dataframe::*;

#[cfg(feature="contrib")]
pub mod manipulation;
#[cfg(feature="contrib")]
pub use crate::trans::manipulation::*;

#[cfg(feature="contrib")]
pub mod sum;
#[cfg(feature="contrib")]
pub use crate::trans::sum::*;

#[cfg(feature="contrib")]
pub mod sum_of_squared_deviations;
#[cfg(feature="contrib")]
pub use crate::trans::sum_of_squared_deviations::*;

#[cfg(feature="contrib")]
pub mod count;
#[cfg(feature="contrib")]
pub use crate::trans::count::*;

#[cfg(feature="contrib")]
pub mod mean;
#[cfg(feature="contrib")]
pub use crate::trans::mean::*;

#[cfg(feature="contrib")]
pub mod variance;
#[cfg(feature="contrib")]
pub use crate::trans::variance::*;

#[cfg(feature="contrib")]
pub mod impute;
#[cfg(feature="contrib")]
pub use crate::trans::impute::*;

#[cfg(feature="contrib")]
pub mod index;
#[cfg(feature="contrib")]
pub use crate::trans::index::*;

#[cfg(feature="contrib")]
pub mod lipschitz_cast;
#[cfg(feature="contrib")]
pub use crate::trans::lipschitz_cast::*;

#[cfg(feature="contrib")]
pub mod lipschitz_mul;
#[cfg(feature="contrib")]
pub use crate::trans::lipschitz_mul::*;

#[cfg(feature="contrib")]
pub mod clamp;
#[cfg(feature="contrib")]
pub use crate::trans::clamp::*;

#[cfg(feature="contrib")]
pub mod cast;
#[cfg(feature="contrib")]
pub use crate::trans::cast::*;

#[cfg(feature="contrib")]
pub mod cast_metric;
#[cfg(feature="contrib")]
pub use crate::trans::cast_metric::*;

#[cfg(feature="contrib")]
pub mod resize;
#[cfg(feature="contrib")]
pub use crate::trans::resize::*;

