#[cfg(feature = "polars")]
use crate::measurements::expr_noisy_max::TopKDistribution;
use crate::{
    core::{Function, Measure, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::{exponential::noisy_top_k_exponential, gumbel::noisy_top_k_gumbel},
    measures::{MaxDivergence, ZeroConcentratedDivergence},
    metrics::LInfDistance,
    traits::{CastInternalRational, InfCast, InfDiv, InfMul, InfPowI, Number},
};
use dashu::{float::FBig, ibig, rational::RBig};
use num::Zero;
use opendp_derive::bootstrap;

#[cfg(feature = "ffi")]
mod ffi;
#[cfg(test)]
mod test;

pub(crate) mod exponential;
pub(crate) mod gumbel;

#[bootstrap(
    features("contrib"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        negate(default = false),
    ),
    generics(MO(suppress), TIA(suppress))
)]
/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable VectorDomain.
/// * `input_metric` - Metric on the input domain. Must be LInfDistance
/// * `output_measure` - One of `MaxDivergence` or `ZeroConcentratedDivergence`
/// * `k` - Number of indices to select.
/// * `scale` - Scale for the noise distribution.
/// * `negate` - Set to true to return bottom k
///
/// # Generics
/// * `MO` - Output Measure.
/// * `TIA` - Atom Input Type. Type of each element in the score vector.
pub fn make_noisy_top_k<MO: TopKMeasure, TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    output_measure: MO,
    k: usize,
    scale: f64,
    negate: bool,
) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, LInfDistance<TIA>, MO, Vec<usize>>>
where
    TIA: Number + CastInternalRational,
    f64: InfCast<TIA> + InfCast<usize>,
    FBig: TryFrom<TIA>,
{
    if input_domain.element_domain.nan() {
        return fallible!(MakeMeasurement, "input_domain elements must be non-nan");
    }

    if k == 0 {
        return fallible!(MakeMeasurement, "k ({k}) must be positive");
    }

    if let Some(size) = input_domain.size {
        if k > size {
            return fallible!(
                MakeMeasurement,
                "k ({k}) must not exceed the number of candidates ({size})"
            );
        }
    }

    if !scale.is_finite() || scale.is_sign_negative() {
        return fallible!(
            MakeMeasurement,
            "scale ({scale}) must not be finite and non-negative"
        );
    }

    Measurement::new(
        input_domain,
        input_metric.clone(),
        output_measure,
        Function::new_fallible(move |x: &Vec<TIA>| MO::noisy_top_k(x, scale, k, negate)),
        PrivacyMap::new_fallible(move |d_in: &TIA| {
            // convert L_\infty distance to range distance
            let d_in = input_metric.range_distance(d_in.clone())?;
            let d_in = f64::inf_cast(d_in)?;

            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity ({d_in}) must be non-negative");
            }

            if scale.is_zero() {
                return Ok(f64::INFINITY);
            }

            MO::privacy_map(d_in, scale)?.inf_mul(&f64::inf_cast(k)?)
        }),
    )
}

pub trait TopKMeasure: Measure<Distance = f64> + 'static {
    #[cfg(feature = "polars")]
    const DISTRIBUTION: TopKDistribution;

    fn noisy_top_k<TIA>(x: &Vec<TIA>, scale: f64, k: usize, negate: bool) -> Fallible<Vec<usize>>
    where
        TIA: Number + CastInternalRational,
        f64: InfCast<TIA> + InfCast<usize>,
        FBig: TryFrom<TIA>;
    fn privacy_map(d_in: f64, scale: f64) -> Fallible<f64>;
}

impl TopKMeasure for MaxDivergence {
    #[cfg(feature = "polars")]
    const DISTRIBUTION: TopKDistribution = TopKDistribution::Exponential;

    fn noisy_top_k<TIA>(x: &Vec<TIA>, scale: f64, k: usize, negate: bool) -> Fallible<Vec<usize>>
    where
        TIA: Number + CastInternalRational,
        f64: InfCast<TIA> + InfCast<usize>,
        FBig: TryFrom<TIA>,
    {
        noisy_top_k_exponential(x, RBig::try_from(scale)?, k, negate)
    }

    fn privacy_map(d_in: f64, scale: f64) -> Fallible<f64> {
        // d_in / scale
        d_in.inf_div(&scale)
    }
}

impl TopKMeasure for ZeroConcentratedDivergence {
    #[cfg(feature = "polars")]
    const DISTRIBUTION: TopKDistribution = TopKDistribution::Gumbel;

    fn noisy_top_k<TIA>(x: &Vec<TIA>, scale: f64, k: usize, negate: bool) -> Fallible<Vec<usize>>
    where
        TIA: Number,
        f64: InfCast<TIA> + InfCast<usize>,
        FBig: TryFrom<TIA> + TryFrom<f64>,
    {
        noisy_top_k_gumbel(x, FBig::try_from(scale)?, k, negate)
    }

    fn privacy_map(d_in: f64, scale: f64) -> Fallible<f64> {
        // (d_in / scale)^2 / 8
        d_in.inf_div(&scale)?.inf_powi(ibig!(2))?.inf_div(&8.0)
    }
}
