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
    measures::{Approximate, MultiDP, PrivacyGuarantee, PureDP, RenyiDP, zCDP},
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

/// Privacy loss can legitimately be infinite for a zero-scale mechanism.
/// Treat it as an absorbing value during composition instead of attempting
/// directed `0 + infinity`, which is numerically indeterminate.
fn privacy_loss_add(left: f64, right: f64) -> Fallible<f64> {
    if left.is_infinite() || right.is_infinite() {
        Ok(f64::INFINITY)
    } else {
        left.inf_add(&right)
    }
}

#[proven(proof_path = "combinators/sequential_composition/CompositionMeasure_for_PureDP.tex")]
impl CompositionMeasure for PureDP {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_mids
            .iter()
            .try_fold(0.0, |sum, d_i| privacy_loss_add(sum, *d_i))
    }
}

#[proven(proof_path = "combinators/sequential_composition/CompositionMeasure_for_zCDP.tex")]
impl CompositionMeasure for zCDP {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_mids
            .iter()
            .try_fold(0.0, |sum, d_i| privacy_loss_add(sum, *d_i))
    }
}

#[proven(proof_path = "combinators/sequential_composition/CompositionMeasure_for_ApproxDP.tex")]
impl CompositionMeasure for Approximate<PureDP> {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_mids
            .iter()
            .try_fold((0.0, 0.0), |(eps_g, del_g), (eps_i, del_i)| {
                Ok((privacy_loss_add(eps_g, *eps_i)?, del_g.inf_add(del_i)?))
            })
    }
}

/// `Approximate<zCDP>` uses OpenDP's existing distance semantics:
/// `(rho, delta)` is an approximate-zCDP guarantee, so sequential
/// composition adds both the rho parameters and their source deltas.
#[proven(proof_path = "combinators/sequential_composition/CompositionMeasure_for_ApproxZCDP.tex")]
impl CompositionMeasure for Approximate<zCDP> {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Sequential)
    }
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_mids
            .iter()
            .try_fold((0.0, 0.0), |(eps_g, del_g), (eps_i, del_i)| {
                Ok((privacy_loss_add(eps_g, *eps_i)?, del_g.inf_add(del_i)?))
            })
    }
}

#[proven(proof_path = "combinators/sequential_composition/CompositionMeasure_for_RenyiDP.tex")]
impl CompositionMeasure for RenyiDP {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }

    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        Ok(Function::new_fallible(move |alpha| {
            d_mids
                .iter()
                .map(|f| f.eval(alpha))
                .try_fold(0.0, |sum, eps| privacy_loss_add(sum, eps?))
        }))
    }
}

#[proven(proof_path = "combinators/sequential_composition/CompositionMeasure_for_MultiDP.tex")]
impl CompositionMeasure for MultiDP {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        // Representation-specific approximate-RDP and approximate-zCDP deltas
        // currently have sequential composition theorems. Privacy profiles and
        // tradeoff functions are intentionally not composition capabilities.
        Ok(Composability::Sequential)
    }

    fn compose_measure(measures: &[Self]) -> Fallible<Self> {
        if measures.is_empty() {
            return fallible!(MakeMeasurement, "Must have at least one measurement");
        }

        let capabilities = measures.iter().map(|measure| measure.capabilities());
        let capabilities = capabilities.collect::<Vec<_>>();

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
        if !zcdp && !renyi && !gaussian {
            return fallible!(
                FailedFunction,
                "MultiDP composition has no supported common capability path"
            );
        }

        let mut derived = crate::measures::PrivacyCapabilities::default();
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
