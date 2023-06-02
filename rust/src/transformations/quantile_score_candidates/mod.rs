use std::cmp::Ordering;

use opendp_derive::bootstrap;

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::LInfDiffDistance,
    traits::{AlertingMul, ExactIntCast, Float, InfDiv, Number, RoundCast},
};

use super::ARDatasetMetric;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    generics(TIA(suppress), MI(suppress)),
    derived_types(TIA = "$get_atom(get_type(input_domain))")
)]
/// Makes a Transformation that scores how similar each candidate is to the given `alpha`-quantile on the input dataset.
///
///
/// # Arguments
/// * `input_domain` - Uses a tighter sensitivity when the size of vectors in the input domain is known.
/// * `input_metric` - Either SymmetricDistance or InsertDeleteDistance.
/// * `candidates` - Potential quantiles to score
/// * `alpha` - a value in [0, 1]. Choose 0.5 for median
///
/// # Generics
/// * `TIA` - Atomic Input Type. Type of elements in the input vector
/// * `A` - Alpha type. Can be a (numer, denom) tuple, or float.
/// * `MI` - Input Metric.
pub fn make_quantile_score_candidates<TIA: Number, A: IntoFrac, MI: ARDatasetMetric>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: MI,
    candidates: Vec<TIA>,
    alpha: A,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        VectorDomain<AtomDomain<usize>>,
        MI,
        LInfDiffDistance<usize>,
    >,
>
where
    (VectorDomain<AtomDomain<TIA>>, MI): MetricSpace,
    (VectorDomain<AtomDomain<usize>>, LInfDiffDistance<usize>): MetricSpace,
{
    if input_domain.element_domain.nullable() {
        return fallible!(MakeTransformation, "input must be non-null");
    }

    if candidates.windows(2).any(|w| w[0] >= w[1]) {
        return fallible!(MakeTransformation, "candidates must be increasing");
    }

    let (alpha_num, alpha_den) = alpha.into_frac(input_domain.size)?;
    if alpha_num > alpha_den || alpha_den <= 0 {
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
                .alerting_mul(&4)?
                .alerting_mul(&alpha_den)
        })
    } else {
        let abs_dist_const = alpha_num.max(alpha_den - alpha_num);
        let inf_diff_dist_const = abs_dist_const.alerting_mul(&2)?;
        StabilityMap::new_from_constant(inf_diff_dist_const)
    };

    Transformation::<_, VectorDomain<AtomDomain<usize>>, _, _>::new(
        input_domain,
        VectorDomain::new(AtomDomain::<usize>::default()).with_size(size_limit),
        Function::new(move |arg: &Vec<TIA>| {
            compute_score(arg.clone(), &candidates, alpha_num, alpha_den, size_limit)
        }),
        input_metric,
        LInfDiffDistance::default(),
        stability_map,
    )
}

pub trait IntoFrac: 'static {
    fn into_frac(self, size: Option<usize>) -> Fallible<(usize, usize)>;
}
impl<T: 'static> IntoFrac for (T, T)
where
    usize: ExactIntCast<T>,
{
    fn into_frac(self, _size: Option<usize>) -> Fallible<(usize, usize)> {
        Ok((
            usize::exact_int_cast(self.0)?,
            usize::exact_int_cast(self.1)?,
        ))
    }
}

impl<F: Float + RoundCast<usize>> IntoFrac for F
where
    usize: RoundCast<F>,
{
    fn into_frac(self, size: Option<usize>) -> Fallible<(usize, usize)> {
        let denom = if let Some(size) = size {
            // choose the finest granularity that won't overflow
            // must have that size * denom < MAX, so let denom = MAX // size
            (usize::MAX).neg_inf_div(&size)?
        } else {
            // default to an alpha granularity of .0001
            10_000
        };
        // numer = alpha * denom
        let numer = usize::round_cast(self * F::round_cast(denom.clone())?)?;
        Ok((numer, denom))
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
/// * `alpha_numer` - numerator of alpha fraction
/// * `alpha_denom` - alpha is parameter for quantile. {0: min, 0.5: median, 1: max, ...}
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

    // compute #(`x` <= c) for each c in candidates
    let mut num_lt = vec![0; candidates.len()];
    let mut num_eq = vec![0; candidates.len()];
    count_lt_eq_recursive(
        num_lt.as_mut_slice(),
        num_eq.as_mut_slice(),
        candidates.as_slice(),
        x.as_slice(),
        0,
    );

    // now that we have num_lte and num_eq, score all candidates
    num_lt
        .into_iter()
        .zip(num_eq.into_iter())
        // score function cannot overflow.
        //     lt <= size_limit, so 0 <= alpha_denom * lt <= usize::MAX
        //     n - eq <= size_limit, so 0 <= size_limit - eq
        .map(|(lt, eq)| {
            (alpha_den * lt.min(size_limit)).abs_diff(alpha_num * (x.len() - eq).min(size_limit))
        })
        .collect()
}

/// Compute number of elements less than each edge
/// Formula is #(x <= e) for each e in `edges`.
///
/// # Arguments
/// * `counts` - location to write the result
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
    // use binary search to find |{i; x[i] < middle edge}|
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
    // println!("lower {:?}, upper {:?}", lower, upper);
    // println!("len {:?}", x.len());

    let eq = if lower == upper || &x[lower] == target {
        upper
    } else {
        lower
    } - lt;

    // println!("lt {:?}, eq {:?}", lt, eq);
    (lt, eq)
}

#[cfg(test)]
mod test_scorer {
    use super::*;

    #[test]
    fn test_count_lte() {
        let x = (5..20).collect::<Vec<i32>>();
        let edges = vec![2, 4, 7, 12, 22];
        let mut count_lt = vec![0; edges.len()];
        let mut count_eq = vec![0; edges.len()];
        count_lt_eq_recursive(
            count_lt.as_mut_slice(),
            count_eq.as_mut_slice(),
            edges.as_slice(),
            x.as_slice(),
            0,
        );
        println!("{:?}", count_lt);
        println!("{:?}", count_eq);
        assert_eq!(count_lt, vec![0, 0, 2, 7, 15]);
        assert_eq!(count_eq, vec![0, 0, 1, 1, 0]);
    }

    #[test]
    fn test_count_lte_repetition() {
        let x = vec![0, 2, 2, 3, 5, 7, 7, 7];
        let edges = vec![-1, 2, 4, 7, 12, 22];
        let mut count_lt = vec![0; edges.len()];
        let mut count_eq = vec![0; edges.len()];
        count_lt_eq_recursive(
            count_lt.as_mut_slice(),
            count_eq.as_mut_slice(),
            edges.as_slice(),
            x.as_slice(),
            0,
        );
        println!("{:?}", count_lt);
        println!("{:?}", count_eq);
        assert_eq!(count_lt, vec![0, 1, 4, 5, 8, 8]);
        assert_eq!(count_eq, vec![0, 2, 0, 3, 0, 0]);
    }

    fn test_case(x: Vec<i32>, edges: Vec<i32>, true_lt: Vec<usize>, true_eq: Vec<usize>) {
        let mut count_lt = vec![0; edges.len()];
        let mut count_eq = vec![0; edges.len()];
        count_lt_eq_recursive(
            count_lt.as_mut_slice(),
            count_eq.as_mut_slice(),
            edges.as_slice(),
            x.as_slice(),
            0,
        );
        println!("LT");
        println!("{:?}", true_lt);
        println!("{:?}", count_lt);

        println!("EQ");
        println!("{:?}", true_eq);
        println!("{:?}", count_eq);

        assert_eq!(true_lt, count_lt);
        assert_eq!(true_eq, count_eq);
    }

    #[test]
    fn test_count_lte_edge_cases() {
        // check constant x
        test_case(vec![0; 10], vec![-1], vec![0], vec![0]);
        test_case(vec![0; 10], vec![0], vec![0], vec![10]);
        test_case(vec![0; 10], vec![1], vec![10], vec![0]);

        // below first split
        test_case(vec![1, 2, 3, 3, 3, 3], vec![2], vec![1], vec![1]);
        test_case(vec![1, 2, 3, 3, 3, 3, 3], vec![2], vec![1], vec![1]);
        // above first split
        test_case(vec![1, 1, 1, 1, 2, 3], vec![2], vec![4], vec![1]);
        test_case(vec![1, 1, 1, 1, 2, 3, 3], vec![2], vec![4], vec![1]);
    }

    #[test]
    fn test_scorer() -> Fallible<()> {
        let edges = vec![-1, 2, 4, 7, 12, 22];

        let x = vec![0, 2, 2, 3, 5, 7, 7, 7];
        let scores = compute_score(x, &edges, 1, 2, 8);
        println!("{:?}", scores);

        let x = vec![0, 2, 2, 3, 4, 7, 7, 7];
        let scores = compute_score(x, &edges, 1, 2, 8);
        println!("{:?}", scores);
        Ok(())
    }
}

// feature-gated because non-mpfr InfCast errors on numbers greater than 2^52
#[cfg(all(test, feature = "use-mpfr"))]
mod test_trans {
    use crate::metrics::SymmetricDistance;

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

        println!("      map: {:?}", trans.map(&1)?);
        println!("sized map: {:?}", trans_sized.map(&2)?);

        Ok(())
    }
}
