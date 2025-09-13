#[cfg(feature = "polars")]
use crate::measurements::expr_noisy_max::TopKDistribution;
use crate::{
    core::{Function, Measure, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::{exponential::exponential_top_k, gumbel::gumbel_top_k},
    measures::{MaxDivergence, ZeroConcentratedDivergence},
    metrics::LInfDistance,
    traits::{CastInternalRational, InfCast, InfDiv, InfMul, InfPowI, Number},
};
use dashu::{float::FBig, ibig};
use num::Zero;
use opendp_derive::{bootstrap, proven};

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

    let monotonic = input_metric.monotonic;

    Measurement::new(
        input_domain,
        input_metric,
        output_measure,
        Function::new_fallible(move |x: &Vec<TIA>| MO::noisy_top_k(x, scale, k, negate)),
        PrivacyMap::new_fallible(move |d_in: &TIA| {
            // Translate a distance bound `d_in` wrt the $L_\infty$ metric to a distance bound wrt the range metric.
            //
            // ```math
            // d_{\mathrm{Range}}(x, x') = max_{ij} |(x_i - x'_i) - (x_j - x'_j)|
            // ```
            let d_in = if monotonic {
                d_in.clone()
            } else {
                d_in.inf_add(&d_in)?
            };

            let d_in = f64::inf_cast(d_in)?;

            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity ({d_in}) must be non-negative");
            }

            if d_in.is_zero() {
                return Ok(0.0);
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

    /// # Proof Definition
    /// Returns the index of the max element $z_i$,
    /// where each $z_i \sim \mathrm{DISTRIBUTION}(\mathrm{shift}=y_i, \mathrm{scale}=\texttt{scale})$,
    /// and each $y_i = -x_i$ if \texttt{negate}, else $y_i = x_i$,
    /// $k$ times with removal.
    fn noisy_top_k<TIA>(x: &Vec<TIA>, scale: f64, k: usize, negate: bool) -> Fallible<Vec<usize>>
    where
        TIA: Number + CastInternalRational,
        f64: InfCast<TIA> + InfCast<usize>,
        FBig: TryFrom<TIA>;

    /// Define
    /// ```math
    /// d_{\mathrm{Range}}(x, x') = max_{ij} |(x_i - x'_i) - (x_j - x'_j)|.
    /// ```
    ///
    /// # Proof Definition
    /// For any $x, x'$ where $d_\mathrm{in} \ge d_\mathrm{Range}(x, x')$,
    /// return $d_\mathrm{out} \ge D_\mathrm{self}(f(x), f(x'))$,
    /// where $f(x) = \mathrm{noisy\_top\_k}(x=x, k=1, \mathrm{scale}=\mathrm{scale})$.
    fn privacy_map(d_in: f64, scale: f64) -> Fallible<f64>;
}

#[proven(proof_path = "measurements/noisy_top_k/TopKMeasure_MaxDivergence.tex")]
impl TopKMeasure for MaxDivergence {
    #[cfg(feature = "polars")]
    const DISTRIBUTION: TopKDistribution = TopKDistribution::Exponential;

    fn noisy_top_k<TIA>(x: &Vec<TIA>, scale: f64, k: usize, negate: bool) -> Fallible<Vec<usize>>
    where
        TIA: Number + CastInternalRational,
    {
        exponential_top_k(x, scale, k, negate)
    }

    fn privacy_map(d_in: f64, scale: f64) -> Fallible<f64> {
        // d_in / scale
        d_in.inf_div(&scale)
    }
}

#[proven(proof_path = "measurements/noisy_top_k/TopKMeasure_ZeroConcentratedDivergence.tex")]
impl TopKMeasure for ZeroConcentratedDivergence {
    #[cfg(feature = "polars")]
    const DISTRIBUTION: TopKDistribution = TopKDistribution::Gumbel;

    fn noisy_top_k<TIA>(x: &Vec<TIA>, scale: f64, k: usize, negate: bool) -> Fallible<Vec<usize>>
    where
        TIA: Number,
        f64: InfCast<TIA>,
        FBig: TryFrom<TIA>,
    {
        gumbel_top_k(x, scale, k, negate)
    }

    fn privacy_map(d_in: f64, scale: f64) -> Fallible<f64> {
        // (d_in / scale)^2 / 8
        d_in.inf_div(&scale)?.inf_powi(ibig!(2))?.inf_div(&8.0)
    }
}
