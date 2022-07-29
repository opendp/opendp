use std::convert::TryFrom;

use az::{SaturatingAs, SaturatingCast};
use num::traits::Pow;
use rug::{Integer, Rational};

use crate::{
    core::{Measurement, PrivacyMap, SensitivityMetric},
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    measures::ZeroConcentratedDivergence,
    metrics::{AbsoluteDistance, L2Distance},
    traits::{samplers::sample_discrete_gaussian, CheckNull},
};

#[cfg(feature="ffi")]
mod ffi;

use super::MappableDomain;

pub trait DiscreteGaussianDomain<Q>: MappableDomain + Default {
    type InputMetric: SensitivityMetric<Distance = Q> + Default;
}
impl<T: CheckNull, Q> DiscreteGaussianDomain<Q> for AllDomain<T> {
    type InputMetric = AbsoluteDistance<Q>;
}
impl<T: CheckNull, Q> DiscreteGaussianDomain<Q> for VectorDomain<AllDomain<T>> {
    type InputMetric = L2Distance<Q>;
}

pub fn make_base_discrete_gaussian<D, Q>(
    scale: Q,
) -> Fallible<Measurement<D, D, D::InputMetric, ZeroConcentratedDivergence<Q>>>
where
    D: DiscreteGaussianDomain<Q>,
    D::Atom: crate::traits::Integer,
    Q: crate::traits::Float,
    Rational: TryFrom<Q>,
    Integer: From<D::Atom> + SaturatingCast<D::Atom>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    let _2 = Q::exact_int_cast(2)?;
    let scale_rational =
        Rational::try_from(scale).map_err(|_| err!(MakeMeasurement, "scale must be finite"))?;

    Ok(Measurement::new(
        D::default(),
        D::default(),
        D::new_map_function(move |arg: &D::Atom| {
            // exact conversion to bignum int
            let arg = Integer::from(arg.clone());
            // exact sampling of noise
            let noise = sample_discrete_gaussian(scale_rational.clone())?;
            // exact addition, and then postprocess by casting to D::Atom
            //     clamp to the data type's bounds if out of range
            Ok((arg + noise).saturating_as())
        }),
        D::InputMetric::default(),
        ZeroConcentratedDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &Q| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if scale.is_zero() {
                return Ok(Q::infinity());
            }

            // (d_in / scale)^2 / 2
            (d_in.inf_div(&scale)?).inf_pow(&_2)?.inf_div(&_2)
        }),
    ))
}



pub fn make_base_discrete_gaussian_rug<D>(
    scale: Rational,
) -> Fallible<Measurement<D, D, D::InputMetric, ZeroConcentratedDivergence<Rational>>>
where
    D: DiscreteGaussianDomain<Rational, Atom = Integer>,
{
    if scale <= 0 {
        return fallible!(MakeMeasurement, "scale must be positive");
    }

    Ok(Measurement::new(
        D::default(),
        D::default(),
        D::new_map_function(enclose!(scale, move |arg: &Integer| {
            sample_discrete_gaussian(scale.clone()).map(|n| arg + n)
        })),
        D::InputMetric::default(),
        ZeroConcentratedDivergence::default(),
        PrivacyMap::new(move |d_in: &Rational| (d_in.clone() / &scale).pow(2) / 2),
    ))
}