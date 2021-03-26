use std::marker::PhantomData;
use std::ops::{Sub, Add};


use crate::core::{Function, Measurement, PrivacyRelation};
use crate::dist::{L2Sensitivity, MaxDivergence};
use crate::dom::AllDomain;
use crate::error::Fallible;
use crate::meas::{MakeMeasurement3, SampleBernoulli, SampleGeometric, SampleUniform};
use crate::traits::DistanceCast;

pub struct BaseSimpleGeometric<T> {
    data: PhantomData<T>
}

// geometric for scalar-valued query
impl<T> MakeMeasurement3<AllDomain<T>, AllDomain<T>, L2Sensitivity<T>, MaxDivergence<f64>, f64, T, T> for BaseSimpleGeometric<T>
    where T: 'static + Clone + SampleGeometric + Sub<Output=T> + Add<Output=T> + DistanceCast {
    fn make3(scale: f64, min: T, max: T) -> Fallible<Measurement<AllDomain<T>, AllDomain<T>, L2Sensitivity<T>, MaxDivergence<f64>>> {
        Ok(Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new_fallible(move |arg: &T| -> Fallible<T> {
                let alpha: f64 = std::f64::consts::E.powf(-1. / scale);
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
            L2Sensitivity::new(),
            MaxDivergence::new(),
            PrivacyRelation::new_from_constant(scale.recip())))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_geometric_mechanism() {
        let measurement = BaseSimpleGeometric::<i32>::make(10.0, 200, 210).unwrap();
        let arg = 205;
        let _ret = measurement.function.eval(&arg).unwrap();

        assert!(measurement.privacy_relation.eval(&1, &0.5).unwrap());
    }
}
