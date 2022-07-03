//! Various implementations of Metric/Measure (and associated Distance).

use std::{marker::PhantomData};

use crate::core::{DatasetMetric, Metric, SensitivityMetric};
use std::fmt::{Debug, Formatter};

use super::type_name;

// default type for distances between datasets
pub type IntDistance = u32;


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
pub struct InsertDeleteDistance;

impl Default for InsertDeleteDistance {
    fn default() -> Self { InsertDeleteDistance }
}

impl PartialEq for InsertDeleteDistance {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl Debug for InsertDeleteDistance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "InsertDeleteDistance()")
    }
}
impl Metric for InsertDeleteDistance {
    type Distance = IntDistance;
}

impl DatasetMetric for InsertDeleteDistance {}

#[derive(Clone)]
pub struct ChangeOneDistance;

impl Default for ChangeOneDistance {
    fn default() -> Self { ChangeOneDistance }
}

impl PartialEq for ChangeOneDistance {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl Debug for ChangeOneDistance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ChangeOneDistance()")
    }
}
impl Metric for ChangeOneDistance {
    type Distance = IntDistance;
}

impl DatasetMetric for ChangeOneDistance {}

#[derive(Clone)]
pub struct HammingDistance;

impl Default for HammingDistance {
    fn default() -> Self { HammingDistance }
}

impl PartialEq for HammingDistance {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl Debug for HammingDistance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "HammingDistance()")
    }
}
impl Metric for HammingDistance {
    type Distance = IntDistance;
}

impl DatasetMetric for HammingDistance {}

// Sensitivity in P-space
pub struct LpDistance<const P: usize, Q>(PhantomData<Q>);
impl<const P: usize, Q> Default for LpDistance<P, Q> {
    fn default() -> Self { LpDistance(PhantomData) }
}

impl<const P: usize, Q> Clone for LpDistance<P, Q> {
    fn clone(&self) -> Self { Self::default() }
}
impl<const P: usize, Q> PartialEq for LpDistance<P, Q> {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl<const P: usize, Q> Debug for LpDistance<P, Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "L{}Distance({})", P, type_name!(Q))
    }
}
impl<const P: usize, Q> Metric for LpDistance<P, Q> {
    type Distance = Q;
}
impl<const P: usize, Q> SensitivityMetric for LpDistance<P, Q> {}

pub type L1Distance<Q> = LpDistance<1, Q>;
pub type L2Distance<Q> = LpDistance<2, Q>;

/// Represents a metric where d(a, b) = |a - b|
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
        write!(f, "AbsoluteDistance({})", type_name!(Q))
    }
}
impl<Q> Metric for AbsoluteDistance<Q> {
    type Distance = Q;
}
impl<Q> SensitivityMetric for AbsoluteDistance<Q> {}
