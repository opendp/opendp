use num::{Float, One, Zero};

use crate::core::{Domain, Function, Measurement, PrivacyRelation, SensitivityMetric};
use crate::dist::{AbsoluteDistance, L2Distance, SmoothedMaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::error::*;
use crate::samplers::SampleGaussian;
use crate::traits::{CheckNull, InfCast};

pub use self::analytic::*;

pub mod analytic;

// const ADDITIVE_GAUSS_CONST: f64 = 8. / 9. + (2. / std::f64::consts::PI).ln();
pub const ADDITIVE_GAUSS_CONST: f64 = 0.4373061836;

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
        Function::new_fallible(move |arg: &Self::Carrier| Self::Carrier::sample_gaussian(*arg, scale, false))
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


pub fn make_base_gaussian<D>(scale: D::Atom, analytic: bool) -> Fallible<Measurement<D, D, D::Metric, SmoothedMaxDivergence<D::Atom>>>
    where D: GaussianDomain,
          f64: InfCast<D::Atom>,
          D::Atom: 'static + Clone + SampleGaussian + Float + InfCast<f64> + CheckNull + Zero + One {
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
                return fallible!(InvalidDistance, "input sensitivity must be non-negative")
            }
            if eps.is_sign_negative() {
                return fallible!(InvalidDistance, "epsilon must not be negative")
            }
            if del.is_sign_negative() || del.is_zero() {
                return fallible!(InvalidDistance, "delta must be positive")
            }

            Ok(if analytic {
                let d_in = f64::inf_cast(d_in.clone())?;
                let eps = f64::inf_cast(eps.clone())?;
                let del = f64::inf_cast(del.clone())?;
                let scale = f64::inf_cast(scale.clone())?;
                if get_analytic_gaussian_sigma(eps, del, d_in) > scale {
                    return Ok(false)
                }
                eps * scale >= d_in * (ADDITIVE_GAUSS_CONST + 2. * del.recip().ln()).sqrt()
            } else {
                let _2 = D::Atom::inf_cast(2.)?;
                let additive_gauss_const = D::Atom::inf_cast(ADDITIVE_GAUSS_CONST)?;

                eps.min(D::Atom::one()) * scale >=
                    d_in * (additive_gauss_const + _2 * del.recip().ln()).sqrt()
            })
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_gaussian_mechanism() -> Fallible<()> {
        let measurement = make_base_gaussian::<AllDomain<_>>(1.0, false)?;
        let arg = 0.0;
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&0.1, &(0.5, 0.00001))?);
        Ok(())
    }

    #[test]
    fn test_make_gaussian_mechanism_analytic() -> Fallible<()> {
        let measurement = make_base_gaussian::<AllDomain<_>>(1.0, true)?;
        let arg = 0.0;
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&0.1, &(0.5, 0.00001))?);
        Ok(())
    }

    #[test]
    fn test_make_gaussian_vec_mechanism() -> Fallible<()> {
        let measurement = make_base_gaussian::<VectorDomain<_>>(1.0, false)?;
        let arg = vec![0.0, 1.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&0.1, &(0.5, 0.00001))?);
        Ok(())
    }
}
