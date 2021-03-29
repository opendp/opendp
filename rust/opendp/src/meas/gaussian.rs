use std::marker::PhantomData;

use num::Float;

use crate::core::{Function, Measurement, PrivacyRelation};
use crate::dist::{L2Sensitivity, SmoothedMaxDivergence};
use crate::dom::AllDomain;
use crate::error::Fallible;
use crate::meas::MakeMeasurement1;
use crate::samplers::SampleGaussian;

pub struct BaseGaussian<T> {
    data: PhantomData<T>
}

// gaussian for scalar-valued query
impl<T> MakeMeasurement1<AllDomain<T>, AllDomain<T>, L2Sensitivity<T>, SmoothedMaxDivergence<T>, T> for BaseGaussian<T>
    where T: 'static + Clone + SampleGaussian + Float {
    fn make1(scale: T) -> Fallible<Measurement<AllDomain<T>, AllDomain<T>, L2Sensitivity<T>, SmoothedMaxDivergence<T>>> {
        let _2_ = T::from(2.).ok_or_else(|| err!(FailedCast))?;
        let _1_25 = T::from(1.25).ok_or_else(|| err!(FailedCast))?;

        Ok(Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new_fallible(move |arg: &T| -> Fallible<T> {
                T::sample_gaussian(arg.clone(), scale.clone(), false)
            }),
            L2Sensitivity::new(),
            SmoothedMaxDivergence::new(),
            PrivacyRelation::new_fallible(move |&d_in: &T, &(eps, del): &(T, T)| {
                if d_in.is_sign_negative() {
                    return fallible!(InvalidDistance, "gaussian mechanism: input sensitivity must be non-negative")
                }
                if eps.is_sign_negative() || eps.is_zero() {
                    return fallible!(InvalidDistance, "gaussian mechanism: epsilon must be positive")
                }
                if del.is_sign_negative() || del.is_zero() {
                    return fallible!(InvalidDistance, "gaussian mechanism: delta must be positive")
                }

                // TODO: should we error if epsilon > 1., or just waste the budget?
                Ok(eps.min(T::one()) >= (d_in / scale) * (_2_ * (_1_25 / del).ln()).sqrt())
            })))
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_gaussian_mechanism() {
        let measurement = BaseGaussian::<f64>::make(1.0).unwrap();
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg).unwrap();

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001)).unwrap());
    }
}
