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
    traits::{InfAdd, InfMul},
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

impl ComposeK for MaxDivergence {
    fn compose_k(&self, d_mid: Self::Distance, k: u32) -> Fallible<Self::Distance> {
        d_mid.inf_mul(&f64::from(k))
    }
}

impl ComposeK for ZeroConcentratedDivergence {
    fn compose_k(&self, d_mid: Self::Distance, k: u32) -> Fallible<Self::Distance> {
        d_mid.inf_mul(&f64::from(k))
    }
}

impl ComposeK for Approximate<MaxDivergence> {
    fn compose_k(&self, (eps, del): Self::Distance, k: u32) -> Fallible<Self::Distance> {
        Ok((eps.inf_mul(&f64::from(k))?, del.inf_mul(&f64::from(k))?))
    }
}

impl ComposeK for Approximate<ZeroConcentratedDivergence> {
    fn compose_k(&self, (rho, del): Self::Distance, k: u32) -> Fallible<Self::Distance> {
        Ok((rho.inf_mul(&f64::from(k))?, del.inf_mul(&f64::from(k))?))
    }
}

impl ComposeK for RenyiDivergence {
    fn compose_k(&self, d_mid: Self::Distance, k: u32) -> Fallible<Self::Distance> {
        let k = f64::from(k);
        Ok(Function::new_fallible(move |alpha| {
            // the shared curve is evaluated once, then charged k times
            d_mid.eval(alpha)?.inf_mul(&k)
        }))
    }
}

#[cfg(test)]
mod test_compose_k {
    use super::*;

    /// `via_mul` must not be less than the exact sum, and should agree with the
    /// `inf_add` fold up to rounding.
    fn assert_bounds_fold(via_mul: f64, via_fold: f64, eps: f64, k: u32) {
        // rounded-to-nearest product is a lower bound on the round-up product
        assert!(via_mul >= (k as f64) * eps);
        assert!((via_mul - via_fold).abs() <= 1e-9 * via_fold.abs().max(f64::MIN_POSITIVE));
    }

    #[test]
    fn test_compose_k_bounds_composing_k_copies() -> Fallible<()> {
        for (eps, k) in [(0.3, 7), (1e-9, 1000), (0.5, 1), (0.1, 0)] {
            let via_mul = MaxDivergence.compose_k(eps, k)?;
            let via_fold = MaxDivergence.compose(vec![eps; k as usize])?;
            assert_bounds_fold(via_mul, via_fold, eps, k);

            let via_mul = ZeroConcentratedDivergence.compose_k(eps, k)?;
            let via_fold = ZeroConcentratedDivergence.compose(vec![eps; k as usize])?;
            assert_bounds_fold(via_mul, via_fold, eps, k);

            let curve = Function::new(move |_alpha: &f64| eps);
            let via_mul = RenyiDivergence.compose_k(curve.clone(), k)?;
            let via_fold = RenyiDivergence.compose(vec![curve; k as usize])?;
            assert_bounds_fold(via_mul.eval(&2.0)?, via_fold.eval(&2.0)?, eps, k);
        }
        Ok(())
    }

    #[test]
    fn test_compose_k_approximate() -> Fallible<()> {
        let (eps, del, k) = (0.3, 1e-9, 7);
        let via_mul = Approximate(MaxDivergence).compose_k((eps, del), k)?;
        let via_fold = Approximate(MaxDivergence).compose(vec![(eps, del); k as usize])?;
        assert_bounds_fold(via_mul.0, via_fold.0, eps, k);
        assert_bounds_fold(via_mul.1, via_fold.1, del, k);

        let via_mul = Approximate(ZeroConcentratedDivergence).compose_k((eps, del), k)?;
        let via_fold =
            Approximate(ZeroConcentratedDivergence).compose(vec![(eps, del); k as usize])?;
        assert_bounds_fold(via_mul.0, via_fold.0, eps, k);
        assert_bounds_fold(via_mul.1, via_fold.1, del, k);
        Ok(())
    }
}
