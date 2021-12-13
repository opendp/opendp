#[cfg(feature="ffi")]
mod ffi;

use crate::core::{Function, Measurement, PrivacyRelation, Domain, SensitivityMetric};
use crate::dist::{L2Distance, AbsoluteDistance, SmoothedMaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::error::*;
use crate::samplers::SampleDiscreteGaussian;
use num::Float;
use crate::traits::{DistanceConstant, InfCast, CheckNull, TotalOrd, InfLn, InfAdd, InfSqrt};

// const ADDITIVE_GAUSS_CONST: f64 = 8. / 9. + (2. / std::f64::consts::PI).ln();
const ADDITIVE_GAUSS_CONST: f64 = 0.4373061836;

pub trait DiscreteGaussianDomain<Q>: Domain {
    type InputMetric: SensitivityMetric<Distance=Q> + Default;
    type Atom;
    fn new() -> Self;
    fn noise_function(scale: f64, bounds: Option<(Self::Atom, Self::Atom)>) -> Function<Self, Self>;
}


impl<T, Q> DiscreteGaussianDomain<Q> for AllDomain<T>
    where T: 'static + Clone + SampleDiscreteGaussian + CheckNull {
    type InputMetric = AbsoluteDistance<Q>;
    type Atom = T;

    fn new() -> Self { AllDomain::new() }
    fn noise_function(scale: f64, bounds: Option<(T, T)>) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier|
            T::sample_discrete_gaussian(arg.clone(), scale, bounds.clone()))
    }
}

impl<T, Q> DiscreteGaussianDomain<Q> for VectorDomain<AllDomain<T>>
    where T: 'static + Clone + SampleDiscreteGaussian + CheckNull {
    type InputMetric = L2Distance<Q>;
    type Atom = T;

    fn new() -> Self { VectorDomain::new_all() }
    fn noise_function(scale: f64, bounds: Option<(T, T)>) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| arg.iter()
            .map(|v| T::sample_discrete_gaussian(v.clone(), scale, bounds.clone()))
            .collect())
    }
}

pub fn make_base_discrete_gaussian<D, Q>(
    scale: Q, bounds: Option<(D::Atom, D::Atom)>
) -> Fallible<Measurement<D, D, D::InputMetric, SmoothedMaxDivergence<Q>>>
    where D: 'static + DiscreteGaussianDomain<Q>,
          D::Atom: 'static + TotalOrd + Clone + InfCast<Q>,
          Q: 'static + Float + DistanceConstant<D::Atom> + InfCast<f64> + InfAdd + InfLn + InfSqrt,
          f64: InfCast<Q> {
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
        SmoothedMaxDivergence::default(),
        PrivacyRelation::new_fallible(move |&d_in: &Q, &(eps, del): &(Q, Q)| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative")
            }
            if eps.is_sign_negative() {
                return fallible!(InvalidDistance, "epsilon must be non-negative")
            }
            if del.is_sign_negative() || del.is_zero() {
                return fallible!(InvalidDistance, "delta must be positive")
            }

            let _2 = Q::inf_cast(2.)?;
            let additive_gauss_const = Q::inf_cast(ADDITIVE_GAUSS_CONST)?;

            // min(eps, 1) * scale >= d_in * (const + sqrt(2 * ln(1/del)))
            Ok(eps.min(Q::one()).neg_inf_mul(&scale)? >=
                d_in.inf_mul(&additive_gauss_const.inf_add(
                    &_2.inf_mul(&del.recip().inf_ln()?)?)?.inf_sqrt()?)?)
        })
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_discrete_gaussian_mechanism_bounded() {
        let measurement = make_base_discrete_gaussian::<AllDomain<_>, f64>(10.0, Some((200, 210))).unwrap_test();
        let arg = 205;
        let _ret = measurement.invoke(&arg).unwrap_test();
        println!("{:?}", _ret);

        assert!(measurement.check(&1., &(0.5, 1e-6)).unwrap_test());
    }

    #[test]
    fn test_make_vector_discrete_laplace_mechanism_bounded() {
        let measurement = make_base_discrete_gaussian::<VectorDomain<_>, f64>(10.0, Some((200, 210))).unwrap_test();
        let arg = vec![1, 2, 3, 4];
        let _ret = measurement.invoke(&arg).unwrap_test();
        println!("{:?}", _ret);

        assert!(measurement.check(&1., &(0.5, 1e-6)).unwrap_test());
    }

    #[test]
    fn test_make_discrete_laplace_mechanism() {
        let measurement = make_base_discrete_gaussian::<AllDomain<_>, f64>(10.0, None).unwrap_test();
        let arg = 205;
        let _ret = measurement.invoke(&arg).unwrap_test();
        println!("{:?}", _ret);

        assert!(measurement.check(&1., &(0.5, 1e-6)).unwrap_test());
    }

    #[test]
    fn test_make_vector_discrete_laplace_mechanism() {
        let measurement = make_base_discrete_gaussian::<VectorDomain<_>, f64>(10.0, None).unwrap_test();
        let arg = vec![1, 2, 3, 4];
        let _ret = measurement.invoke(&arg).unwrap_test();
        println!("{:?}", _ret);

        assert!(measurement.check(&1., &(0.5, 1e-6)).unwrap_test());
    }
}
