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
    measures::{
        Approximate, MaxDivergence, MultiDP, PrivacyGuarantee, RenyiDivergence,
        ZeroConcentratedDivergence,
    },
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

    /// Derive the output measure from the measures of all child measurements.
    ///
    /// Most measures are only composable when all children use the same
    /// measure, so the default implementation preserves that behavior.
    /// Measures whose equality intentionally ignores construction metadata can
    /// override this hook to derive a sound output descriptor instead.
    fn compose_measure(measures: &[Self]) -> Fallible<Self> {
        let Some(first) = measures.first() else {
            return fallible!(MakeMeasurement, "Must have at least one measurement");
        };
        if !measures.iter().all(|measure| measure == first) {
            return fallible!(MetricMismatch, "All output measures must be the same");
        }
        Ok(first.clone())
    }

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

#[proven(proof_path = "combinators/sequential_composition/CompositionMeasure_for_MultiDP.tex")]
impl CompositionMeasure for MultiDP {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        // Representation-specific approximate-RDP and approximate-zCDP deltas
        // currently have sequential composition theorems. Analytical profiles
        // are intentionally not composition capabilities.
        Ok(Composability::Sequential)
    }

    fn compose_measure(measures: &[Self]) -> Fallible<Self> {
        if measures.is_empty() {
            return fallible!(MakeMeasurement, "Must have at least one measurement");
        }

        let capabilities = measures.iter().map(|measure| measure.capabilities());
        let capabilities = capabilities.collect::<Vec<_>>();

        // Exact approximate-DP certificates compose independently of profiles.
        let approx_dp = capabilities.iter().all(|caps| caps.approx_dp().is_some());

        // Native zCDP composition is available whenever every child
        // guarantees zCDP. Purity is the meet of the child guarantees.
        let zcdp = capabilities.iter().all(|caps| caps.zcdp().is_some());

        // Pure zCDP embeds into pure RDP. Native pure RDP is the other
        // supported RDP path. Approximate representations are deliberately
        // excluded from this exact-RDP capability.
        let renyi = capabilities.iter().all(|caps| {
            caps.renyi() == Some(crate::measures::Purity::Pure)
                || caps.zcdp() == Some(crate::measures::Purity::Pure)
        });

        let gaussian = capabilities.iter().all(|caps| caps.gaussian());
        if !approx_dp && !zcdp && !renyi && !gaussian {
            return fallible!(
                FailedFunction,
                "MultiDP composition has no supported common capability path"
            );
        }

        let approx_purity = approx_dp.then(|| {
            if capabilities
                .iter()
                .all(|caps| caps.approx_dp() == Some(crate::measures::Purity::Pure))
            {
                crate::measures::Purity::Pure
            } else {
                crate::measures::Purity::Approximate
            }
        });
        let mut derived = crate::measures::PrivacyCapabilities::new(approx_purity, None);
        if zcdp {
            let purity = if capabilities
                .iter()
                .all(|caps| caps.zcdp() == Some(crate::measures::Purity::Pure))
            {
                crate::measures::Purity::Pure
            } else {
                crate::measures::Purity::Approximate
            };
            derived = derived.with_zcdp(purity);
        }
        if renyi {
            derived = derived.with_renyi(crate::measures::Purity::Pure);
        }
        if gaussian {
            derived = derived.with_gaussian();
        }

        // Keep this independent of the first child's descriptor. MultiDP
        // equality intentionally ignores capability richness.
        Ok(MultiDP::new(derived))
    }

    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        PrivacyGuarantee::compose(d_mids)
    }
}
