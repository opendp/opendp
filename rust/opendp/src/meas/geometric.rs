use std::marker::PhantomData;
use std::ops::{Sub, Add};


use crate::core::{Function, Measurement, PrivacyRelation};
use crate::dist::{MaxDivergence, L1Sensitivity};
use crate::dom::AllDomain;
use crate::error::*;
use crate::samplers::{SampleBernoulli, SampleGeometric, SampleUniform};
use crate::meas::MakeMeasurement3;
use crate::traits::DistanceCast;
use num::Float;

pub struct BaseSimpleGeometric<T, QO> {
    data: PhantomData<T>,
    output_distance: PhantomData<QO>
}

// geometric for scalar-valued query
impl<T, QO> MakeMeasurement3<AllDomain<T>, AllDomain<T>, L1Sensitivity<T>, MaxDivergence<QO>, QO, T, T> for BaseSimpleGeometric<T, QO>
    where T: 'static + Clone + SampleGeometric + Sub<Output=T> + Add<Output=T> + DistanceCast,
          QO: 'static + Float + DistanceCast, f64: From<QO> {
    fn make3(scale: QO, min: T, max: T) -> Fallible<Measurement<AllDomain<T>, AllDomain<T>, L1Sensitivity<T>, MaxDivergence<QO>>> {
        if scale.is_sign_negative() {
            return fallible!(MakeMeasurement, "scale must not be negative")
        }
        Ok(Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new_fallible(move |arg: &T| -> Fallible<T> {
                // cast up to f64
                let alpha: f64 = std::f64::consts::E.powf(-1. / f64::from(scale));
                let max_trials: T = max - min;

                // return 0 noise with probability (1-alpha) / (1+alpha), otherwise sample from geometric
                Ok(if f64::sample_standard_uniform(false)? < (1. - alpha) / (1. + alpha) {
                    arg.clone()
                } else if bool::sample_standard_bernoulli()? {
                    arg.clone() + T::sample_geometric(1. - alpha, max_trials, false)?
                } else {
                    arg.clone() - T::sample_geometric(1. - alpha, max_trials, false)?
                })
            }),
            L1Sensitivity::new(),
            MaxDivergence::new(),
            PrivacyRelation::new_from_constant(scale.recip())))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_geometric_mechanism() {
        let measurement = BaseSimpleGeometric::<i32, f64>::make(10.0, 200, 210).unwrap_test();
        let arg = 205;
        let _ret = measurement.function.eval(&arg).unwrap_test();

        assert!(measurement.privacy_relation.eval(&1, &0.5).unwrap_test());
    }
}
