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

// Divergence measure from: https://arxiv.org/pdf/1905.02383.pdf
#[derive(Clone)]
pub struct GaussianTradeOff<T> {
    _phantom: PhantomData<T>
}

impl<T> Default for GaussianTradeOff<T> {
    fn default() -> Self {
        GaussianTradeOff {_phantom: PhantomData}
    }
}
impl<T> Debug for GaussianTradeOff<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "GaussianTradeOff()")
    }
}
// The Gaussian Differential Privacy distance type consists of just mu
impl<T: Clone> Measure for GaussianTradeOff<T> {
    type Distance = T;
}
// All instances of the gaussian tradeoff measure are equivalent
impl<T> PartialEq for GaussianTradeOff<T> {
    fn eq(&self, _other: &Self) -> bool { true }
}

// Divergence measure from https://arxiv.org/pdf/1702.07476.pdf
#[derive(Clone)]
pub struct RenyiDivergence<T> {
    _phantom: PhantomData<T>,
    pub alpha: i32
}
impl<T> RenyiDivergence<T> {
    pub fn new(alpha: i32) -> Self {
        RenyiDivergence { _phantom: Default::default(), alpha }
    }
}

impl<T> Debug for RenyiDivergence<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RenyiDivergence()")
    }
}
impl<T> Default for RenyiDivergence<T> {
    fn default() -> Self {
        RenyiDivergence {
            _phantom: PhantomData,
            // TODO: this is not valid! We can't just hardcode alpha!
            alpha: 0
        }
    }
}
// The Gaussian Differential Privacy distance type consists of just mu
impl<T: Clone> Measure for RenyiDivergence<T> {
    type Distance = T;
}
// All instances of the gaussian tradeoff measure are equivalent
impl<T> PartialEq for RenyiDivergence<T> {
    fn eq(&self, other: &Self) -> bool { self.alpha == other.alpha }
}

// for zero-concentrated DP, where the union bound is considered over all alpha in [1, inf]
// See https://arxiv.org/pdf/1605.02065.pdf#page=4 Definition 1.1
#[derive(Clone)]
pub struct UnionRenyiDivergence<T> {
    _phantom: PhantomData<T>
}

impl<T> Default for UnionRenyiDivergence<T> {
    fn default() -> Self {
        UnionRenyiDivergence {_phantom: PhantomData}
    }
}
impl<T> Debug for UnionRenyiDivergence<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UnionRenyiDivergence()")
    }
}
// The zCDP Differential Privacy distance type consists of just rho
impl<T: Clone> Measure for UnionRenyiDivergence<T> {
    type Distance = T;
}
// All instances of the gaussian tradeoff measure are equivalent
impl<T> PartialEq for UnionRenyiDivergence<T> {
    fn eq(&self, _other: &Self) -> bool { true }
}


// for approximate zero-concentrated DP, where the union bound is considered over all alpha in [1, inf]
// See https://arxiv.org/pdf/1605.02065.pdf#page=4 Definition 1.1
#[derive(Clone)]
pub struct SmoothedUnionRenyiDivergence<T> {
    _phantom: PhantomData<T>
}

impl<T> Default for SmoothedUnionRenyiDivergence<T> {
    fn default() -> Self {
        SmoothedUnionRenyiDivergence {_phantom: PhantomData}
    }
}
impl<T> Debug for SmoothedUnionRenyiDivergence<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SmoothedUnionRenyiDivergence()")
    }
}
// The zCDP Differential Privacy distance type consists of just rho
impl<T: Clone> Measure for SmoothedUnionRenyiDivergence<T> {
    type Distance = T;
}
// All instances of the gaussian tradeoff measure are equivalent
impl<T> PartialEq for SmoothedUnionRenyiDivergence<T> {
    fn eq(&self, _other: &Self) -> bool { true }
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
