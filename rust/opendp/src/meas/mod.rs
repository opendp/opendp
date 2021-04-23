//! Various implementations of Measurement.
//!
//! The different [`Measurement`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Measurement` does.

use crate::core::{Domain, Measure, Metric};
use crate::error::Fallible;

pub mod laplace;
pub mod gaussian;
pub mod geometric;
pub mod stability;
pub mod snapping;

// Trait for all constructors, can have different implementations depending on concrete types of Domains and/or Metrics
pub trait MakeMeasurement<DI: Domain, DO: Domain, MI: Metric, MO: Measure> {
    fn make() -> Fallible<crate::core::Measurement<DI, DO, MI, MO>> {
        Self::make0()
    }
    fn make0() -> Fallible<crate::core::Measurement<DI, DO, MI, MO>>;
}

pub trait MakeMeasurement1<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1> {
    fn make(param1: P1) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>> {
        Self::make1(param1)
    }
    fn make1(param1: P1) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>>;
}

pub trait MakeMeasurement2<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2> {
    fn make(param1: P1, param2: P2) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>> {
        Self::make2(param1, param2)
    }
    fn make2(param1: P1, param2: P2) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>>;
}

pub trait MakeMeasurement3<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2, P3> {
    fn make(param1: P1, param2: P2, param3: P3) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>> {
        Self::make3(param1, param2, param3)
    }
    fn make3(param1: P1, param2: P2, param3: P3) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>>;
}

pub trait MakeMeasurement4<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2, P3, P4> {
    fn make(param1: P1, param2: P2, param3: P3, param4: P4) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>> {
        Self::make4(param1, param2, param3, param4)
    }
    fn make4(param1: P1, param2: P2, param3: P3, param4: P4) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>>;
}

