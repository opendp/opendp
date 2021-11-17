#[cfg(feature="ffi")]
mod ffi;

use crate::core::{Function, Measurement, PrivacyRelation, Domain, SensitivityMetric};
use crate::dist::{MaxDivergence, L1Distance, AbsoluteDistance};
use crate::dom::{AllDomain, VectorDomain};
use crate::error::*;
use crate::samplers::SampleTwoSidedGeometric;
use num::Float;
use crate::traits::{DistanceConstant, InfCast, CheckNull, TotalOrd};


pub trait GeometricDomain: Domain {
    type InputMetric: SensitivityMetric<Distance=Self::Atom> + Default;
    // Atom is an alias for Self::InputMetric::Distance.
    // It would be possible to fill this with associated type defaults: https://github.com/rust-lang/rust/issues/29661
    type Atom;
    fn new() -> Self;
    fn noise_function(scale: f64, bounds: Option<(Self::Atom, Self::Atom)>) -> Function<Self, Self>;
}


impl<T> GeometricDomain for AllDomain<T>
    where T: 'static + Clone + SampleTwoSidedGeometric + CheckNull {
    type InputMetric = AbsoluteDistance<T>;
    type Atom = T;

    fn new() -> Self { AllDomain::new() }
    fn noise_function(scale: f64, bounds: Option<(T, T)>) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier|
            T::sample_two_sided_geometric(arg.clone(), scale, bounds.clone()))
    }
}

impl<T> GeometricDomain for VectorDomain<AllDomain<T>>
    where T: 'static + Clone + SampleTwoSidedGeometric + CheckNull {
    type InputMetric = L1Distance<T>;
    type Atom = T;

    fn new() -> Self { VectorDomain::new_all() }
    fn noise_function(scale: f64, bounds: Option<(T, T)>) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| arg.iter()
            .map(|v| T::sample_two_sided_geometric(v.clone(), scale, bounds.clone()))
            .collect())
    }
}

pub fn make_base_geometric<D, QO>(
    scale: QO, bounds: Option<(D::Atom, D::Atom)>
) -> Fallible<Measurement<D, D, D::InputMetric, MaxDivergence<QO>>>
    where D: 'static + GeometricDomain,
          D::Atom: 'static + TotalOrd + Clone + InfCast<QO>,
          QO: 'static + Float + DistanceConstant<D::Atom>,
          f64: InfCast<QO> {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    if bounds.as_ref().map(|(lower, upper)| lower > upper).unwrap_or(false) {
        return fallible!(MakeMeasurement, "lower may not be greater than upper")
    }

    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(f64::inf_cast(scale)?, bounds),
        D::InputMetric::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_all(
            move |d_in: &D::Atom, d_out: &QO| {
                let d_in = QO::inf_cast(d_in.clone())?;
                if d_in.is_sign_negative() {
                    return fallible!(InvalidDistance, "sensitivity must be non-negative")
                }
                if d_out.is_sign_negative() {
                    return fallible!(InvalidDistance, "epsilon must be non-negative")
                }
                // d_out * scale >= d_in
                Ok(d_out.neg_inf_mul(&scale)? >= d_in)
            },
            Some(move |d_out: &QO| D::Atom::inf_cast(d_out.neg_inf_mul(&scale)?)))
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_geometric_mechanism_bounded() {
        let measurement = make_base_geometric::<AllDomain<_>, f64>(10.0, Some((200, 210))).unwrap_test();
        let arg = 205;
        let _ret = measurement.invoke(&arg).unwrap_test();
        println!("{:?}", _ret);

        assert!(measurement.check(&1, &0.5).unwrap_test());
    }

    #[test]
    fn test_make_vector_geometric_mechanism_bounded() {
        let measurement = make_base_geometric::<VectorDomain<_>, f64>(10.0, Some((200, 210))).unwrap_test();
        let arg = vec![1, 2, 3, 4];
        let _ret = measurement.invoke(&arg).unwrap_test();
        println!("{:?}", _ret);

        assert!(measurement.check(&1, &0.5).unwrap_test());
    }

    #[test]
    fn test_make_geometric_mechanism() {
        let measurement = make_base_geometric::<AllDomain<_>, f64>(10.0, None).unwrap_test();
        let arg = 205;
        let _ret = measurement.invoke(&arg).unwrap_test();
        println!("{:?}", _ret);

        assert!(measurement.check(&1, &0.5).unwrap_test());
    }

    #[test]
    fn test_make_vector_geometric_mechanism() {
        let measurement = make_base_geometric::<VectorDomain<_>, f64>(10.0, None).unwrap_test();
        let arg = vec![1, 2, 3, 4];
        let _ret = measurement.invoke(&arg).unwrap_test();
        println!("{:?}", _ret);

        assert!(measurement.check(&1, &0.5).unwrap_test());
    }
}
