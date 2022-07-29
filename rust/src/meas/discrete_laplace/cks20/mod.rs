use std::convert::TryFrom;

use az::{SaturatingAs, SaturatingCast};
use rug::{Complete, Integer, Rational};

use crate::{
    core::{Measurement, PrivacyMap},
    error::Fallible,
    measures::MaxDivergence,
    traits::{InfCast, samplers::sample_discrete_laplace},
};

use super::DiscreteLaplaceDomain;

pub fn make_base_discrete_laplace_cks20_rug<D>(
    scale: Rational,
) -> Fallible<Measurement<D, D, D::InputMetric, MaxDivergence<Rational>>>
where
    D: DiscreteLaplaceDomain<Atom = Integer>,
{
    if scale <= 0 {
        return fallible!(MakeMeasurement, "scale must be positive");
    }

    Ok(Measurement::new(
        D::default(),
        D::default(),
        D::new_map_function(enclose!(scale, move |arg: &Integer| {
            sample_discrete_laplace(scale.clone()).map(|n| arg + n)
        })),
        D::InputMetric::default(),
        MaxDivergence::default(),
        PrivacyMap::new(move |arg: &Integer| (arg / &scale).complete()),
    ))
}

pub fn make_base_discrete_laplace_cks20<D, QO>(
    scale: QO,
) -> Fallible<Measurement<D, D, D::InputMetric, MaxDivergence<QO>>>
where
    D: DiscreteLaplaceDomain,
    D::Atom: crate::traits::Integer,
    QO: crate::traits::Float + InfCast<D::Atom>,
    Rational: TryFrom<QO>,
    Integer: From<D::Atom> + SaturatingCast<D::Atom>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    let scale_rational =
        Rational::try_from(scale).map_err(|_| err!(MakeMeasurement, "scale must be finite"))?;

    Ok(Measurement::new(
        D::default(),
        D::default(),
        D::new_map_function(move |arg: &D::Atom| {
            let arg = Integer::from(arg.clone());
            let noise = sample_discrete_laplace(scale_rational.clone())?;
            Ok((arg + noise).saturating_as())
        }),
        D::InputMetric::default(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &D::Atom| {
            let d_in = QO::inf_cast(d_in.clone())?;
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if scale.is_zero() {
                return Ok(QO::infinity());
            }
            // d_in / scale
            d_in.inf_div(&scale)
        }),
    ))
}
