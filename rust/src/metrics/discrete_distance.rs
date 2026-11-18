//! Production DiscreteDistance definition shared with the Verus harness.
#[cfg(verus_only)]
use vstd::prelude::*;
#[cfg(verus_only)]
use crate::Metric;
#[cfg(not(verus_only))]
use crate::core::Metric;
#[cfg(not(verus_only))]
use super::IntDistance;
#[cfg(verus_only)]
type IntDistance = u32;
use std::fmt::{Debug, Formatter};

/// Indicates if two elements are equal to each other.
///
/// This is used in the context of randomized response,
/// to capture the distance between adjacent inputs (they are either equivalent or not).
///
/// # Proof Definition
///
/// ### `d`-closeness
/// For any two datasets $u, v \in$ `AtomDomain<T>` and any $d$ of type [`IntDistance`],
/// we say that $u, v$ are $d$-close under the discrete metric (abbreviated as $d_{Eq}$) whenever
///
/// ```math
/// d_{Eq}(u, v) = \mathbb{1}[u \ne v] \leq d
/// ```
///
/// # Notes
/// Clearly, `d` is bounded above by 1.
/// 1 is the expected argument on measurements that use this distance.
///
/// # Compatible Domains
/// * `AtomDomain<T>` for any valid `T`.
#[cfg_attr(verus_only, verus_verify)]
#[derive(Clone, Default, PartialEq)]
pub struct DiscreteDistance;

impl Debug for DiscreteDistance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "DiscreteDistance()")
    }
}
#[cfg_attr(verus_only, verus_verify)]
impl Metric for DiscreteDistance {
    type Distance = IntDistance;
}


#[cfg(verus_only)]
verus! {
impl DiscreteDistance {
    /// The documented discrete metric: distinct Boolean values are
    /// one unit apart, identical values are zero apart.
    pub open spec fn bool_close(left: bool, right: bool, distance: u32) -> bool {
        left == right || distance > 0u32
    }
}
}
