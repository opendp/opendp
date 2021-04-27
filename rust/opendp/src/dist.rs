//! Various implementations of Metric/Measure (and associated Distance).

use std::marker::PhantomData;

use crate::core::{DatasetMetric, Measure, Metric, SensitivityMetric};

/// Measures
#[derive(Clone)]
pub struct MaxDivergence<Q>(PhantomData<Q>);
impl<Q> Default for  MaxDivergence<Q> {
    fn default() -> Self { MaxDivergence(PhantomData) }
}

impl<Q: Clone> Measure for MaxDivergence<Q> {
    type Distance = Q;
}

#[derive(Clone)]
pub struct SmoothedMaxDivergence<Q>(PhantomData<Q>);

impl<Q> Default for SmoothedMaxDivergence<Q> {
    fn default() -> Self { SmoothedMaxDivergence(PhantomData) }
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

impl Metric for SymmetricDistance {
    type Distance = u32;
}

impl DatasetMetric for SymmetricDistance {}

#[derive(Clone)]
pub struct HammingDistance;

impl Default for HammingDistance {
    fn default() -> Self { HammingDistance }
}

impl Metric for HammingDistance {
    type Distance = u32;
}

impl DatasetMetric for HammingDistance {}

pub struct L1Sensitivity<Q>(PhantomData<Q>);

impl<Q> Default for L1Sensitivity<Q> {
    fn default() -> Self { L1Sensitivity(PhantomData) }
}

impl<Q> Clone for L1Sensitivity<Q> {
    fn clone(&self) -> Self { Self::default() }
}

impl<Q> Metric for L1Sensitivity<Q> {
    type Distance = Q;
}

impl<Q> SensitivityMetric for L1Sensitivity<Q> {}

pub struct L2Sensitivity<Q>(PhantomData<Q>);

impl<Q> Default for L2Sensitivity<Q> {
    fn default() -> Self { L2Sensitivity(PhantomData) }
}

impl<Q> Clone for L2Sensitivity<Q> {
    fn clone(&self) -> Self { Self::default() }
}

impl<Q> Metric for L2Sensitivity<Q> {
    type Distance = Q;
}

impl<Q> SensitivityMetric for L2Sensitivity<Q> {}
