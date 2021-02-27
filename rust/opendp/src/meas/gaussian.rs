use std::marker::PhantomData;

use num::NumCast;

use crate::core::{Measurement, Function, PrivacyRelation};
use crate::dist::{L2Sensitivity, SmoothedMaxDivergence};
use crate::dom::AllDomain;
use crate::meas::{MakeMeasurement1, sample_gaussian};
use crate::error::Fallible;


pub struct GaussianMechanism<T> {
    data: PhantomData<T>
}

// gaussian for scalar-valued query
impl<T> MakeMeasurement1<AllDomain<T>, AllDomain<T>, L2Sensitivity<f64>, SmoothedMaxDivergence, f64> for GaussianMechanism<T>
    where T: Copy + NumCast {
    fn make1(sigma: f64) -> Fallible<Measurement<AllDomain<T>, AllDomain<T>, L2Sensitivity<f64>, SmoothedMaxDivergence>> {
        Ok(Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new_fallible(move |arg: &T| -> Fallible<T> {
                <f64 as NumCast>::from(*arg).ok_or_else(|| err!(FailedCast))
                    .and_then(|v| sample_gaussian(0., sigma, false)
                        .and_then(|noise|  T::from(v + noise).ok_or_else(|| err!(FailedCast))))
            }),
            L2Sensitivity::new(),
            SmoothedMaxDivergence::new(),
            PrivacyRelation::new_fallible(move |&d_in: &f64, &(eps, del): &(f64, f64)| {
                if d_in < 0. {
                    return fallible!(InvalidDistance, "gaussian mechanism: input sensitivity must be non-negative")
                }
                if eps <= 0. {
                    return fallible!(InvalidDistance, "gaussian mechanism: epsilon must be positive")
                }
                if del <= 0. {
                    return fallible!(InvalidDistance, "gaussian mechanism: delta must be positive")
                }
                // TODO: should we error if epsilon > 1., or just waste the budget?
                Ok(eps.min(1.) >= (d_in / sigma) * (2. * (1.25 / del).ln()).sqrt())
            })))
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_gaussian_mechanism() {
        let measurement = GaussianMechanism::<f64>::make(1.0);
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001)));
    }
}
