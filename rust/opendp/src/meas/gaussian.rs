use num::Float;

use crate::core::{Function, Measurement, PrivacyRelation};
use crate::dist::{L2Sensitivity, SmoothedMaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::error::*;
use crate::samplers::SampleGaussian;

// const ADDITIVE_GAUSS_CONST: f64 = 8. / 9. + (2. / PI).ln();
pub(in crate::meas) const ADDITIVE_GAUSS_CONST: f64 = 0.4373061836;

fn make_gaussian_privacy_relation<T: 'static + Clone + Float>(scale: T) -> PrivacyRelation<L2Sensitivity<T>, SmoothedMaxDivergence<T>> {
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

pub fn make_base_gaussian<T>(scale: T) -> Fallible<Measurement<AllDomain<T>, AllDomain<T>, L2Sensitivity<T>, SmoothedMaxDivergence<T>>>
    where T: 'static + Clone + SampleGaussian + Float {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new_fallible(move |arg: &T| -> Fallible<T> {
            T::sample_gaussian(*arg, scale, false)
        }),
        L2Sensitivity::default(),
        SmoothedMaxDivergence::default(),
        make_gaussian_privacy_relation(scale),
    ))
}


pub fn make_base_gaussian_vec<T>(
    scale: T
) -> Fallible<Measurement<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L2Sensitivity<T>, SmoothedMaxDivergence<T>>>
    where T: 'static + Clone + SampleGaussian + Float {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<T>| -> Fallible<Vec<T>> {
            arg.iter()
                .map(|v| T::sample_gaussian(*v, scale, false))
                .collect()
        }),
        L2Sensitivity::default(),
        SmoothedMaxDivergence::default(),
        make_gaussian_privacy_relation(scale),
    ))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_gaussian_mechanism() {
        let measurement = make_base_gaussian(1.0).unwrap_test();
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg).unwrap_test();

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001)).unwrap_test());
    }

    #[test]
    fn test_make_gaussian_vec_mechanism() {
        let measurement = make_base_gaussian_vec(1.0).unwrap_test();
        let arg = vec![0.0, 1.0];
        let _ret = measurement.function.eval(&arg).unwrap_test();

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001)).unwrap_test());
    }
}
