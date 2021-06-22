use num::Float;

use crate::core::{Function, Measurement, PrivacyRelation, Domain};
use crate::dist::{L2Sensitivity, SmoothedMaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::error::*;
use crate::samplers::SampleGaussian;

// const ADDITIVE_GAUSS_CONST: f64 = 8. / 9. + (2. / PI).ln();
const ADDITIVE_GAUSS_CONST: f64 = 0.4373061836;

fn make_gaussian_privacy_relation<T: 'static + Clone + SampleGaussian + Float>(scale: T) -> PrivacyRelation<L2Sensitivity<T>, SmoothedMaxDivergence<T>> {
    PrivacyRelation::new_fallible(move |&d_in: &T, &(eps, del): &(T, T)| {
        let _2 = num_cast!(2.; T)?;
        let additive_gauss_const = num_cast!(ADDITIVE_GAUSS_CONST; T)?;

        if d_in.is_sign_negative() {
            return fallible!(InvalidDistance, "gaussian mechanism: input sensitivity must be non-negative")
        }
        if eps.is_sign_negative() || eps.is_zero() {
            return fallible!(InvalidDistance, "gaussian mechanism: epsilon must be positive")
        }
        if del.is_sign_negative() || del.is_zero() {
            return fallible!(InvalidDistance, "gaussian mechanism: delta must be positive")
        }

        // TODO: should we error if epsilon > 1., or just waste the budget?
        Ok(eps.min(T::one()) >= (d_in / scale) * (additive_gauss_const + _2 * del.recip().ln()).sqrt())
    })
}


pub trait GaussianDomain: Domain {
    type Atom;
    fn new() -> Self;
    fn noise_function(scale: Self::Atom) -> Function<Self, Self>;
}


impl<T> GaussianDomain for AllDomain<T>
    where T: 'static + SampleGaussian + Float {
    type Atom = Self::Carrier;

    fn new() -> Self { AllDomain::new() }
    fn noise_function(scale: Self::Carrier) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| Self::Carrier::sample_gaussian(*arg, scale, false))
    }
}

impl<T> GaussianDomain for VectorDomain<AllDomain<T>>
    where T: 'static + SampleGaussian + Float {
    type Atom = T;

    fn new() -> Self { VectorDomain::new_all() }
    fn noise_function(scale: T) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| arg.iter()
            .map(|v| T::sample_gaussian(*v, scale, false))
            .collect())
    }
}


pub fn make_base_gaussian<DA>(scale: DA::Atom) -> Fallible<Measurement<DA, DA, L2Sensitivity<DA::Atom>, SmoothedMaxDivergence<DA::Atom>>>
    where DA: GaussianDomain,
          DA::Atom: 'static + Clone + SampleGaussian + Float {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        DA::new(),
        DA::new(),
        DA::noise_function(scale.clone()),
        L2Sensitivity::default(),
        SmoothedMaxDivergence::default(),
        make_gaussian_privacy_relation(scale),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_gaussian_mechanism() -> Fallible<()> {
        let measurement = make_base_gaussian::<AllDomain<_>>(1.0)?;
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg)?;

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001))?);
        Ok(())
    }

    #[test]
    fn test_make_gaussian_vec_mechanism() -> Fallible<()> {
        let measurement = make_base_gaussian::<VectorDomain<_>>(1.0)?;
        let arg = vec![0.0, 1.0];
        let _ret = measurement.function.eval(&arg)?;

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001))?);
        Ok(())
    }
}
