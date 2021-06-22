//! Various implementations of Metric/Measure (and associated Distance).

use std::marker::PhantomData;

use crate::core::{DatasetMetric, Measure, Metric, SensitivityMetric};

/// Measures
#[derive(Clone)]
pub struct MaxDivergence<Q>(PhantomData<Q>);
impl<Q> Default for MaxDivergence<Q> {
    fn default() -> Self { MaxDivergence(PhantomData) }
}

impl<Q> PartialEq for MaxDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool { true }
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

impl Metric for SymmetricDistance {
    type Distance = u32;
}

impl DatasetMetric for SymmetricDistance {}

#[derive(Clone)]
pub struct HammingDistance;

impl Default for HammingDistance {
    fn default() -> Self { HammingDistance }
}

impl PartialEq for HammingDistance {
    fn eq(&self, _other: &Self) -> bool { true }
}

impl Metric for HammingDistance {
    type Distance = u32;
}

impl DatasetMetric for HammingDistance {}

// Sensitivity in P-space
pub struct LPSensitivity<Q, const P: usize>(PhantomData<Q>);
impl<Q, const P: usize> Default for LPSensitivity<Q, P> {
    fn default() -> Self { LPSensitivity(PhantomData) }
}

impl<Q, const P: usize> Clone for LPSensitivity<Q, P> {
    fn clone(&self) -> Self { Self::default() }
}
impl<Q, const P: usize> PartialEq for LPSensitivity<Q, P> {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl<Q, const P: usize> Metric for LPSensitivity<Q, P> {
    type Distance = Q;
}
impl<Q, const P: usize> SensitivityMetric for LPSensitivity<Q, P> {}

pub type L1Sensitivity<Q> = LPSensitivity<Q, 1>;
pub type L2Sensitivity<Q> = LPSensitivity<Q, 2>;
