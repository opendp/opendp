use crate::{
    core::{Function, Measure, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::{Error, ErrorVariant, Fallible},
    measures::{MaxDivergence, ZeroConcentratedDivergence},
    metrics::LInfDistance,
    traits::{
        CastInternalRational, FiniteBounds, InfCast, InfDiv, InfMul, InfPowI, Number, ProductOrd,
    },
};
use dashu::{base::Sign, float::FBig, ibig, rational::RBig};
use num::Zero;
use opendp_derive::{bootstrap, proven};
use opendp_verified::{
    error::{Error as VerifiedError, ErrorVariant as VerifiedErrorVariant},
    measurements::noisy_top_k as verified_noisy_top_k,
};

#[cfg(feature = "ffi")]
mod ffi;
#[cfg(test)]
mod test;

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
/// # Citations
/// * [MS20 Permute-and-Flip: A New Mechanism for Differentially Private Selection](https://arxiv.org/abs/2010.12603)
///
/// # Runtime
/// Per release, worst-case runtime is `O(k n)`, where `n` is the number of scores.
/// For `k = 1`, this reduces to `O(n)`.
///
/// # Utility
/// When `scale = 0`, the mechanism returns the exact top-`k` indices.
/// For positive `scale`, utility improves as score gaps grow relative to `scale`;
/// for a fixed gap parameter `g`, the failure probability decays exponentially in `g / scale`
/// up to constants from the selection mechanism analysis.
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
        Function::new_fallible(move |x: &Vec<TIA>| {
            noisy_top_k(x, scale, k, negate, MO::REPLACEMENT)
        }),
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
    /// # Proof Definition
    /// If replacement is set, the function $f$ returns a sample from $\mathcal{M}_{EM}$ (as defined in MS2020 Definition 4),
    /// otherwise returns a sample from $\mathcal{M}_{PF}$ (as defined in MS2020 Lemma 1),
    /// $k$ times by peeling.
    const REPLACEMENT: bool;

    /// Define
    /// ```math
    /// d_{\mathrm{Range}}(x, x') = max_{ij} |(x_i - x'_i) - (x_j - x'_j)|.
    /// ```
    ///
    /// # Proof Definition
    /// For any $x, x'$ where $d_\mathrm{in} \ge d_\mathrm{Range}(x, x')$,
    /// return $d_\mathrm{out} \ge D_\mathrm{self}(f(x), f(x'))$,
    /// where $f(x) = \mathrm{noisy\_top\_k}(x=x, k=1, \mathrm{scale}=\mathrm{scale}, \mathrm{replacement}=\mathrm{Self::REPLACEMENT})$.
    fn privacy_map(d_in: f64, scale: f64) -> Fallible<f64>;
}

#[proven(proof_path = "measurements/noisy_top_k/TopKMeasure_MaxDivergence.tex")]
impl TopKMeasure for MaxDivergence {
    const REPLACEMENT: bool = false;

    fn privacy_map(d_in: f64, scale: f64) -> Fallible<f64> {
        // d_in / scale
        d_in.inf_div(&scale)
    }
}

#[proven(proof_path = "measurements/noisy_top_k/TopKMeasure_ZeroConcentratedDivergence.tex")]
impl TopKMeasure for ZeroConcentratedDivergence {
    const REPLACEMENT: bool = true;

    fn privacy_map(d_in: f64, scale: f64) -> Fallible<f64> {
        // (d_in / scale)^2 / 8
        d_in.inf_div(&scale)?.inf_powi(ibig!(2))?.inf_div(&8.0)
    }
}

#[proven]
/// Returns the indices of the noisy top k elements.
/// Gumbel noise when replacement is true, otherwise exponential noise.
///
/// # Proof Definition
/// Each value in $x$ must be finite.
///
/// If replacement is set, returns a sample from $\mathcal{M}_{EM}$ (as defined in MS2023 Definition 4),
/// otherwise returns a sample from $\mathcal{M}_{PF}$ (as defined in MS2023 Lemma 1),
/// $k$ times by peeling,
/// where $\texttt{scale} = \frac{2 \cdot \Delta}{\epsilon}$.
pub(crate) fn noisy_top_k<TIA: Clone + CastInternalRational + ProductOrd + FiniteBounds>(
    x: &[TIA],
    scale: f64,
    k: usize,
    negate: bool,
    replacement: bool,
) -> Fallible<Vec<usize>> {
    let sign = Sign::from(negate);
    let scale = scale.into_rational()?;

    let y = (x.into_iter().cloned())
        .map(|x_i| {
            x_i.total_clamp(TIA::MIN_FINITE, TIA::MAX_FINITE)?
                .into_rational()
                .map(|x_i| x_i * sign)
        })
        .collect::<Fallible<_>>()?;

    verified_noisy_top_k::peel_permute_and_flip(y, scale, k, replacement)
        .map_err(from_verified_error)
}

#[proven]
/// # Proof Definition
/// If replacement is set, returns a sample from $\mathcal{M}_{EM}$ (as defined in MS2023 Definition 4),
/// otherwise returns a sample from $\mathcal{M}_{PF}$ (as defined in MS2023 Lemma 1),
/// $k$ times by peeling,
/// where $\texttt{scale} = \frac{2 \cdot \Delta}{\epsilon}$.
fn peel_permute_and_flip(
    x: Vec<RBig>,
    scale: RBig,
    k: usize,
    replacement: bool,
) -> Fallible<Vec<usize>> {
    verified_noisy_top_k::peel_permute_and_flip(x, scale, k, replacement)
        .map_err(from_verified_error)
}

#[proven]
/// # Proof Definition
/// If replacement is set, returns a sample from $\mathcal{M}_{EM}$ (as defined in MS2023 Definition 4),
/// otherwise returns a sample from $\mathcal{M}_{PF}$ (as defined in MS2023 Lemma 1),
/// where $\texttt{scale} = \frac{2 \cdot \Delta}{\epsilon}$.
fn permute_and_flip(x: &[RBig], scale: &RBig, replacement: bool) -> Fallible<usize> {
    verified_noisy_top_k::permute_and_flip(x, scale, replacement).map_err(from_verified_error)
}

fn from_verified_error(error: VerifiedError) -> Error {
    let variant = match error.variant {
        VerifiedErrorVariant::FailedFunction => ErrorVariant::FailedFunction,
        VerifiedErrorVariant::FailedCast => ErrorVariant::FailedCast,
        VerifiedErrorVariant::EntropyExhausted => ErrorVariant::EntropyExhausted,
        _ => ErrorVariant::FailedFunction,
    };
    Error {
        variant,
        message: error.message,
        backtrace: error.backtrace,
    }
}
