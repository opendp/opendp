//! Various implementations of Metric/Measure (and associated Distance).

use std::marker::PhantomData;

use crate::core::{DatasetMetric, Measure, Metric, SensitivityMetric};

/// Measures
#[derive(Clone)]
pub struct MaxDivergence<Q>(PhantomData<Q>);
impl<Q: Clone> Measure for MaxDivergence<Q> {
    type Distance = Q;
    fn new() -> Self { Self::new() }
}
impl<Q> MaxDivergence<Q> {
    pub fn new() -> Self { MaxDivergence(PhantomData) }
}

#[derive(Clone)]
pub struct SmoothedMaxDivergence<Q>(PhantomData<Q>);
impl<Q: Clone> Measure for SmoothedMaxDivergence<Q> {
    type Distance = (Q, Q);
    fn new() -> Self { Self::new() }
}
impl<Q> SmoothedMaxDivergence<Q> {
    pub fn new() -> Self { SmoothedMaxDivergence(PhantomData) }
}

/// Metrics
#[derive(Clone)]
pub struct SymmetricDistance;
impl Metric for SymmetricDistance {
    type Distance = u32;
    fn new() -> Self { Self::new() }
}
impl SymmetricDistance {
    pub fn new() -> Self { SymmetricDistance }
}

#[derive(Clone)]
pub struct HammingDistance;

impl Metric for HammingDistance {
    type Distance = u32;
    fn new() -> Self { Self::new() }
}

impl DatasetMetric for HammingDistance {}

impl HammingDistance {
    pub fn new() -> Self { HammingDistance }
}

impl DatasetMetric for SymmetricDistance {}

pub struct L1Sensitivity<Q> {
    _marker: PhantomData<Q>
}

impl<Q> SensitivityMetric for L1Sensitivity<Q> {}

impl<Q> L1Sensitivity<Q> {
    pub fn new() -> Self {
        L1Sensitivity { _marker: PhantomData }
    }
}

impl<Q> Clone for L1Sensitivity<Q> {
    fn clone(&self) -> Self { Self::new() }
}

impl<Q> SensitivityMetric for L2Sensitivity<Q> {}

impl<Q> Metric for L1Sensitivity<Q> {
    type Distance = Q;
    fn new() -> Self { Self::new() }
}

pub struct L2Sensitivity<Q> {
    _marker: PhantomData<Q>
}
impl<Q> L2Sensitivity<Q> {
    pub fn new() -> Self {
        L2Sensitivity { _marker: PhantomData }
    }
}
impl <Q> Clone for L2Sensitivity<Q> {
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<Q> Metric for L2Sensitivity<Q> {
    type Distance = Q;
    fn new() -> Self { Self::new() }
}
