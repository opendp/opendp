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


#[cfg(verus_only)]
verus! {
impl<Y> crate::CertifiedMeasure<Result<Y, ()>> for MaxDivergence {
    open spec fn valid_cost(&self, cost: &f64) -> bool {
        crate::verified_arithmetic::finite(*cost)
            || f64_to_bits(*cost) == 0x7ff0_0000_0000_0000u64
    }

    open spec fn dominates(
        &self,
        left: spec_fn(spec_fn(Result<Y, ()>) -> real) -> real,
        right: spec_fn(spec_fn(Result<Y, ()>) -> real) -> real,
        cost: &f64,
    ) -> bool {
        match MaxDivergence::multiplicative_factor(*cost) {
            None => true,
            Some(factor) => factor >= 1real &&
                forall|payoff: spec_fn(Result<Y, ()>) -> real|
                    crate::measurement_proof::bounded_payoff(payoff) ==>
                    #[trigger] left(payoff) <= factor * right(payoff),
        }
    }
}

impl MaxDivergence {
    /// The multiplicative interpretation of a reported pure-DP epsilon.
    /// An unbounded +infinity value imposes no finite factor bound.
    /// Numeric validity (finite or +infinity, nonnegative) is checked
    /// separately at the certified constructor.
    pub open spec fn multiplicative_factor(epsilon: f64) -> Option<real> {
        if f64_to_bits(epsilon) == 0x7ff0_0000_0000_0000u64 {
            None
        } else if f64_to_bits(epsilon) == 0u64 {
            Some(1real)
        } else {
            Some(crate::verified_arithmetic::real_exp(
                crate::verified_arithmetic::float_value(epsilon)))
        }
    }
}
}
