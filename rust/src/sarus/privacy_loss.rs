use std::{marker::PhantomData, rc::Rc};

use rug::Rational;

use crate::{core::Measure, error::Fallible};

use super::PLDistribution;

/// A Measure that comes with a privacy loss distribution.
#[derive(Clone)]
pub struct PLDSmoothedMaxDivergence<MI,Q> where MI:Clone, Q:Clone {
    _marker: PhantomData<Q>,
    privacy_loss_distribution: Rc<dyn Fn(&MI) -> Fallible<PLDistribution>>,
}

impl<MI,Q> PLDSmoothedMaxDivergence<MI,Q> where MI:Clone, Q:Clone {
    pub fn new(privacy_loss_distribution: impl Fn(&MI) -> Fallible<PLDistribution> + 'static) -> Self {
        PLDSmoothedMaxDivergence {
            _marker: PhantomData::<Q>::default(),
            privacy_loss_distribution: Rc::new(privacy_loss_distribution)
        }
    }
}

impl<MI,Q> Default for PLDSmoothedMaxDivergence<MI,Q> where MI:Clone, Q:Clone {
    fn default() -> Self {
        PLDSmoothedMaxDivergence::new(|m:&MI| Ok(PLDistribution::default()))
    }
}

impl<MI,Q> PartialEq for PLDSmoothedMaxDivergence<MI,Q> where MI:Clone, Q:Clone {
    fn eq(&self, _other: &Self) -> bool { true }
}

impl<MI,Q> Measure for PLDSmoothedMaxDivergence<MI,Q> where MI:Clone, Q:Clone {
    type Distance = (Q, Q);
}

// A way to build privacy relations from privacy loss distribution approximation

// fn make_pl_privacy_relation<MI>(out_dom: PLDomain<D>) -> PrivacyRelation<SymmetricDistance, SmoothedMaxDivergence<Rational>> {
//     PrivacyRelation::new_fallible( move |d_in: &IntDistance, (epsilon, delta): &(Rational, Rational)| {
//         if d_in<&0 {
//             return fallible!(InvalidDistance, "Privacy Loss Mechanism: input sensitivity must be non-negative")
//         }
//         if delta<=&0 {
//             return fallible!(InvalidDistance, "Privacy Loss Mechanism: delta must be positive")
//         }
//         let mut exp_epsilon = Float::with_val_round(64, epsilon, Round::Down).0;
//         exp_epsilon.exp_round(Round::Down);
//         Ok(delta >= &out_dom.delta(exp_epsilon))
//     })
// }

// fn make_gaussian_privacy_relation<T, MI>(scale: T) -> PrivacyRelation<MI, SmoothedMaxDivergence<T>>
//     where T: 'static + Clone + SampleGaussian + Float + InfCast<f64>,
//           MI: SensitivityMetric<Distance=T> {
//     PrivacyRelation::new_fallible(move |&d_in: &T, &(eps, del): &(T, T)| {
//         let _2 = T::inf_cast(2.)?;
//         let additive_gauss_const = T::inf_cast(ADDITIVE_GAUSS_CONST)?;

//         if d_in.is_sign_negative() {
//             return fallible!(InvalidDistance, "gaussian mechanism: input sensitivity must be non-negative")
//         }
//         if eps.is_sign_negative() || eps.is_zero() {
//             return fallible!(InvalidDistance, "gaussian mechanism: epsilon must be positive")
//         }
//         if del.is_sign_negative() || del.is_zero() {
//             return fallible!(InvalidDistance, "gaussian mechanism: delta must be positive")
//         }

//         // TODO: should we error if epsilon > 1., or just waste the budget?
//         Ok(eps.min(T::one()) >= (d_in / scale) * (additive_gauss_const + _2 * del.recip().ln()).sqrt())
//     })
// }
