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
        Approximate, MaxDivergence, MultiDP, PrivacyBudget, PrivacyGuarantee, RenyiDivergence,
        ZeroConcentratedDivergence,
    },
    traits::{InfAdd, ProductOrd},
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    type Budget: 'static + Clone + std::fmt::Debug;

    /// Interpret a fixed budget as a privacy fact for composition.
    fn budget_guarantee(&self, _budget: &Self::Budget) -> Fallible<Self::Distance> {
        fallible!(MakeMeasurement, "this measure does not support fixed budgets")
    }

    /// Check that a privacy fact certifies a fixed budget.
    fn check_budget(&self, _distance: &Self::Distance, _budget: &Self::Budget) -> Fallible<bool> {
        fallible!(FailedFunction, "this measure does not support fixed budgets")
    }

    fn composability(&self, adaptivity: Adaptivity) -> Fallible<Composability>;

    /// Compose privacy facts under the interaction model enforced by the caller.
    fn compose(&self, d_mids: Vec<Self::Distance>, _composability: Composability) -> Fallible<Self::Distance>;
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_MaxDivergence.tex"
)]
impl CompositionMeasure for MaxDivergence {
    type Budget = Self::Distance;

    fn budget_guarantee(&self, budget: &Self::Budget) -> Fallible<Self::Distance> {
        Ok(*budget)
    }

    fn check_budget(&self, distance: &Self::Distance, budget: &Self::Budget) -> Fallible<bool> {
        budget.total_ge(distance)
    }

    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<Self::Distance>, _composability: Composability) -> Fallible<Self::Distance> {
        d_mids.iter().try_fold(0.0, |sum, d_i| sum.inf_add(d_i))
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_ZeroConcentratedDivergence.tex"
)]
impl CompositionMeasure for ZeroConcentratedDivergence {
    type Budget = Self::Distance;

    fn budget_guarantee(&self, budget: &Self::Budget) -> Fallible<Self::Distance> {
        Ok(*budget)
    }

    fn check_budget(&self, distance: &Self::Distance, budget: &Self::Budget) -> Fallible<bool> {
        budget.total_ge(distance)
    }

    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<Self::Distance>, _composability: Composability) -> Fallible<Self::Distance> {
        d_mids.iter().try_fold(0.0, |sum, d_i| sum.inf_add(d_i))
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_ApproximateMaxDivergence.tex"
)]
impl CompositionMeasure for Approximate<MaxDivergence> {
    type Budget = Self::Distance;

    fn budget_guarantee(&self, budget: &Self::Budget) -> Fallible<Self::Distance> {
        Ok(*budget)
    }

    fn check_budget(&self, distance: &Self::Distance, budget: &Self::Budget) -> Fallible<bool> {
        budget.total_ge(distance)
    }

    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<Self::Distance>, _composability: Composability) -> Fallible<Self::Distance> {
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
    type Budget = Self::Distance;

    fn budget_guarantee(&self, budget: &Self::Budget) -> Fallible<Self::Distance> {
        Ok(*budget)
    }

    fn check_budget(&self, distance: &Self::Distance, budget: &Self::Budget) -> Fallible<bool> {
        budget.total_ge(distance)
    }

    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Sequential)
    }
    fn compose(&self, d_mids: Vec<Self::Distance>, _composability: Composability) -> Fallible<Self::Distance> {
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
    // Arbitrary RDP functions are not comparable budgets.
    type Budget = ();

    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }

    fn compose(&self, d_mids: Vec<Self::Distance>, _composability: Composability) -> Fallible<Self::Distance> {
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
    type Budget = PrivacyBudget;

    fn budget_guarantee(&self, budget: &Self::Budget) -> Fallible<Self::Distance> {
        budget.to_guarantee()
    }

    fn check_budget(&self, distance: &Self::Distance, budget: &Self::Budget) -> Fallible<bool> {
        budget.allows(distance)
    }

    fn composability(&self, adaptivity: Adaptivity) -> Fallible<Composability> {
        match adaptivity {
            Adaptivity::NonAdaptive | Adaptivity::Adaptive => Ok(Composability::Concurrent),
            // A generic conjunction of runtime privacy facts is not a
            // fully-adaptive filter theorem. Require a fixed accounting budget.
            Adaptivity::FullyAdaptive => fallible!(
                MakeMeasurement,
                "fully adaptive MultiDP composition requires a budget-specific theorem"
            ),
        }
    }

    fn compose(
        &self,
        d_mids: Vec<Self::Distance>,
        composability: Composability,
    ) -> Fallible<Self::Distance> {
        PrivacyGuarantee::compose(d_mids, composability)
    }
}
