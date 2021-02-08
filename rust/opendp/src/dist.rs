//! Various implementations of Metric/Measure (and associated Distance).

use std::marker::PhantomData;

use crate::core::{Measure, Metric};

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
    type Distance = i32;
}
impl SymmetricDistance {
    pub fn new() -> Self { SymmetricDistance }
}

#[derive(Clone)]
pub struct HammingDistance;
impl Metric for HammingDistance {
    type Distance = i32;
}
impl HammingDistance {
    pub fn new() -> Self { HammingDistance }
}

pub struct L1Sensitivity<T> {
    _marker: PhantomData<T>
}
impl<T> L1Sensitivity<T> {
    pub fn new() -> Self {
        L1Sensitivity { _marker: PhantomData }
    }
}
impl <T> Clone for L1Sensitivity<T> {
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<T> Metric for L1Sensitivity<T> {
    type Distance = T;
}

pub struct L2Sensitivity<T> {
    _marker: PhantomData<T>
}
impl<T> L2Sensitivity<T> {
    pub fn new() -> Self {
        L2Sensitivity { _marker: PhantomData }
    }
}
impl <T> Clone for L2Sensitivity<T> {
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<T> Metric for L2Sensitivity<T> {
    type Distance = T;
}
