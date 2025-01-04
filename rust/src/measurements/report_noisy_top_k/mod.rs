use crate::{
    core::{Function, Measure, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::{MaxDivergence, RangeDivergence},
    metrics::LInfDistance,
    traits::{
        InfCast, InfDiv, InfMul, Number,
        samplers::{GumbelRV, InverseCDF, PartialSample, Shuffle},
    },
};
use Ordering::*;
use dashu::float::FBig;
use num::Zero;
use opendp_derive::bootstrap;
use std::{cmp::Ordering, fmt::Display};

#[cfg(feature = "polars")]
use super::expr_report_noisy_max::SelectionDistribution;

#[cfg(feature = "ffi")]
mod ffi;
#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    arguments(
        optimize(c_type = "char *", rust_type = "String"),
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
    ),
    generics(MO(suppress), TIA(suppress))
)]
/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable VectorDomain.
/// * `input_metric` - Metric on the input domain. Must be LInfDistance
/// * `output_measure` - One of `MaxDivergence` or `BoundedRange`.
/// * `k` - Number of indices to select.
/// * `scale` - Scale for the noise distribution.
/// * `optimize` - Indicate whether to privately return the "max" or "min"
///
/// # Generics
/// * `TIA` - Atom Input Type. Type of each element in the score vector.
pub fn make_report_noisy_top_k<MO: SelectionMeasure, TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    output_measure: MO,
    k: usize,
    scale: f64,
    optimize: Optimize,
) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, Vec<usize>, LInfDistance<TIA>, MO>>
where
    TIA: Number,
    FBig: TryFrom<TIA> + TryFrom<f64>,
    f64: InfCast<TIA>,
{
    if input_domain.element_domain.nan() {
        return fallible!(
            MakeMeasurement,
            "input_domain member elements must not be nan"
        );
    }

    if let Some(size) = input_domain.size {
        if k > size {
            return fallible!(
                MakeMeasurement,
                "k ({k}) must not exceed the number of candidates ({size})"
            );
        }
    }

    if k > 1 && !MO::ONE_SHOT {
        return fallible!(
            MakeMeasurement,
            "output measure ({:?}) does not currently support one-shot composition (k > 1)",
            output_measure
        );
    }

    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale ({}) must not be negative", scale);
    }

    let f_scale = FBig::try_from(scale.clone())
        .map_err(|_| err!(MakeMeasurement, "scale ({}) must be finite", scale))?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |x: &Vec<TIA>| {
            report_noisy_top_k::<TIA, MO::RV>(x, k, f_scale.clone(), optimize.clone())
        }),
        input_metric.clone(),
        output_measure.clone(),
        PrivacyMap::new_fallible(move |d_in: &TIA| {
            // convert L_\infty distance to range distance
            let d_in = input_metric.range_distance(d_in.clone())?;

            // convert data type to f64
            let d_in = f64::inf_cast(d_in)?;

            // upper bound the privacy loss in terms of the output measure
            output_measure.privacy_map(d_in, scale, k)
        }),
    )
}
pub trait SelectionRV: InverseCDF {
    /// # Proof Definition
    /// `scale` must be non-negative.
    ///
    /// Returns a random variable.
    fn new(shift: FBig, scale: FBig) -> Self;
}

impl SelectionRV for GumbelRV {
    fn new(shift: FBig, scale: FBig) -> Self {
        GumbelRV { shift, scale }
    }
}
impl SelectionRV for ExponentialRV {
    fn new(shift: FBig, scale: FBig) -> Self {
        ExponentialRV { shift, scale }
    }
}

/// # Proof Definition
/// `scale` must be non-negative.
///
/// Returns a noninteractive function with no side-effects that,
/// when given a vector of non-null scores,
/// returns the indices of the top k $z_i$,
/// where each $z_i \sim RV(\mathrm{shift}=y_i, \mathrm{scale}=\texttt{scale})$,
/// and each $y_i = -x_i$ if \texttt{optimize} is \texttt{min}, else $y_i = x_i$.
///
/// The returned function will only return an error if entropy is exhausted.
/// If an error is returned, the error is data-dependent.
pub fn report_noisy_top_k<TIA: Number, RV: SelectionRV>(
    x: &[TIA],
    k: usize,
    scale: FBig,
    optimize: Optimize,
) -> Fallible<Vec<usize>>
where
    FBig: TryFrom<TIA>,
{
    if scale.is_zero() {
        // If scale is zero, we can just return the top k indices.
        // This is a special case that avoids the need to sample.
        let cmp = match optimize {
            Optimize::Max => |l: &mut (usize, &TIA), r: &mut (usize, &TIA)| Ok(l.1 > r.1),
            Optimize::Min => |l: &mut (usize, &TIA), r: &mut (usize, &TIA)| Ok(l.1 < r.1),
        };

        let x_top = top(x.iter().enumerate(), k, cmp)?;
        return Ok(x_top.into_iter().map(|(i, _)| i).collect());
    }

    // When all scores are same, return a random index.
    // This is a workaround for slow performance of the samplers
    // when all scores are the same.
    if x.windows(2).all(|w| w[0] == w[1]) {
        let mut y = (0..x.len()).collect::<Vec<_>>();
        y.shuffle()?;
        return Ok(y[..k].to_vec());
    }

    let iter = (x.iter().enumerate())
        // Cast to FBig and discard failed casts.
        // Cast only fails on NaN scores, which are not in the input domain but could still be passed by the user.
        // If the user still passes NaN in the input data, discarding results in graceful failure.
        .filter_map(|(i, x_i)| Some((i, FBig::try_from(*x_i).ok()?)))
        // Normalize sign.
        .map(|(i, x_i)| {
            let y_i = match optimize {
                Optimize::Min => -x_i,
                Optimize::Max => x_i,
            };
            (i, y_i)
        })
        // Initialize partial sample.
        .map(|(i, shift)| (i, PartialSample::new(RV::new(shift, scale.clone()))));

    // Reduce to the k pairs with largest samples.
    let k_pairs = top(iter, k, |l, r| l.1.greater_than(&mut r.1))?;

    // Discard samples, keep indices.
    Ok(k_pairs.into_iter().map(|(i, _)| i).collect())
}

/// Returns the top k elements from the iterator, using a heap to track the top k elements.
/// Optimized for the case where k is small compared to the number of elements in the iterator.
///
/// # Proof Definition
/// `iter` must be finite and `greater_than` must form a total order.
///
/// Returns the top k elements from the iterator in sorted order.
///
/// Only returns an error if `greater_than` returns an error.
fn top<T>(
    mut iter: impl Iterator<Item = T>,
    k: usize,
    greater_than: impl Fn(&mut T, &mut T) -> Fallible<bool>,
) -> Fallible<Vec<T>> {
    let heap = Vec::with_capacity(k);

    if k == 0 {
        return Ok(heap);
    }

    iter.try_fold(heap, |mut heap, mut value| {
        if let Some(last) = heap.get_mut(k - 1) {
            if greater_than(last, &mut value)? {
                return Ok(heap);
            }
            heap.pop();
        }

        let index = partition_point_mut(&mut heap, |x| greater_than(x, &mut value))?;
        heap.insert(index, value);

        Ok(heap)
    })
}

/// # Proof Definition
/// `x` must be partitioned by `pred`.
/// `pred` may mutate its argument, but not change its true value used for comparisons.
///
/// Returns the index of the partition point according to the given predicate (the index of the first element of the second partition),
/// or an error if the predicate fails.
pub fn partition_point_mut<T>(
    x: &mut Vec<T>,
    mut pred: impl FnMut(&mut T) -> Fallible<bool>,
) -> Fallible<usize> {
    binary_search_by_mut(x, |x_i| Ok(if pred(x_i)? { Less } else { Greater }))
}

/// # Proof Definition
/// `f` may mutate its argument, but not change its true value used for comparisons.
///
/// Returns the index $i$ of the first element in $x$ that is less than $f(x_i)$,
/// or an error if the comparator fails.
pub fn binary_search_by_mut<T>(
    x: &mut Vec<T>,
    mut f: impl FnMut(&mut T) -> Fallible<Ordering>,
) -> Fallible<usize> {
    let mut size = x.len();
    if size == 0 {
        return Ok(0);
    }
    let mut base = 0usize;

    while size > 1 {
        let half = size / 2;
        let mid = base + half;

        let cmp = f(&mut x[mid])?;
        base = if let Greater = cmp { base } else { mid };

        size -= half;
    }

    let cmp = f(&mut x[base])?;
    Ok(base + (cmp == Less) as usize)
}

/// Defines the noise distribution associated with the privacy measure.
pub trait SelectionMeasure: 'static + Measure<Distance = f64> {
    /// # Proof Definition
    /// ONE_SHOT is true if the measure supports one-shot top-$k$ release.
    const ONE_SHOT: bool;

    #[cfg(feature = "polars")]
    const DISTRIBUTION: SelectionDistribution;

    /// # Proof Definition
    /// The type of a random variable whose distribution is in the location-scale family
    /// `RV = shift + scale * X`, where X is a random variable.
    type RV: SelectionRV;

    /// # Proof Definition
    /// `scale` must be positive.
    ///
    /// Returns a random variable whose location and scale are parameterized by `shift` and `scale`.
    fn random_variable(shift: FBig, scale: FBig) -> Self::RV;

    /// # Proof Definition
    /// `d_in` must be non-null and non-negative.
    /// `scale` must be non-null and non-negative.
    ///
    /// Let `input_domain` be a VectorDomain with non-null numeric elements (scores).
    ///
    /// Let $texttt{function}$ be a noninteractive function with no side-effects that,
    /// when given a vector of non-null scores,
    /// returns the indices of the top k $z_i$,
    /// where each $z_i \sim RV(\mathrm{shift}=y_i, \mathrm{scale}=\texttt{scale})$,
    /// and each $y_i = -x_i$ if \texttt{optimize} is \texttt{min}, else $y_i = x_i$.
    ///
    /// Define the range distance as
    /// ```math
    /// d_{\mathrm{Range}}(x, x') = \max_{ij} |(x_i - x'_i) - (x_j - x'_j)|.
    /// ```
    ///
    /// For every pair of elements $x, x'$ in `input_domain` and for every pair $(d_\mathrm{in}, d_\mathrm{out})$,
    /// if $x, x'$ are $d_\mathrm{in}$-close under the range distance, $\texttt{privacy\_map}(\d_\mathrm{in}, \texttt{scale}, $k$)$ does not raise an exception,
    /// and $\texttt{privacy\_map}(\d_\mathrm{in}, \texttt{scale}, $k$) \leq d_\mathrm{out}$,
    /// then $\texttt{function}(x), \texttt{function}(x')$ are $d_\mathrm{out}$-close under \texttt{output\_measure} (Self).
    fn privacy_map(&self, d_in: f64, scale: f64, k: usize) -> Fallible<f64> {
        if d_in.is_sign_negative() {
            return fallible!(
                InvalidDistance,
                "sensitivity ({}) must be non-negative",
                d_in
            );
        }

        if scale.is_zero() {
            return Ok(f64::INFINITY);
        }

        // d_out >= d_in / scale
        d_in.inf_div(&scale)?.inf_mul(&f64::inf_cast(k)?)
    }
}

impl SelectionMeasure for RangeDivergence {
    const ONE_SHOT: bool = true;

    #[cfg(feature = "polars")]
    const DISTRIBUTION: SelectionDistribution = SelectionDistribution::Gumbel;

    type RV = GumbelRV;

    fn random_variable(shift: FBig, scale: FBig) -> Self::RV {
        GumbelRV { shift, scale }
    }
}

impl SelectionMeasure for MaxDivergence {
    const ONE_SHOT: bool = false;

    #[cfg(feature = "polars")]
    const DISTRIBUTION: SelectionDistribution = SelectionDistribution::Exponential;

    type RV = ExponentialRV;

    fn random_variable(shift: FBig, scale: FBig) -> Self::RV {
        ExponentialRV { shift, scale }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "polars", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "polars", serde(rename_all = "lowercase"))]
pub enum Optimize {
    Min,
    Max,
}

impl Display for Optimize {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Optimize::Min => f.write_str("min"),
            Optimize::Max => f.write_str("max"),
        }
    }
}

impl TryFrom<&str> for Optimize {
    type Error = crate::error::Error;
    fn try_from(s: &str) -> Fallible<Self> {
        Ok(match s {
            "min" => Optimize::Min,
            "max" => Optimize::Max,
            _ => return fallible!(FailedCast, "optimize must be \"min\" or \"max\""),
        })
    }
}
