#[cfg(feature="ffi")]
mod ffi;

use std::fmt::Debug;
use num::{Float, One, Zero};

use crate::core::{Domain, Function, Measure, Measurement, Metric, PrivacyRelation, SensitivityMetric};
use crate::dist::{AbsoluteDistance, GaussianTradeOff, L2Distance, RenyiDivergence, SmoothedMaxDivergence, UnionRenyiDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::error::*;
use crate::samplers::SampleGaussian;

use crate::traits::{InfCast, CheckNull, InfMul, InfAdd, InfLn, InfSqrt, InfDiv};
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

pub trait GaussianMeasure<MI>: Measure
    where MI: Metric {
    fn new_privacy_relation(&self, scale: MI::Distance) -> PrivacyRelation<MI, Self>;
}

// Tie the input distance type to the atomic output distance type,
//  (atomic output distance type is epsilon or delta)
impl<MI: Metric> GaussianMeasure<MI> for SmoothedMaxDivergence<MI::Distance>
    where MI::Distance: 'static + Clone + InfCast<f64> + One + Float + InfAdd + InfMul + InfLn + InfSqrt {

    fn new_privacy_relation(&self, scale: MI::Distance) -> PrivacyRelation<MI, Self> {
        PrivacyRelation::new_fallible(move |&d_in: &MI::Distance, &(eps, del): &(MI::Distance, MI::Distance)| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative")
            }
            if eps.is_sign_negative() {
                return fallible!(InvalidDistance, "epsilon must be non-negative")
            }
            if del.is_sign_negative() || del.is_zero() {
                return fallible!(InvalidDistance, "delta must be positive")
            }

            let _2 = MI::Distance::inf_cast(2.)?;
            let additive_gauss_const = MI::Distance::inf_cast(ADDITIVE_GAUSS_CONST)?;

            // min(eps, 1) * scale >= d_in * (const + sqrt(2 * ln(1/del)))
            Ok(eps.min(MI::Distance::one()).neg_inf_mul(&scale)? >=
                d_in.inf_mul(&additive_gauss_const.inf_add(
                    &_2.inf_mul(&del.recip().inf_ln()?)?)?.inf_sqrt()?)?)
        })
    }
}


impl<MI: Metric> GaussianMeasure<MI> for GaussianTradeOff<MI::Distance>
    // TODO: reimplement conservative rounding (Inf* traits)
    where MI::Distance: 'static + Clone + Float {
    fn new_privacy_relation(&self, scale: MI::Distance) -> PrivacyRelation<MI, Self> {
        PrivacyRelation::new_fallible(move |&sens: &MI::Distance, &mu: &MI::Distance| {
            if sens.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative")
            }
            if mu.is_sign_negative() {
                return fallible!(InvalidDistance, "mu must be non-negative")
            }
            // https://arxiv.org/pdf/1905.02383.pdf#page=11 Theorem 2.7
            // scale = sens^2 / mu^2
            Ok(scale * mu.powi(2) >= sens.powi(2))
        })
    }
}


impl<MI: Metric> GaussianMeasure<MI> for RenyiDivergence<MI::Distance>
// TODO: reimplement conservative rounding (Inf* traits)
    where MI::Distance: 'static + Clone + Float + InfCast<i32> + Debug {
    fn new_privacy_relation(&self, scale: MI::Distance) -> PrivacyRelation<MI, Self> {
        let alpha = self.alpha;
        PrivacyRelation::new_fallible(move |&sens: &MI::Distance, &rho: &MI::Distance| {
            if sens.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative")
            }
            if rho.is_sign_negative() {
                return fallible!(InvalidDistance, "rho must be non-negative")
            }
            let _2 = MI::Distance::inf_cast(2)?;
            let alpha = MI::Distance::inf_cast(alpha)?;
            // https://arxiv.org/pdf/1702.07476.pdf#page=8 Corollary 3
            // rho = alpha * sens^2 / (2 * scale^2)
            println!("{:?}", rho * _2 * scale.powi(2));
            println!("{:?}", alpha * sens.powi(2));
            Ok(rho * _2 * scale.powi(2) >= alpha * sens.powi(2))
        })
    }
}

impl<MI: Metric> GaussianMeasure<MI> for UnionRenyiDivergence<MI::Distance>
    // TODO: reimplement conservative rounding (Inf* traits)
    where MI::Distance: 'static + Clone + Float + InfCast<i64> + Debug + InfDiv + InfMul {
    fn new_privacy_relation(&self, scale: MI::Distance) -> PrivacyRelation<MI, Self> {
        PrivacyRelation::new_fallible(move |d_in: &MI::Distance, d_out: &MI::Distance| {
            // https://arxiv.org/pdf/1605.02065.pdf#page=7 Proposition 1.6
            // rho = sens^2 / (2 scale^2)
            Ok(d_out.inf_mul(&scale)?.inf_mul(&scale)? >=
                   d_in.neg_inf_mul(&d_in)?.neg_inf_div(&MI::Distance::inf_cast(2)?)?)
        })
    }
}

pub fn make_base_gaussian<D, MO>(
    scale: D::Atom, output_measure: MO
) -> Fallible<Measurement<D, D, D::Metric, MO>>
    where D: GaussianDomain,
          f64: InfCast<D::Atom>,
          D::Atom: 'static + Clone + SampleGaussian + Float + InfCast<f64> + CheckNull + InfMul + InfAdd + InfLn + InfSqrt,
          MO: GaussianMeasure<D::Metric> {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    let relation = output_measure.new_privacy_relation(scale);
    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale.clone()),
        D::Metric::default(),
        // TODO: default will not work when implementing Renyi divergence, as alpha must be known
        output_measure,
        relation,
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
    fn test_make_gaussian_mechanism_smoothed_max_div() -> Fallible<()> {
        let measurement = make_base_gaussian::<AllDomain<_>, _>(1.0, SmoothedMaxDivergence::default())?;
        let arg = 0.0;
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&0.1, &(0.5, 0.00001))?);
        Ok(())
    }

    #[test]
    fn test_make_gaussian_mechanism_gaussian_tradeoff() -> Fallible<()> {
        let measurement = make_base_gaussian::<AllDomain<_>, _>(1.0, GaussianTradeOff::default())?;

        let arg = 0.0;
        let _ret = measurement.invoke(&arg)?;

        // TODO: more robust testing of the Gaussian Tradeoff relation
        assert!(measurement.check(&0.1, &0.1.sqrt())?);
        Ok(())
    }

    #[test]
    fn test_make_gaussian_mechanism_renyi_divergence() -> Fallible<()> {
        let measurement = make_base_gaussian::<AllDomain<_>, _>(
            1.0, RenyiDivergence::new(2))?;

        let arg = 0.0;
        let _ret = measurement.invoke(&arg)?;

        // TODO: more robust testing of the Gaussian Tradeoff relation
        assert!(measurement.check(&0.1, &0.02)?);
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
        let measurement = make_base_gaussian::<VectorDomain<_>, _>(1.0, SmoothedMaxDivergence::default())?;
        let arg = vec![0.0, 1.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&0.1, &(0.5, 0.00001))?);
        Ok(())
    }
}
