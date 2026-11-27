use crate::{
    core::{Function, Measure, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::{
        AccountingPreference, MaxDivergence, MultiDP, PrivacyGuarantee, ZeroConcentratedDivergence,
    },
    metrics::LInfDistance,
    traits::{
        CastInternalRational, FiniteBounds, InfCast, InfDiv, InfMul, InfPowI, Number, ProductOrd,
        SInterval,
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
/// * `output_measure` - Privacy measure used for accounting. One of `MaxDivergence`, `ZeroConcentratedDivergence`, or `MultiDP` with an accounting preference.
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
    let replacement = output_measure.replacement()?;
    let measure = output_measure.clone();

    Measurement::new(
        input_domain,
        input_metric,
        output_measure,
        Function::new_fallible(move |x: &Vec<TIA>| noisy_top_k(x, scale, k, negate, replacement)),
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

            measure.privacy_map(d_in, scale, k)
        }),
    )
}

#[bootstrap(
    features("contrib"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        negate(default = false),
    ),
    generics(MO(suppress), TIA(suppress))
)]
/// Report the indices selected by a private top-k mechanism.
///
/// This is an alias for [`make_noisy_top_k`] that makes the report-level API
/// explicit while preserving the same accounting and distribution semantics.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector.
/// * `input_metric` - Metric on the input domain.
/// * `output_measure` - Privacy measure used for accounting.
/// * `k` - Number of indices to select.
/// * `scale` - Scale for the selection distribution.
/// * `negate` - Set to true to report the bottom k indices.
pub fn make_report_noisy_top_k<MO: TopKMeasure, TIA>(
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
    make_noisy_top_k(input_domain, input_metric, output_measure, k, scale, negate)
}

pub trait TopKMeasure: Measure + 'static {
    /// # Proof Definition
    /// Returns whether to sample with replacement (exponential mechanism)
    /// or without replacement (permute-and-flip), or an error.
    fn replacement(&self) -> Fallible<bool>;

    /// # Proof Definition
    /// For nonnegative range sensitivity d_in and scale, returns a valid
    /// privacy bound for k peeling rounds of the mechanism selected by self.
    fn privacy_map(&self, d_in: f64, scale: f64, k: usize) -> Fallible<Self::Distance>;
}

fn top_k_epsilon(d_in: f64, scale: f64, k: usize) -> Fallible<f64> {
    if d_in.is_zero() {
        return Ok(0.0);
    }
    if scale.is_zero() {
        return Ok(f64::INFINITY);
    }
    d_in.inf_div(&scale)?.inf_mul(&f64::inf_cast(k)?)
}

fn top_k_rho(d_in: f64, scale: f64, k: usize) -> Fallible<f64> {
    if d_in.is_zero() {
        return Ok(0.0);
    }
    if scale.is_zero() {
        return Ok(f64::INFINITY);
    }
    d_in.inf_div(&scale)?
        .inf_powi(ibig!(2))?
        .inf_div(&8.0)?
        .inf_mul(&f64::inf_cast(k)?)
}

#[proven(proof_path = "measurements/noisy_top_k/TopKMeasure_MaxDivergence.tex")]
impl TopKMeasure for MaxDivergence {
    fn replacement(&self) -> Fallible<bool> {
        Ok(false)
    }

    fn privacy_map(&self, d_in: f64, scale: f64, k: usize) -> Fallible<f64> {
        top_k_epsilon(d_in, scale, k)
    }
}

#[proven(proof_path = "measurements/noisy_top_k/TopKMeasure_ZeroConcentratedDivergence.tex")]
impl TopKMeasure for ZeroConcentratedDivergence {
    fn replacement(&self) -> Fallible<bool> {
        Ok(true)
    }

    fn privacy_map(&self, d_in: f64, scale: f64, k: usize) -> Fallible<f64> {
        top_k_rho(d_in, scale, k)
    }
}

// Optimize the two-endpoint privacy-loss distribution over all intervals of
// width nu. Work in log space to avoid exponentiating positive large values.
fn bounded_range_rdp(alpha: f64, nu: f64) -> Fallible<f64> {
    if !alpha.is_finite() || alpha <= 1.0 {
        return fallible!(FailedMap, "Rényi order must be finite and greater than one");
    }
    if nu == 0.0 || nu.is_infinite() {
        return Ok(nu);
    }
    let evaluate = || -> Fallible<f64> {
        let nu = SInterval::point(nu)?;
        let alpha = SInterval::point(alpha)?;
        let t = (alpha.clone() - SInterval::point(1.0)?)?;
        let log_one_minus_exp = |x: SInterval| (-(-x).exp_m1()?).ln();
        let a = log_one_minus_exp(nu.clone())?;
        let b = log_one_minus_exp((alpha.clone() * nu.clone())?)?;
        let c = log_one_minus_exp((t.clone() * nu.clone())?)?;
        let log_alpha = alpha.ln()?;
        let first = (((b.clone() - a)? - log_alpha.clone())? / t.clone())?;
        let second = (((b - c)? + t.ln()?)? - log_alpha)?;
        ((nu + first)? + second)?.upper_f64()
    };
    // The range bound itself implies nu-DP, hence nu-RDP at every order.
    Ok(evaluate().unwrap_or(nu).clamp(0.0, nu))
}

#[proven(proof_path = "measurements/noisy_top_k/TopKMeasure_MultiDP.tex")]
impl TopKMeasure for MultiDP {
    fn replacement(&self) -> Fallible<bool> {
        match self.preference() {
            AccountingPreference::PureDp => Ok(false),
            AccountingPreference::Zcdp => Ok(true),
            AccountingPreference::Auto => fallible!(
                MakeMeasurement,
                "accounting preference is required for MultiDP noisy selection"
            ),
        }
    }

    fn privacy_map(&self, d_in: f64, scale: f64, k: usize) -> Fallible<PrivacyGuarantee> {
        let mut guarantee = PrivacyGuarantee::new();
        let replacement = self.replacement()?;
        if replacement {
            guarantee = guarantee.with_zCDP(top_k_rho(d_in, scale, k)?, 0.0)?;
        }
        let epsilon = top_k_epsilon(d_in, scale, k)?;
        if epsilon.is_finite() {
            guarantee = guarantee.with_approx_dp(epsilon, 0.0)?;
            if replacement {
                let nu = if epsilon == 0.0 {
                    0.0
                } else {
                    d_in.inf_div(&scale)?
                };
                let count = f64::inf_cast(k)?;
                guarantee = guarantee.with_renyiDP(
                    move |orders| {
                        orders
                            .iter()
                            .map(|&alpha| {
                                Ok(bounded_range_rdp(alpha, nu)?.inf_mul(&count)?.min(epsilon))
                            })
                            .collect()
                    },
                    0.0,
                )?;
            }
        }
        Ok(guarantee)
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
