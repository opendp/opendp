use std::cmp::Ordering;

use opendp_derive::bootstrap;

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::LInfDistance,
    traits::{AlertingMul, ExactIntCast, InfDiv, Number, RoundCast},
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

    if candidates.windows(2).any(|w| w[0] >= w[1]) {
        return fallible!(MakeTransformation, "candidates must be increasing");
    }

    let alpha_den = if let Some(size) = input_domain.size {
        // choose the finest granularity that won't overflow
        // must have that size * denom < MAX, so let denom = MAX // size
        (usize::MAX).neg_inf_div(&size)?
    } else {
        // default to an alpha granularity of .00001
        10_000
    };
    // numer = alpha * denom
    let alpha_num = usize::round_cast(alpha * f64::round_cast(alpha_den.clone())?)?;
    if alpha_num > alpha_den || alpha_den == 0 {
        return fallible!(MakeTransformation, "alpha must be within [0, 1]");
    }

    let size_limit = if let Some(size_limit) = input_domain.size {
        // ensures that there is no overflow
        size_limit.alerting_mul(&alpha_den)?;
        size_limit
    } else {
        (usize::MAX).neg_inf_div(&alpha_den)?
    };

    let stability_map = if input_domain.size.is_some() {
        StabilityMap::new_fallible(move |d_in| {
            usize::exact_int_cast(d_in / 2)?
                .alerting_mul(&2)?
                .alerting_mul(&alpha_den)
        })
    } else {
        let abs_dist_const = alpha_num.max(alpha_den - alpha_num);
        StabilityMap::new_from_constant(abs_dist_const)
    };

    Transformation::<_, VectorDomain<AtomDomain<usize>>, _, _>::new(
        input_domain,
        VectorDomain::default().with_size(candidates.len()),
        Function::new(move |arg: &Vec<TIA>| {
            compute_score(arg.clone(), &candidates, alpha_num, alpha_den, size_limit)
        }),
        input_metric,
        LInfDistance::new(true),
        stability_map,
    )
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
fn compute_score<TIA: PartialOrd>(
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

#[cfg(all(test, feature = "derive"))]
mod test_trans {
    use crate::{
        measurements::{make_report_noisy_max_gumbel, Optimize},
        metrics::SymmetricDistance,
    };

    use super::*;

    #[test]
    fn test_scorer() -> Fallible<()> {
        let candidates = vec![7, 12, 14, 72, 76];
        let input_domain = VectorDomain::new(AtomDomain::default());
        let input_metric = SymmetricDistance::default();
        let trans =
            make_quantile_score_candidates(input_domain, input_metric, candidates.clone(), 0.75)?;

        let input_domain = VectorDomain::new(AtomDomain::default()).with_size(100);
        let input_metric = SymmetricDistance::default();
        let trans_sized =
            make_quantile_score_candidates(input_domain, input_metric, candidates, 0.75)?;

        let _scores = trans.invoke(&(0..100).collect())?;
        let _scores_sized = trans_sized.invoke(&(0..100).collect())?;

        // because alpha is .75, sensitivity is 1.5 (because not monotonic)
        // granularity of quantile is .00001, so scores are integerized at a scale of 10000x
        assert_eq!(trans.map(&1)?, 7500);

        // alpha does not affect sensitivity- it's solely based on the size of the input domain
        // using all of the range of the usize,
        //     so scores can be scaled up by a factor of usize::MAX / 100 before being converted to integers and not overflow
        // factor of 4 breaks into:
        //   * a factor of 2 from non-monotonicity
        //   * a factor of 2 from difference in score after moving one record from above to below a candidate
        assert_eq!(trans_sized.map(&2)?, usize::MAX / 100 * 2);

        Ok(())
    }

    #[test]
    fn test_release() -> Fallible<()> {
        let candidates = vec![7, 12, 14, 72, 76];
        let input_domain = VectorDomain::new(AtomDomain::default());
        let input_metric = SymmetricDistance::default();
        let trans = make_quantile_score_candidates(input_domain, input_metric, candidates, 0.75)?;
        let exp_mech = make_report_noisy_max_gumbel(
            trans.output_domain.clone(),
            trans.output_metric.clone(),
            trans.map(&1)? as f64 * 2.,
            Optimize::Min,
        )?;

        let quantile_meas = (trans >> exp_mech)?;
        let idx = quantile_meas.invoke(&(0..100).collect())?;
        println!("idx {:?}", idx);
        assert!(quantile_meas.check(&1, &1.)?);
        Ok(())
    }

    #[test]
    fn test_release_sized() -> Fallible<()> {
        let candidates = vec![7, 12, 14, 72, 76];
        let input_domain = VectorDomain::new(AtomDomain::default()).with_size(100);
        let input_metric = SymmetricDistance::default();
        let trans_sized =
            make_quantile_score_candidates(input_domain, input_metric, candidates, 0.75)?;
        let exp_mech = make_report_noisy_max_gumbel(
            trans_sized.output_domain.clone(),
            trans_sized.output_metric.clone(),
            trans_sized.map(&2)? as f64 * 2.,
            Optimize::Min,
        )?;

        let quantile_sized_meas = (trans_sized >> exp_mech)?;
        let idx = quantile_sized_meas.invoke(&(0..100).collect())?;
        println!("idx sized {:?}", idx);
        assert!(quantile_sized_meas.check(&1, &1.)?);

        Ok(())
    }
}
