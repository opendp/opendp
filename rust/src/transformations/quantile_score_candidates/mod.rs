use std::cmp::Ordering;

use dashu::{integer::UBig, rational::RBig};
use num::NumCast;
use opendp_derive::bootstrap;

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{IntDistance, LInfDistance},
    traits::{ExactIntCast, Integer, Number, RoundCast},
};

use super::traits::UnboundedMetric;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    generics(MI(suppress), TIA(suppress)),
    derived_types(TIA = "$get_atom(get_type(input_domain))")
)]
/// Makes a Transformation that scores how similar each candidate is to the given `alpha`-quantile on the input dataset.
///
///
/// # Arguments
/// * `input_domain` - Uses a tighter sensitivity when the size of vectors in the input domain is known.
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
        VectorDomain<AtomDomain<usize>>,
        MI,
        LInfDistance<usize>,
    >,
>
where
    (VectorDomain<AtomDomain<TIA>>, MI): MetricSpace,
{
    if input_domain.element_domain.nullable() {
        return fallible!(MakeTransformation, "input must be non-null");
    }

    validate_candidates(&candidates)?;

    let (alpha_num, alpha_den, size_limit) = score_candidates_constants(input_domain.size, alpha)?;

    Transformation::<_, VectorDomain<AtomDomain<usize>>, _, _>::new(
        input_domain.clone(),
        VectorDomain::default().with_size(candidates.len()),
        Function::new(move |arg: &Vec<TIA>| {
            compute_score(arg.clone(), &candidates, alpha_num, alpha_den, size_limit)
        }),
        input_metric,
        LInfDistance::default(),
        StabilityMap::new_fallible(score_candidates_map(
            alpha_num,
            alpha_den,
            input_domain.size.is_some(),
        )),
    )
}

pub(crate) fn validate_candidates<T: Number>(candidates: &Vec<T>) -> Fallible<()> {
    if candidates.is_empty() {
        return fallible!(MakeTransformation, "candidates must be non-empty");
    }
    if candidates.windows(2).any(|w| {
        w[0].partial_cmp(&w[1])
            .map(|c| c != Ordering::Less)
            .unwrap_or(true)
    }) {
        return fallible!(
            MakeTransformation,
            "candidates must be non-null and increasing"
        );
    }
    Ok(())
}

pub(crate) fn score_candidates_constants<T: Integer>(
    size: Option<T>,
    alpha: f64,
) -> Fallible<(T, T, T)>
where
    f64: RoundCast<T>,
    T: RoundCast<f64> + NumCast,
    UBig: From<T>,
{
    if !(0.0..=1.0).contains(&alpha) {
        return fallible!(MakeTransformation, "alpha must be within [0, 1]");
    }

    let (alpha_num_exact, alpha_den_exact) = RBig::try_from(alpha)?.into_parts();

    let alpha_den_approx = if let Some(size) = size {
        // choose the finest granularity that won't overflow
        // must have that size * denom < MAX, so let denom = MAX // size
        T::MAX_FINITE.neg_inf_div(&size)?
    } else {
        // default to an alpha granularity of .00001
        T::exact_int_cast(10_000)?
    };

    let (alpha_num, alpha_den) = if alpha_den_exact < UBig::from(alpha_den_approx) {
        T::from(alpha_num_exact.into_parts().1)
            .zip(T::from(alpha_den_exact))
            .unwrap()
    } else {
        // numer = alpha * denom
        let alpha_num_approx = T::round_cast(alpha * f64::round_cast(alpha_den_approx.clone())?)?;
        (alpha_num_approx, alpha_den_approx)
    };

    let size_limit = if let Some(size_limit) = size {
        // ensures that there is no overflow
        size_limit.alerting_mul(&alpha_den)?;
        size_limit
    } else {
        T::MAX_FINITE.neg_inf_div(&alpha_den)?
    };

    Ok((alpha_num, alpha_den, size_limit))
}

pub(crate) fn score_candidates_map<T: Integer + ExactIntCast<IntDistance>>(
    alpha_num: T,
    alpha_den: T,
    known_size: bool,
) -> impl Fn(&IntDistance) -> Fallible<T> {
    move |d_in| {
        if known_size {
            T::exact_int_cast(d_in / 2 * 2)? // round down to even
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
/// Under the precondition that `x` is non-null,
/// that `candidates` is strictly increasing,
/// that `alpha_numer / alpha_denom` is in [0, 1],
/// and that `size_limit * alpha_denom` does not overflow,
/// computes the score of each candidate in `candidates` on the dataset `x`.
///
/// The score for each `c` in `candidates` is computed as follows:
/// |alpha_denom * min(#(x < c), size_limit) -
///  alpha_numer * min(n - #(x = c)), size_limit)|
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
pub(crate) fn compute_score<TIA: PartialOrd>(
    mut x: Vec<TIA>,
    candidates: &Vec<TIA>,
    alpha_num: usize,
    alpha_den: usize,
    size_limit: usize,
) -> Vec<usize> {
    // x must be sorted because counts are done via binary search
    x.sort_by(|a, b| a.partial_cmp(&b).unwrap_or(Ordering::Equal));

    // compute #(`x` < c) and #(`x` = c) for each c in candidates
    let mut num_lt = vec![0; candidates.len()];
    let mut num_eq = vec![0; candidates.len()];
    count_lt_eq_recursive(
        num_lt.as_mut_slice(),
        num_eq.as_mut_slice(),
        candidates.as_slice(),
        x.as_slice(),
        0,
    );

    // now that we have num_lt and num_eq for each candidate, score all candidates
    num_lt
        .into_iter()
        .zip(num_eq.into_iter())
        // score function cannot overflow.
        //     lt <= size_limit, so 0 <= alpha_denom * lt <= usize::MAX
        //     n - eq <= size_limit, so 0 <= size_limit - eq
        .map(|(lt, eq)| {
            // |α_den * #(x < c) - α_num * (n - #(x = c))|
            (alpha_den * lt.min(size_limit)).abs_diff(alpha_num * (x.len() - eq).min(size_limit))
        })
        .collect()
}

/// Compute number of elements less than and equal to each edge
/// Formula is (#(`x` < `e`), #(`x` == `e`)) for each e in `edges`.
///
/// # Arguments
/// * `count_lt` - location to write the count of elements less than each edge
/// * `count_eq` - location to write the count of elements equal to each edge
/// * `edges` - edges to collect counts for. Must be sorted
/// * `x` - dataset to count against
/// * `x_start_idx` - value to add to the count. Useful for recursion on subslices
fn count_lt_eq_recursive<TI: PartialOrd>(
    count_lt: &mut [usize],
    count_eq: &mut [usize],
    edges: &[TI],
    x: &[TI],
    x_start_idx: usize,
) {
    if edges.is_empty() {
        return;
    }
    if edges.len() == 1 {
        let (num_lt, num_eq) = count_lt_eq(x, &edges[0]);
        count_lt[0] = x_start_idx + num_lt;
        count_eq[0] = num_eq;
        return;
    }
    // use binary search to find #(x < middle edge) = |{i; x[i] < middle edge}|
    let mid_edge_idx = (edges.len() + 1) / 2;
    let mid_edge = &edges[mid_edge_idx];
    let (num_lt, num_eq) = count_lt_eq(x, mid_edge);
    count_lt[mid_edge_idx] = x_start_idx + num_lt;
    count_eq[mid_edge_idx] = num_eq;

    count_lt_eq_recursive(
        &mut count_lt[..mid_edge_idx],
        &mut count_eq[..mid_edge_idx],
        &edges[..mid_edge_idx],
        &x[..num_lt + num_eq],
        x_start_idx,
    );

    count_lt_eq_recursive(
        &mut count_lt[mid_edge_idx + 1..],
        &mut count_eq[mid_edge_idx + 1..],
        &edges[mid_edge_idx + 1..],
        &x[num_lt + num_eq..],
        x_start_idx + num_lt + num_eq,
    );
}

/// Find the number of elements in `x` < `target` and number of elements in `x` == `target`.
/// Formula is (#(`x` < `target`), #(`x` == `target`))
///
/// # Arguments
/// * `x` - dataset to count against
/// * `target` - value to compare against
fn count_lt_eq<TI: PartialOrd>(x: &[TI], target: &TI) -> (usize, usize) {
    if x.is_empty() {
        return (0, 0);
    }
    let (mut lower, mut upper) = (0, x.len());
    let mut eq_upper = upper;

    while upper - lower > 1 {
        let middle = lower + (upper - lower) / 2;

        if &x[middle] < target {
            lower = middle;
        } else {
            upper = middle;
            // tighten eq_upper to last middle where x[middle] was still greater than target
            if &x[middle] > target {
                eq_upper = middle;
            }
        }
    }
    let lt = if &x[lower] < target { upper } else { lower };

    // run another search to find the greatest index equal to target
    // search starting from the first index equal to target
    lower = lt;
    // search for the smallest middle where x[middle] is greater than target
    upper = eq_upper;
    while upper - lower > 1 {
        let middle = lower + (upper - lower) / 2;

        if &x[middle] == target {
            lower = middle;
        } else {
            upper = middle;
        }
    }

    let eq = if lower == upper || &x[lower] == target {
        upper
    } else {
        lower
    } - lt;

    (lt, eq)
}

#[cfg(test)]
mod test;
