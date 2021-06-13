use crate::core::{Function, Measurement, PrivacyRelation, Domain};
use crate::dist::{MaxDivergence, L1Sensitivity};
use crate::dom::{AllDomain, VectorDomain};
use crate::error::*;
use crate::samplers::SampleTwoSidedGeometric;
use crate::traits::DistanceCast;
use num::Float;
use std::ops::Sub;


pub trait GeometricDomain: Domain {
    type Atom;
    fn new() -> Self;
    fn noise_function(scale: f64, max_trials: Self::Atom, constant_time: bool) -> Function<Self, Self>;
}


impl<T> GeometricDomain for AllDomain<T>
    where T: 'static + Clone + SampleTwoSidedGeometric {
    type Atom = Self::Carrier;

    fn new() -> Self { AllDomain::new() }
    fn noise_function(scale: f64, max_trials: T, constant_time: bool) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier|
            T::sample_two_sided_geometric(arg.clone(), scale, max_trials.clone(), constant_time))
    }
}

impl<T> GeometricDomain for VectorDomain<AllDomain<T>>
    where T: 'static + Clone + SampleTwoSidedGeometric {
    type Atom = T;

    fn new() -> Self { VectorDomain::new_all() }
    fn noise_function(scale: f64, max_trials: T, constant_time: bool) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| arg.iter()
            .map(|v| T::sample_two_sided_geometric(v.clone(), scale, max_trials.clone(), constant_time))
            .collect())
    }
}

pub fn make_base_geometric<D, QO>(
    scale: QO, min: D::Atom, max: D::Atom, constant_time: bool
) -> Fallible<Measurement<D, D, L1Sensitivity<D::Atom>, MaxDivergence<QO>>>
    where D: 'static + GeometricDomain,
          D::Atom: 'static + DistanceCast + Sub<Output=D::Atom>,
          QO: 'static + Float + DistanceCast,
          f64: From<QO> {
    if scale.is_sign_negative() { return fallible!(MakeMeasurement, "scale must not be negative") }
    let max_trials: D::Atom = max - min;

    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(f64::from(scale), max_trials, constant_time),
        L1Sensitivity::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_from_constant(scale.recip())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_geometric_mechanism() {
        let measurement = make_base_geometric::<AllDomain<_>, f64>(10.0, 200, 210, false).unwrap_test();
        let arg = 205;
        let _ret = measurement.function.eval(&arg).unwrap_test();

        assert!(measurement.privacy_relation.eval(&1, &0.5).unwrap_test());
    }
}
