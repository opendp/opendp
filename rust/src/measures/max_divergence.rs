//! Production MaxDivergence definition shared with the Verus harness.
#[cfg(verus_only)]
use vstd::prelude::*;
#[cfg(verus_only)]
use crate::Measure;
#[cfg(not(verus_only))]
use crate::core::Measure;
use std::fmt::Debug;

/// Privacy measure used to define $\epsilon$-pure differential privacy.
///
/// In the following proof definition, $d$ corresponds to $\epsilon$ when also quantified over all adjacent datasets.
/// That is, $\epsilon$ is the greatest possible $d$
/// over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// # Proof Definition
///
/// ## `d`-closeness
///
/// For any two distributions $Y, Y'$ and any non-negative $d$,
/// $Y, Y'$ are $d$-close under the max divergence measure whenever
///
/// ```math
/// D_\infty(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S]}{\Pr[Y' \in S]} \Big] \leq d.
/// ```
#[cfg_attr(verus_only, verus_verify)]
#[derive(Default, Clone, Debug, PartialEq)]
pub struct MaxDivergence;

#[cfg_attr(verus_only, verus_verify)]
impl Measure for MaxDivergence {
    type Distance = f64;
}

