//! Various implementations of Metric/Measure (and associated Distance).

use std::marker::PhantomData;

use crate::core::{DatasetMetric, Measure, Metric, SensitivityMetric};

/// Measures
#[derive(Clone)]
pub struct MaxDivergence;
impl Measure for MaxDivergence {
    type Distance = f64;
}
impl MaxDivergence {
    pub fn new() -> Self { MaxDivergence }
}

#[derive(Clone)]
pub struct SmoothedMaxDivergence;
impl Measure for SmoothedMaxDivergence {
    type Distance = (f64, f64);
}
impl SmoothedMaxDivergence {
    pub fn new() -> Self { SmoothedMaxDivergence }
}

/// Metrics
#[derive(Clone)]
pub struct SymmetricDistance;
impl Metric for SymmetricDistance {
    type Distance = u32;
}
impl SymmetricDistance {
    pub fn new() -> Self { SymmetricDistance }
}

#[derive(Clone)]
pub struct HammingDistance;

impl Metric for HammingDistance {
    type Distance = u32;
}

impl DatasetMetric for HammingDistance {
    fn new() -> Self { HammingDistance }
}

impl HammingDistance {
    pub fn new() -> Self { HammingDistance }
}

impl DatasetMetric for SymmetricDistance {
    fn new() -> Self { SymmetricDistance }
}

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
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<Q> SensitivityMetric for L2Sensitivity<Q> {}

impl<Q> Metric for L1Sensitivity<Q> {
    type Distance = Q;
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
}
