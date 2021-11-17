#[cfg(feature="ffi")]
mod ffi;

use num::{Float, One, Zero};

use crate::core::{Domain, Function, Measurement, PrivacyRelation, SensitivityMetric};
use crate::dist::{AbsoluteDistance, L2Distance, SmoothedMaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::error::*;
use crate::samplers::SampleGaussian;

use crate::traits::{InfCast, CheckNull, InfMul, InfAdd, InfLn, InfSqrt};
mod analytic;
use analytic::get_analytic_gaussian_sigma;

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
          f64: InfCast<D::Atom>,
          D::Atom: 'static + Clone + SampleGaussian + Float + InfCast<f64> + CheckNull + InfMul + InfAdd + InfLn + InfSqrt {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale.clone()),
        D::Metric::default(),
        SmoothedMaxDivergence::default(),
        PrivacyRelation::new_fallible(move |&d_in: &D::Atom, &(eps, del): &(D::Atom, D::Atom)| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative")
            }
            if eps.is_sign_negative() {
                return fallible!(InvalidDistance, "epsilon must be non-negative")
            }
            if del.is_sign_negative() || del.is_zero() {
                return fallible!(InvalidDistance, "delta must be positive")
            }

            let _2 = D::Atom::inf_cast(2.)?;
            let additive_gauss_const = D::Atom::inf_cast(ADDITIVE_GAUSS_CONST)?;

            // min(eps, 1) * scale >= d_in * (const + sqrt(2 * ln(1/del)))
            Ok(eps.min(D::Atom::one()).neg_inf_mul(&scale)? >=
                d_in.inf_mul(&additive_gauss_const.inf_add(
                    &_2.inf_mul(&del.recip().inf_ln()?)?)?.inf_sqrt()?)?)
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
        PrivacyRelation::new_fallible(move |&d_in: &D::Atom, &(eps, del): &(D::Atom, D::Atom)| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative")
            }
            if eps.is_sign_negative() {
                return fallible!(InvalidDistance, "epsilon must be non-negative")
            }
            if del.is_sign_negative() || del.is_zero() {
                return fallible!(InvalidDistance, "delta must be positive")
            }

            let d_in = f64::inf_cast(d_in.clone())?;
            let eps = f64::inf_cast(eps.clone())?;
            let del = f64::inf_cast(del.clone())?;
            let scale = f64::inf_cast(scale.clone())?;

            Ok(scale >= get_analytic_gaussian_sigma(d_in, eps, del))
        }),
    ))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_gaussian_mechanism() -> Fallible<()> {
        let measurement = make_base_gaussian::<AllDomain<_>>(1.0)?;
        let arg = 0.0;
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&0.1, &(0.5, 0.00001))?);
        Ok(())
    }

    fn catastrophic_analytic_check(scale: f64, d_in: f64, d_out: (f64, f64)) -> bool {
        let (eps, del) = d_out;
        // simple shortcut to check the analytic gaussian.
        // suffers from catastrophic cancellation
        use statrs::function::erf;
        fn phi(t: f64) -> f64 {
            0.5 * (1. + erf::erf(t / 2.0_f64.sqrt()))
        }

        let prob_l_xy = phi(d_in / (2. * scale) - eps * scale / d_in);
        let prob_l_yx = phi(-d_in / (2. * scale) - eps * scale / d_in);
        del >= prob_l_xy - eps.exp() * prob_l_yx
    }

    #[test]
    fn test_make_gaussian_mechanism_analytic() -> Fallible<()> {
        let d_in = 1.;
        let d_out = (1., 1e-5);
        let scale = 3.730632;

        let measurement = make_base_analytic_gaussian::<AllDomain<_>>(scale)?;
        let arg = 0.0;
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&d_in, &d_out)?);
        // use the simpler version of the check that suffers from catastrophic cancellation,
        // to check the more complicated algorithm for finding the analytic gaussian scale
        assert!(catastrophic_analytic_check(scale, d_in, d_out));
        assert!(!catastrophic_analytic_check(scale - 1e-6, d_in, d_out));

        Ok(())
    }

    #[test]
    fn test_make_gaussian_vec_mechanism() -> Fallible<()> {
        let measurement = make_base_gaussian::<VectorDomain<_>>(1.0)?;
        let arg = vec![0.0, 1.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&0.1, &(0.5, 0.00001))?);
        Ok(())
    }
}
