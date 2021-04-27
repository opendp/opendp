use crate::core::{Function, Measurement, PrivacyRelation};
use crate::dist::{MaxDivergence, L1Sensitivity};
use crate::dom::AllDomain;
use crate::error::*;
use crate::samplers::{SampleBernoulli, SampleGeometric, SampleUniform};
use crate::traits::DistanceCast;
use num::{Float, CheckedAdd, CheckedSub, Zero};


pub fn make_base_geometric<T, QO>(scale: QO, min: T, max: T) -> Fallible<Measurement<AllDomain<T>, AllDomain<T>, L1Sensitivity<T>, MaxDivergence<QO>>>
    where T: 'static + Clone + SampleGeometric + CheckedSub<Output=T> + CheckedAdd<Output=T> + DistanceCast + Zero,
          QO: 'static + Float + DistanceCast, f64: From<QO> {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    let max_trials: T = max - min;
    let alpha: f64 = (-f64::from(scale).recip()).exp();

    Ok(Measurement::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new_fallible(move |arg: &T| -> Fallible<T> {
            // return 0 noise with probability (1-alpha) / (1+alpha), otherwise sample from geometric
            Ok(if f64::sample_standard_uniform(false)? < (1. - alpha) / (1. + alpha) {
                arg.clone()
            } else {
                let noise = T::sample_geometric(1. - alpha, max_trials.clone(), false)?;
                if bool::sample_standard_bernoulli()? {
                    arg.clone().checked_add(&noise)
                } else {
                    arg.clone().checked_sub(&noise)
                }.unwrap_or_else(T::zero)
            })
        }),
        L1Sensitivity::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_from_constant(scale.recip())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_geometric_mechanism() {
        let measurement = make_base_geometric::<i32, f64>(10.0, 200, 210).unwrap_test();
        let arg = 205;
        let _ret = measurement.function.eval(&arg).unwrap_test();

        assert!(measurement.privacy_relation.eval(&1, &0.5).unwrap_test());
    }
}
