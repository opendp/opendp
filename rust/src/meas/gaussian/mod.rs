#[cfg(feature="ffi")]
mod ffi;

use num::{Float, One, Zero};

use crate::core::{Domain, Function, Measurement, PrivacyMap, SensitivityMetric};
use crate::core::{AbsoluteDistance, L2Distance, SmoothedMaxDivergence, SMDCurve};
use crate::core::{AllDomain, VectorDomain};
use crate::error::*;
use crate::traits::samplers::SampleGaussian;

use crate::traits::{InfCast, CheckNull, InfMul, InfAdd, InfLn, InfSqrt, InfDiv, InfSub, InfExp};


mod analytic;

use self::analytic::get_analytic_gaussian_epsilon;

// const ADDITIVE_GAUSS_CONST: f64 = 8. / 9. + (2. / std::f64::consts::PI).ln();
const ADDITIVE_GAUSS_CONST: f64 = 0.4373061836;

pub trait GaussianDomain: Domain {
    type Metric: SensitivityMetric<Distance=Self::Atom> + Default;
    type Atom;
    fn new() -> Self;
    fn noise_function(scale: Self::Atom) -> Function<Self, Self>;
}


impl<T> GaussianDomain for AllDomain<T>
    where T: 'static + SampleGaussian + Float + CheckNull {
    type Metric = AbsoluteDistance<T>;
    type Atom = T;

    fn new() -> Self { AllDomain::new() }
    fn noise_function(scale: Self::Carrier) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier|
            Self::Carrier::sample_gaussian(*arg, scale, false))
    }
}

impl<T> GaussianDomain for VectorDomain<AllDomain<T>>
    where T: 'static + SampleGaussian + Float + CheckNull {
    type Metric = L2Distance<T>;
    type Atom = T;

    fn new() -> Self { VectorDomain::new_all() }
    fn noise_function(scale: T) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| arg.iter()
            .map(|v| T::sample_gaussian(*v, scale, false))
            .collect())
    }
}


pub fn make_base_gaussian<D>(
    scale: D::Atom
) -> Fallible<Measurement<D, D, D::Metric, SmoothedMaxDivergence<D::Atom>>>
    where D: GaussianDomain,
          D::Atom: 'static + Clone + SampleGaussian + Float + InfCast<f64> + InfSub + InfDiv + CheckNull + InfMul + InfAdd + InfLn + InfSqrt + InfExp + Zero {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale.clone()),
        D::Metric::default(),
        SmoothedMaxDivergence::default(),
        PrivacyMap::new_fallible(move |&d_in: &D::Atom| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative")
            }

            let _2 = D::Atom::inf_cast(2.)?;
            let additive_gauss_const = D::Atom::inf_cast(ADDITIVE_GAUSS_CONST)?;

            Ok(SMDCurve::new(
                move |del: &D::Atom| {
                    if !del.is_sign_positive() {
                        return fallible!(FailedRelation, "delta must be positive")
                    }

                    if scale.is_zero() {
                        return Ok(D::Atom::infinity())
                    }

                    let eps = d_in.inf_mul(&additive_gauss_const.inf_add(
                        &_2.inf_mul(&del.recip().inf_ln()?)?)?.inf_sqrt()?)?.inf_div(&scale)?;
                    
                    if eps > D::Atom::one() {
                        return fallible!(RelationDebug, "The gaussian mechanism has an epsilon of at most one. Epsilon is greater than one at the given delta.")
                    }
                    Ok(eps)
                }
            ))
        }),
    ))
}

pub fn make_base_analytic_gaussian<D>(
    scale: D::Atom
) -> Fallible<Measurement<D, D, D::Metric, SmoothedMaxDivergence<D::Atom>>>
    where D: GaussianDomain,
          f64: InfCast<D::Atom>,
          D::Atom: 'static + Clone + SampleGaussian + Float + InfCast<f64> + CheckNull {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale.clone()),
        D::Metric::default(),
        SmoothedMaxDivergence::default(),
        PrivacyMap::new_fallible(move |&d_in: &D::Atom| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative")
            }

            let d_in = f64::inf_cast(d_in.clone())?;
            let scale = f64::inf_cast(scale.clone())?;

            Ok(SMDCurve::new(
                move |del: &D::Atom| {
                    let del = f64::inf_cast(del.clone())?;
                    if del <= 0. {
                        return fallible!(InvalidDistance, "delta must be positive")
                    }
                    D::Atom::inf_cast(get_analytic_gaussian_epsilon(d_in, scale, del))
                }
            ))
        }),
    ))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_gaussian_mechanism() -> Fallible<()> {
        let measurement = make_base_gaussian::<AllDomain<_>>(1.0f64)?;
        let arg = 0.0;
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.map(&0.1)?.epsilon(&0.00001)? <= 0.5);
        Ok(())
    }

    #[test]
    fn test_make_gaussian_vec_mechanism() -> Fallible<()> {
        let measurement = make_base_gaussian::<VectorDomain<_>>(1.0f64)?;
        let arg = vec![0.0, 1.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.map(&0.1)?.epsilon(&0.00001)? <= 0.5);
        Ok(())
    }
}
