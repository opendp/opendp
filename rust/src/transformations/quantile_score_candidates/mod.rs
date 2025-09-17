use std::{cmp::Ordering, iter::zip};

use dashu::{integer::UBig, rational::RBig};
use opendp_derive::{bootstrap, proven};

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{IntDistance, LInfDistance},
    traits::{AlertingMul, ExactIntCast, InfDiv, Integer, Number, RoundCast},
};

use super::traits::UnboundedMetric;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    generics(MI(suppress), TIA(suppress)),
    derived_types(TIA = "$get_atom(get_type(input_domain))")
)]
/// Makes a Transformation that scores how similar each candidate is to the given `alpha`-quantile on the input dataset.
///
/// # Arguments
/// * `input_domain` - Uses a smaller sensitivity when the size of vectors in the input domain is known.
/// * `input_metric` - Either SymmetricDistance or InsertDeleteDistance.
/// * `candidates` - Potential quantiles to score
/// * `alpha` - a value in $[0, 1]$. Choose 0.5 for median
///
/// # Generics
/// * `MI` - Input Metric.
/// * `TIA` - Atomic Input Type. Type of elements in the input vector
pub fn make_quantile_score_candidates<MI: UnboundedMetric, TIA: Number>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: MI,
    candidates: Vec<TIA>,
    alpha: f64,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        MI,
        VectorDomain<AtomDomain<u64>>,
        LInfDistance<u64>,
    >,
>
where
    (VectorDomain<AtomDomain<TIA>>, MI): MetricSpace,
{
    if input_domain.element_domain.nan() {
        return fallible!(
            MakeTransformation,
            "input_domain members must have non-nan elements"
        );
    }

    check_candidates(&candidates)?;

    let (alpha_num, alpha_den, size_limit) = score_candidates_constants(
        input_domain.size.map(u64::exact_int_cast).transpose()?,
        alpha,
    )?;

    Transformation::<_, _, VectorDomain<AtomDomain<u64>>, _>::new(
        input_domain.clone(),
        input_metric,
        VectorDomain::default().with_size(candidates.len()),
        LInfDistance::default(),
        Function::new(move |arg: &Vec<TIA>| {
            Vec::from_iter(score_candidates(
                arg.iter().cloned(),
                candidates.clone(),
                alpha_num,
                alpha_den,
                size_limit,
            ))
        }),
        StabilityMap::new_fallible(score_candidates_map(
            alpha_num,
            alpha_den,
            input_domain.size.is_some(),
        )),
    )
}

/// # Proof Definition
/// Raises an error if:
/// * `candidates` is empty
/// * `candidates` is not strictly increasing
/// * `candidates` is not totally ordered
#[proven(proof_path = "transformations/quantile_score_candidates/check_candidates.tex")]
pub(crate) fn check_candidates<T: Number>(candidates: &Vec<T>) -> Fallible<()> {
    if candidates.is_empty() {
        return fallible!(MakeTransformation, "candidates must be non-empty");
    }
    if candidates.windows(2).any(|w| {
        w[0].partial_cmp(&w[1])
            // NaN always evaluates to false
            .map(|c| c != Ordering::Less)
            .unwrap_or(true)
    }) {
        return fallible!(
            MakeTransformation,
            "candidates must be non-null and strictly increasing"
        );
    }
    Ok(())
}

/// Returns the following constants:
/// * `alpha_num`
/// * `alpha_den`
/// * `size_limit`
///
/// The constants are chosen to ensure that `alpha_num / alpha_den` is approximately equal to `alpha`,
/// in a way that minimizes `alpha_den` to maximize `size_limit`.
///
/// # Proof Definition
/// The constants follow the following properties:
/// * $alpha_num / alpha_den \in [0, 1]$
/// * $size_limit \cdot alpha_den < 2^{64}$
///
/// An error is raised if these properties cannot be met.
#[proven(proof_path = "transformations/quantile_score_candidates/score_candidates_constants.tex")]
pub(crate) fn score_candidates_constants(
    size: Option<u64>,
    alpha: f64,
) -> Fallible<(u64, u64, u64)> {
    if !(0.0..=1.0).contains(&alpha) {
        return fallible!(MakeTransformation, "alpha must be within [0, 1]");
    }

    let (alpha_num_exact, alpha_den_exact) = RBig::try_from(alpha)?.into_parts();

    let alpha_den_approx = if let Some(size) = size {
        // choose the finest granularity that won't overflow
        // must have that size * denom < MAX, so let denom = MAX // size
        u64::MAX.neg_inf_div(&size)?
    } else {
        // default to an alpha granularity of .00001
        u64::exact_int_cast(10_000)?
    };

    let (alpha_num, alpha_den) = if alpha_den_exact < UBig::from(alpha_den_approx) {
        (
            u64::try_from(alpha_num_exact.into_parts().1)?,
            u64::try_from(alpha_den_exact)?,
        )
    } else {
        // numer = alpha * denom
        let alpha_num_approx = u64::round_cast(alpha * f64::round_cast(alpha_den_approx.clone())?)?;
        (alpha_num_approx, alpha_den_approx)
    };

    let size_limit = if let Some(size_limit) = size {
        size_limit
    } else {
        u64::MAX.neg_inf_div(&alpha_den)?
    };

    // both of these are un-fail-able, but they make the proof easy
    assert!(alpha_num <= alpha_den);
    size_limit.alerting_mul(&alpha_den)?;

    Ok((alpha_num, alpha_den, size_limit))
}

/// # Proof Definition
/// Assume `alpha_den >= alpha_num`.
///
/// If `known_size` is set,
/// then returns a function that computes $\texttt{d\_in} // 2 \cdot \texttt{alpha\_den}$
/// for argument $\texttt{d\_in}$.
///
/// Otherwise,
/// returns a function that computes $\texttt{d\_in} \cdot \max(\texttt{alpha\_num}, \texttt{alpha\_den} - \texttt{alpha\_num})$,
/// for argument $\texttt{d\_in}$.
#[proven(proof_path = "transformations/quantile_score_candidates/score_candidates_map.tex")]
pub(crate) fn score_candidates_map<T: Integer + ExactIntCast<IntDistance>>(
    alpha_num: T,
    alpha_den: T,
    known_size: bool,
) -> impl Fn(&IntDistance) -> Fallible<T> {
    move |d_in| {
        if known_size {
            T::exact_int_cast(d_in / 2)? // round down to even
                .alerting_mul(&alpha_den)
        } else {
            let abs_dist_const = alpha_num.max(alpha_den - alpha_num);
            T::exact_int_cast(*d_in)?.alerting_mul(&abs_dist_const)
        }
    }
}

/// Compute score of each candidate on a dataset
///
/// # Proof Definition
///
/// Under the precondition that `x` is totally ordered,
/// that `x` has no greater than $2^{64}$ elements,
/// that `candidates` is strictly increasing,
/// that `alpha_numer / alpha_denom <= 1`,
/// and that `size_limit * alpha_denom < 2^{64}`,
/// computes the score of each candidate in `candidates` on the dataset `x`.
///
/// The score for each `c` in `candidates` is computed as follows:
/// |alpha_denom * min(#(x < c), size_limit) -
///  alpha_numer * min(#(x > c)), size_limit)|
///
/// # Intuition
/// Lower score is better.
/// Score is roughly |observed_value - ideal_value|, where ideal_value is a rescaled `alpha`-quantile.
/// We want greater scores when observed value is near ideal value.
/// The further away the observed value is from the ideal value, the more negative it gets
///
/// # Arguments
/// * `x` - dataset to score against. Must be non-null
/// * `candidates` - values to be scored. Must be strictly increasing
/// * `alpha_num` - numerator of alpha fraction
/// * `alpha_den` - denominator of alpha fraction. alpha fraction is {0: min, 0.5: median, 1: max, ...}
/// * `size_limit` - maximum size of `x`. If `x` is larger than `size_limit`, scores are truncated
#[proven(proof_path = "transformations/quantile_score_candidates/score_candidates.tex")]
pub(crate) fn score_candidates<TIA: PartialOrd>(
    x: impl Iterator<Item = TIA>,
    candidates: Vec<TIA>,
    alpha_num: u64,
    alpha_den: u64,
    size_limit: u64,
) -> impl Iterator<Item = u64> {
    // count of the number of records between...
    //  (-inf, c1), [c1, c2), [c2, c3), ..., [ck, inf)
    let mut hist_ro = vec![0u64; candidates.len() + 1]; // histogram of right-open intervals
    //  (-inf, c1], (c1, c2], (c2, c3], ..., (ck, inf)
    let mut hist_lo = vec![0u64; candidates.len() + 1]; // histogram of left-open intervals

    x.for_each(|x_i| {
        let idx_lt = candidates.partition_point(|c| *c < x_i);
        hist_lo[idx_lt] += 1;

        let idx_eq = idx_lt + candidates[idx_lt..].partition_point(|c| *c == x_i);
        hist_ro[idx_eq] += 1;
    });

    let n: u64 = hist_lo.iter().sum();

    // don't care about the number of elements greater than all candidates
    hist_ro.pop();
    hist_lo.pop();

    let (mut lt, mut le) = (0u64, 0u64);

    zip(hist_ro, hist_lo).map(move |(ro, lo)| {
        // cumsum the right-open histogram to get the total number of records less than the candidate
        lt += ro;
        // cumsum the left-open histogram to get the total number of records lt or equal to the candidate
        le += lo;

        let gt = n - le;

        // ensures the score calculation won't overflow
        let (lt_lim, gt_lim) = (lt.min(size_limit), gt.min(size_limit));

        // |     (1 - α)         * #(x < c)    -           α * #(x > c)| * α_den
        ((alpha_den - alpha_num) * lt_lim).abs_diff(alpha_num * gt_lim)
    })
}
