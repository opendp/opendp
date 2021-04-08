//! Various implementations of Transformations.
//!
//! The different [`Transformation`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Transformation` does.

use crate::core::{Domain, Metric};
use crate::error::Fallible;
pub use crate::trans::dataframe::*;

pub mod dataframe;
pub mod manipulation;
pub mod sum;
pub mod count;

// Trait for all constructors, can have different implementations depending on concrete types of Domains and/or Metrics
pub trait MakeTransformation0<DI: Domain, DO: Domain, MI: Metric, MO: Metric> {
    fn make() -> Fallible<crate::core::Transformation<DI, DO, MI, MO>> {
        Self::make0()
    }
    fn make0() -> Fallible<crate::core::Transformation<DI, DO, MI, MO>>;
}

pub trait MakeTransformation1<DI: Domain, DO: Domain, MI: Metric, MO: Metric, P1> {
    fn make(param1: P1) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>> {
        Self::make1(param1)
    }
    fn make1(param1: P1) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>>;
}

pub trait MakeTransformation2<DI: Domain, DO: Domain, MI: Metric, MO: Metric, P1, P2> {
    fn make(param1: P1, param2: P2) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>> {
        Self::make2(param1, param2)
    }
    fn make2(param1: P1, param2: P2) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>>;
}

pub trait MakeTransformation3<DI: Domain, DO: Domain, MI: Metric, MO: Metric, P1, P2, P3> {
    fn make(param1: P1, param2: P2, param3: P3) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>> {
        Self::make3(param1, param2, param3)
    }
    fn make3(param1: P1, param2: P2, param3: P3) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>>;
}

pub trait MakeTransformation4<DI: Domain, DO: Domain, MI: Metric, MO: Metric, P1, P2, P3, P4> {
    fn make(param1: P1, param2: P2, param3: P3, param4: P4) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>> {
        Self::make4(param1, param2, param3, param4)
    }
    fn make4(param1: P1, param2: P2, param3: P3, param4: P4) -> Fallible<crate::core::Transformation<DI, DO, MI, MO>>;
}
