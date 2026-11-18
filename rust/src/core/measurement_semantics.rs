//! Semantic interfaces for certified measurement construction.
//!
//! The native traits are marker bounds: mathematical specifications exist
//! only in the Verus build. Implementations must be verified there.
//! The certified constructor's safety comes from checked Verus call sites,
//! not from a runtime Rust trait bound.

#[cfg(not(verus_only))]
use super::{Domain, Metric, Measure};
#[cfg(verus_only)]
use crate::{Domain, Metric, Measure};
#[cfg(verus_only)]
use vstd::prelude::*;

#[cfg(not(verus_only))]
pub trait CertifiedDomain: Domain {}
#[cfg(not(verus_only))]
impl<D: Domain> CertifiedDomain for D {}

#[cfg(not(verus_only))]
pub trait CertifiedMetric<X>: Metric {}
#[cfg(not(verus_only))]
impl<X, M: Metric> CertifiedMetric<X> for M {}

#[cfg(not(verus_only))]
pub trait CertifiedMeasure<Z>: Measure {}
#[cfg(not(verus_only))]
impl<Z, M: Measure> CertifiedMeasure<Z> for M {}

#[cfg(verus_only)]
verus! {
/// Mathematical membership, independent of the implementation of Domain::member.
/// Concrete implementations must be reviewed as specifications.
pub trait CertifiedDomain: Domain {
    spec fn contains(&self, value: &Self::Carrier) -> bool;
}

/// Mathematical closeness and the supported distance values of a metric.
pub trait CertifiedMetric<X>: Metric {
    spec fn valid_distance(&self, distance: &Self::Distance) -> bool;
    spec fn close(&self, left: &X, right: &X, distance: &Self::Distance) -> bool;
}

/// Mathematical measure semantics, expressed using expectation functionals
/// rather than an explicitly represented probability distribution.
pub trait CertifiedMeasure<Z>: Measure {
    spec fn valid_cost(&self, cost: &Self::Distance) -> bool;
    spec fn dominates(
        &self,
        left: spec_fn(spec_fn(Z) -> real) -> real,
        right: spec_fn(spec_fn(Z) -> real) -> real,
        cost: &Self::Distance,
    ) -> bool;
}
}
