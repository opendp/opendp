//! Various implementations of Metric/Measure (and associated Distance).

use std::marker::PhantomData;

use crate::core::{DatasetMetric, Measure, Metric, SensitivityMetric};
use std::fmt::{Debug, Formatter};

// default type for distances between datasets
pub type IntDistance = u32;

/// Measures
#[derive(Clone)]
pub struct MaxDivergence<Q>(PhantomData<Q>);
impl<Q> Default for MaxDivergence<Q> {
    fn default() -> Self { MaxDivergence(PhantomData) }
}

impl<Q> PartialEq for MaxDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool { true }
}

impl<Q> Debug for MaxDivergence<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "MaxDivergence()")
    }
}

impl<Q: Clone> Measure for MaxDivergence<Q> {
    type Distance = Q;
}

#[derive(Clone)]
pub struct SmoothedMaxDivergence<Q>(PhantomData<Q>);

impl<Q> Default for SmoothedMaxDivergence<Q> {
    fn default() -> Self { SmoothedMaxDivergence(PhantomData) }
}

impl<Q> PartialEq for SmoothedMaxDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl<Q> Debug for SmoothedMaxDivergence<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "SmoothedMaxDivergence()")
    }
}
impl<Q: Clone> Measure for SmoothedMaxDivergence<Q> {
    type Distance = (Q, Q);
}

/// Metrics
#[derive(Clone)]
pub struct SymmetricDistance;

impl Default for SymmetricDistance {
    fn default() -> Self { SymmetricDistance }
}

impl PartialEq for SymmetricDistance {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl Debug for SymmetricDistance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "SymmetricDistance()")
    }
}
impl Metric for SymmetricDistance {
    type Distance = IntDistance;
}

impl DatasetMetric for SymmetricDistance {}

#[derive(Clone)]
pub struct SubstituteDistance;

impl Default for SubstituteDistance {
    fn default() -> Self { SubstituteDistance }
}

impl PartialEq for SubstituteDistance {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl Debug for SubstituteDistance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "SubstituteDistance()")
    }
}
impl Metric for SubstituteDistance {
    type Distance = IntDistance;
}

impl DatasetMetric for SubstituteDistance {}

// Sensitivity in P-space
pub struct LpDistance<Q, const P: usize>(PhantomData<Q>);
impl<Q, const P: usize> Default for LpDistance<Q, P> {
    fn default() -> Self { LpDistance(PhantomData) }
}

impl<Q, const P: usize> Clone for LpDistance<Q, P> {
    fn clone(&self) -> Self { Self::default() }
}
impl<Q, const P: usize> PartialEq for LpDistance<Q, P> {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl<Q, const P: usize> Debug for LpDistance<Q, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "L{}Distance()", P)
    }
}
impl<Q, const P: usize> Metric for LpDistance<Q, P> {
    type Distance = Q;
}
impl<Q, const P: usize> SensitivityMetric for LpDistance<Q, P> {}

pub type L1Distance<Q> = LpDistance<Q, 1>;
pub type L2Distance<Q> = LpDistance<Q, 2>;


pub struct AbsoluteDistance<Q>(PhantomData<Q>);
impl<Q> Default for AbsoluteDistance<Q> {
    fn default() -> Self { AbsoluteDistance(PhantomData) }
}

impl<Q> Clone for AbsoluteDistance<Q> {
    fn clone(&self) -> Self { Self::default() }
}
impl<Q> PartialEq for AbsoluteDistance<Q> {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl<Q> Debug for AbsoluteDistance<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "AbsoluteDistance()")
    }
}
impl<Q> Metric for AbsoluteDistance<Q> {
    type Distance = Q;
}
impl<Q> SensitivityMetric for AbsoluteDistance<Q> {}
