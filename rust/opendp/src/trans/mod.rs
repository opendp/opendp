//! Various implementations of Transformations.
//!
//! The different [`Transformation`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Transformation` does.

#[cfg(all(feature="contrib"))]
pub mod dataframe;
#[cfg(all(feature="contrib"))]
pub use crate::trans::dataframe::*;

#[cfg(all(feature="contrib"))]
pub mod manipulation;
#[cfg(all(feature="contrib"))]
pub use crate::trans::manipulation::*;

#[cfg(all(feature="contrib"))]
pub mod sum;
#[cfg(all(feature="contrib"))]
pub use crate::trans::sum::*;

#[cfg(all(feature="contrib"))]
pub mod count;
#[cfg(all(feature="contrib"))]
pub use crate::trans::count::*;

#[cfg(all(feature="contrib"))]
pub mod mean;
#[cfg(all(feature="contrib"))]
pub use crate::trans::mean::*;

#[cfg(all(feature="contrib"))]
pub mod variance;
#[cfg(all(feature="contrib"))]
pub use crate::trans::variance::*;

#[cfg(all(feature="contrib"))]
pub mod impute;
#[cfg(all(feature="contrib"))]
pub use crate::trans::impute::*;

#[cfg(all(feature="contrib"))]
pub mod clamp;
#[cfg(all(feature="contrib"))]
pub use crate::trans::clamp::*;

#[cfg(all(feature="contrib"))]
pub mod cast;
#[cfg(all(feature="contrib"))]
pub use crate::trans::cast::*;

#[cfg(all(feature="contrib"))]
pub mod resize;
#[cfg(all(feature="contrib"))]
pub use crate::trans::resize::*;

