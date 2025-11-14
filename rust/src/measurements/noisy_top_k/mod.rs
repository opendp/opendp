use crate::{
    core::{Function, Measure, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::{MaxDivergence, ZeroConcentratedDivergence},
    metrics::LInfDistance,
    traits::{
        CastInternalRational, FiniteBounds, InfCast, InfDiv, InfMul, InfPowI, Number, ProductOrd,
        samplers::{sample_bernoulli_exp, sample_uniform_uint_below},
    },
};
use dashu::{base::Sign, float::FBig, ibig, rational::RBig};
use num::Zero;
use opendp_derive::{bootstrap, proven};
use std::{collections::BTreeSet, ops::Range};

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

    peel_permute_and_flip(y, scale, k, replacement)
}

#[proven]
/// # Proof Definition
/// If replacement is set, returns a sample from $\mathcal{M}_{EM}$ (as defined in MS2023 Definition 4),
/// otherwise returns a sample from $\mathcal{M}_{PF}$ (as defined in MS2023 Lemma 1),
/// $k$ times by peeling,
/// where $\texttt{scale} = \frac{2 \cdot \Delta}{\epsilon}$.
fn peel_permute_and_flip(
    mut x: Vec<RBig>,
    scale: RBig,
    k: usize,
    replacement: bool,
) -> Fallible<Vec<usize>> {
    let mut natural_order = Vec::new();
    let mut sorted_order = BTreeSet::new();

    for _ in 0..k.min(x.len()) {
        let mut index = permute_and_flip(&x, &scale, replacement)?;
        x.remove(index);

        // map index on modified x back to original x (postprocessing)
        for &del in &sorted_order {
            if del <= index { index += 1 } else { break }
        }

        sorted_order.insert(index);
        natural_order.push(index);
    }
    Ok(natural_order)
}

#[proven]
/// # Proof Definition
/// If replacement is set, returns a sample from $\mathcal{M}_{EM}$ (as defined in MS2023 Definition 4),
/// otherwise returns a sample from $\mathcal{M}_{PF}$ (as defined in MS2023 Lemma 1),
/// where $\texttt{scale} = \frac{2 \cdot \Delta}{\epsilon}$.
fn permute_and_flip(x: &[RBig], scale: &RBig, replacement: bool) -> Fallible<usize> {
    let x_is_empty = || err!(FailedFunction, "x is empty");

    if scale.is_zero() {
        return (0..x.len()).max_by_key(|&i| &x[i]).ok_or_else(x_is_empty);
    }

    let x_max = x.iter().max().ok_or_else(x_is_empty)?;

    let mut candidates: Vec<usize> = (0..x.len()).collect();

    let sequence = match replacement {
        false => Sequence::Range(0..x.len()),
        true => Sequence::Zero,
    };

    for left in sequence {
        let right = left + sample_uniform_uint_below(x.len() - left)?;
        candidates.swap(left, right); // if w/o replacement, fisher-yates shuffle up to left

        let candidate = candidates[left];
        if sample_bernoulli_exp((x_max - &x[candidate]) / scale)? {
            return Ok(candidate);
        }
    }
    unreachable!("at least one x[candidate] is equal to x_max")
}

enum Sequence {
    Range(Range<usize>),
    Zero,
}
impl Iterator for Sequence {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Sequence::Range(range) => range.next(),
            Sequence::Zero => Some(0),
        }
    }
}
