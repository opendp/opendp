#[cfg(feature = "contrib")]
mod non_adaptive;
#[cfg(feature = "contrib")]
pub use non_adaptive::*;

#[cfg(feature = "contrib")]
mod adaptive;
#[cfg(feature = "contrib")]
pub use adaptive::*;

#[cfg(feature = "contrib")]
mod fully_adaptive;
#[cfg(feature = "contrib")]
pub use fully_adaptive::*;

use opendp_derive::proven;

#[cfg(feature = "ffi")]
mod ffi;

use crate::{
    core::{Function, Measure},
    error::Fallible,
    measures::{Approximate, MaxDivergence, RenyiDivergence, ZeroConcentratedDivergence},
    traits::InfAdd,
};

#[derive(Debug)]
pub enum Adaptivity {
    /// All queries are executed together in a single batch.
    NonAdaptive,
    /// The privacy loss parameters are non-adaptive,
    /// but the queries can be chosen adaptively based on the results of previous queries.
    Adaptive,
    /// The privacy loss parameters and queries can be chosen adaptively
    /// based on the results of previous queries.
    FullyAdaptive,
}

#[derive(Debug)]
pub enum Composability {
    /// Previous interactive mechanisms are locked when a new query is submitted.
    Sequential,
    /// Previous interactive mechanisms are not locked when a new query is submitted.
    Concurrent,
}

/// # Proof Definition
/// `composability` returns `Ok(out)` if the composition of a vector of privacy parameters `d_mids`
/// is bounded above by `self.compose(d_mids)` under `adaptivity` adaptivity and `out`-composability.
/// Otherwise returns an error.
pub trait CompositionMeasure: Measure {
    fn composability(&self, adaptivity: Adaptivity) -> Fallible<Composability>;
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance>;
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_MaxDivergence.tex"
)]
impl CompositionMeasure for MaxDivergence {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_mids.iter().try_fold(0.0, |sum, d_i| sum.inf_add(d_i))
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_ZeroConcentratedDivergence.tex"
)]
impl CompositionMeasure for ZeroConcentratedDivergence {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_mids.iter().try_fold(0.0, |sum, d_i| sum.inf_add(d_i))
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_ApproximateMaxDivergence.tex"
)]
impl CompositionMeasure for Approximate<MaxDivergence> {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_mids
            .iter()
            .try_fold((0.0, 0.0), |(eps_g, del_g), (eps_i, del_i)| {
                Ok((eps_g.inf_add(eps_i)?, del_g.inf_add(del_i)?))
            })
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_ApproximateZeroConcentratedDivergence.tex"
)]
impl CompositionMeasure for Approximate<ZeroConcentratedDivergence> {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Sequential)
    }
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_mids
            .iter()
            .try_fold((0.0, 0.0), |(eps_g, del_g), (eps_i, del_i)| {
                Ok((eps_g.inf_add(eps_i)?, del_g.inf_add(del_i)?))
            })
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_RenyiDivergence.tex"
)]
impl CompositionMeasure for RenyiDivergence {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }

    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        Ok(Function::new_fallible(move |alpha| {
            d_mids
                .iter()
                .map(|f| f.eval(alpha))
                .try_fold(0.0, |sum, eps| sum.inf_add(&eps?))
        }))
    }
}

pub trait ComposeK: CompositionMeasure {
    fn compose_k(&self, d_mid: Self::Distance, k: u32) -> Fallible<Self::Distance>
    where
        Self::Distance: Clone,
    {
        self.compose(vec![d_mid; k as usize])
    }
}

impl ComposeK for MaxDivergence {}
impl ComposeK for ZeroConcentratedDivergence {}
impl ComposeK for Approximate<MaxDivergence> {}
impl ComposeK for Approximate<ZeroConcentratedDivergence> {}

impl ComposeK for RenyiDivergence {
    fn compose_k(&self, d_mid: Self::Distance, k: u32) -> Fallible<Self::Distance>
    where
        Self::Distance: Clone,
    {
        Ok(Function::new_fallible(move |alpha| {
            // the shared curve is evaluated once, then charged k times
            inf_add_k(d_mid.eval(alpha)?, k)
        }))
    }
}

/// Upper-bounds the sum of `k` copies of `eps` with k-fold `inf_add`a
fn inf_add_k(eps: f64, k: u32) -> Fallible<f64> {
    (0..k).try_fold(0.0, |sum, _| sum.inf_add(&eps))
}

#[cfg(test)]
mod test_compose_k {
    use super::*;

    /// Upper-bounds the sum of `k` copies of `eps` with ~2logk `inf_add`s
    /// Reduces floating point exposure and would require tolerance for comparisons
    /// with compose
    fn inf_add_k_doubling(eps: f64, mut k: u32) -> Fallible<f64> {
        let mut total = 0.0;
        let mut power = eps;
        loop {
            if k & 1 == 1 {
                total = total.inf_add(&power)?;
            }
            k >>= 1;
            if k == 0 {
                return Ok(total);
            }
            power = power.inf_add(&power)?;
        }
    }

    #[test]
    fn test_compose_k_matches_composing_k_copies() -> Fallible<()> {
        for (eps, k) in [(0.3, 7), (1e-9, 1000), (0.5, 1), (0.1, 0)] {
            let curve = Function::new(move |_alpha: &f64| eps);
            let via_compose_k = RenyiDivergence.compose_k(curve.clone(), k)?;
            let via_compose = RenyiDivergence.compose(vec![curve; k as usize])?;
            assert_eq!(via_compose_k.eval(&2.0)?, via_compose.eval(&2.0)?);
        }
        Ok(())
    }

    #[test]
    fn test_doubling_candidate_matches_inf_add_k() -> Fallible<()> {
        for eps in [0.0, 1e-300, 1e-9, 0.1, 1.0 / 3.0, 1.0, 1e9] {
            for k in [0u32, 1, 2, 3, 4, 5, 7, 8, 100, 999, 12345] {
                let linear = inf_add_k(eps, k)?;
                let doubled = inf_add_k_doubling(eps, k)?;

                // both upper-bound the exact sum k * eps
                let exact_lo = (k as f64) * eps * (1.0 - 1e-12);
                assert!(linear >= exact_lo);
                assert!(doubled >= exact_lo);
                assert!((doubled - linear).abs() <= 1e-9 * linear.abs().max(f64::MIN_POSITIVE));
            }
        }
        Ok(())
    }
}
