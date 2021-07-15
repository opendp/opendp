use num::{Float, One, Zero};

use crate::core::{Domain, Function, Measure, Measurement, Metric, PrivacyRelation, SensitivityMetric};
use crate::dist::{AbsoluteDistance, FSmoothedMaxDivergence, L2Distance, SmoothedMaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::error::*;
use crate::samplers::{CastInternalReal, SampleGaussian};

// const ADDITIVE_GAUSS_CONST: f64 = 8. / 9. + (2. / PI).ln();
const ADDITIVE_GAUSS_CONST: f64 = 0.4373061836;

pub trait GaussianDomain: Domain {
    type Metric: SensitivityMetric<Distance=Self::Atom> + Default;
    type Atom;
    fn new() -> Self;
    fn noise_function(scale: Self::Atom) -> Function<Self, Self>;
}


impl<T> GaussianDomain for AllDomain<T>
    where T: 'static + SampleGaussian + Float {
    type Metric = AbsoluteDistance<T>;
    type Atom = T;

    fn new() -> Self { AllDomain::new() }
    fn noise_function(scale: Self::Carrier) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| Self::Carrier::sample_gaussian(*arg, scale, false))
    }
}

impl<T> GaussianDomain for VectorDomain<AllDomain<T>>
    where T: 'static + SampleGaussian + Float {
    type Metric = L2Distance<T>;
    type Atom = T;

    fn new() -> Self { VectorDomain::new_all() }
    fn noise_function(scale: T) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| arg.iter()
            .map(|v| T::sample_gaussian(*v, scale, false))
            .collect())
    }
}

pub trait GaussianPrivacyRelation<MI: Metric>: Measure {
    fn privacy_relation(scale: MI::Distance) -> PrivacyRelation<MI, Self>;
}

impl<MI: Metric> GaussianPrivacyRelation<MI> for SmoothedMaxDivergence<MI::Distance>
    where MI::Distance: 'static + Clone + SampleGaussian + Float + One,
          MI: SensitivityMetric {
    fn privacy_relation(scale: MI::Distance) -> PrivacyRelation<MI, Self> {
        PrivacyRelation::new_fallible(move |&d_in: &MI::Distance, &(eps, del): &(MI::Distance, MI::Distance)| {
            let _2 = num_cast!(2.; MI::Distance)?;
            let additive_gauss_const = num_cast!(ADDITIVE_GAUSS_CONST; MI::Distance)?;

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
            Ok(eps.min(MI::Distance::one()) >= (d_in / scale) * (additive_gauss_const + _2 * del.recip().ln()).sqrt())
        })
    }
}

#[cfg(feature="use-mpfr")]
impl<MI: Metric> GaussianPrivacyRelation<MI> for FSmoothedMaxDivergence
    where MI::Distance: 'static + Clone + CastInternalReal {
    fn privacy_relation(scale: MI::Distance) -> PrivacyRelation<MI, Self> {
        PrivacyRelation::new_fallible(move |d_in: &MI::Distance, d_out: &Vec<(rug::Float, rug::Float)>| {
            use rug::float::Round;
            let d_in: rug::Float = d_in.clone().into_internal();
            let mut scale: rug::Float = scale.clone().into_internal();
            println!("input distance {:?}", d_in);
            println!("scale {:?}", scale);

            scale.ln_round(Round::Up);
            let _ln_scale_back_in_native_type = MI::Distance::from_internal(scale);
            unimplemented!()
        })
    }
}

pub fn make_base_gaussian<D, MO>(scale: D::Atom) -> Fallible<Measurement<D, D, D::Metric, MO>>
    where D: GaussianDomain,
          D::Atom: 'static + Clone + SampleGaussian + Float,
          MO: Measure + GaussianPrivacyRelation<D::Metric> {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale.clone()),
        D::Metric::default(),
        MO::default(),
        MO::privacy_relation(scale),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_gaussian_mechanism() -> Fallible<()> {
        let measurement = make_base_gaussian::<AllDomain<_>, SmoothedMaxDivergence<_>>(1.0)?;
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg)?;

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001))?);
        Ok(())
    }

    #[test]
    fn test_make_gaussian_vec_mechanism() -> Fallible<()> {
        let measurement = make_base_gaussian::<VectorDomain<_>, SmoothedMaxDivergence<_>>(1.0)?;
        let arg = vec![0.0, 1.0];
        let _ret = measurement.function.eval(&arg)?;

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001))?);
        Ok(())
    }
}
