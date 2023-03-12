use std::cmp::Ordering;

use opendp_derive::bootstrap;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AtomDomain, SizedDomain, VectorDomain},
    error::Fallible,
    metrics::{InfDifferenceDistance, SymmetricDistance},
    traits::{AlertingMul, ExactIntCast, Float, InfDiv, Number, RoundCast},
};

#[cfg(feature = "ffi")]
mod ffi;

pub trait IntoFrac {
    fn into_frac(self, size: Option<usize>) -> Fallible<(usize, usize)>;
}
impl<T> IntoFrac for (T, T)
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

#[bootstrap(features("contrib"))]
/// Makes a Transformation that scores how similar each candidate is to the given `alpha`-quantile on the input dataset.
///
/// # Arguments
/// * `candidates` - Potential quantiles to score
/// * `alpha` - a value in [0, 1]. Choose 0.5 for median
///
/// # Generics
/// * `TIA` - Atomic Input Type. Type of elements in the input vector
/// * `TOA` - Atomic Output Type. Type of elements in the score vector
pub fn make_quantile_score_candidates<TIA: Number, F: IntoFrac>(
    candidates: Vec<TIA>,
    alpha: F,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        VectorDomain<AtomDomain<usize>>,
        SymmetricDistance,
        InfDifferenceDistance<usize>,
    >,
> {
    if candidates.windows(2).any(|w| w[0] >= w[1]) {
        return fallible!(MakeTransformation, "candidates must be increasing");
    }
    let (alpha_num, alpha_den) = alpha.into_frac(None)?;
    if alpha_num > alpha_den || alpha_den == 0 {
        return fallible!(MakeTransformation, "alpha must be within [0, 1]");
    }

    let size_limit = (usize::MAX).neg_inf_div(&alpha_den)?;

    let abs_dist_const = alpha_num.max(alpha_den - alpha_num);
    let inf_diff_dist_const = abs_dist_const.alerting_mul(&2)?;

    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new(move |arg: &Vec<TIA>| {
            score(arg.clone(), &candidates, alpha_num, alpha_den, size_limit)
        }),
        SymmetricDistance::default(),
        InfDifferenceDistance::default(),
        StabilityMap::new_from_constant(inf_diff_dist_const),
    ))
}

#[bootstrap(features("contrib"))]
/// Makes a Transformation that scores how similar each candidate is to the given `alpha`-quantile on the input dataset.
///
/// # Arguments
/// * `size` - Number of elements in the input dataset
/// * `candidates` - Potential quantiles to score
/// * `alpha` - a value in [0, 1]. Choose 0.5 for median
///
/// # Generics
/// * `TIA` - Atomic Input Type. Type of elements in the input vector
/// * `TOA` - Atomic Output Type (Integer). Type of elements in the score vector
pub fn make_sized_quantile_score_candidates<TIA: Number, F: IntoFrac>(
    size: usize,
    candidates: Vec<TIA>,
    alpha: F,
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<AtomDomain<TIA>>>,
        VectorDomain<AtomDomain<usize>>,
        SymmetricDistance,
        InfDifferenceDistance<usize>,
    >,
> {
    if candidates.windows(2).any(|w| w[0] >= w[1]) {
        return fallible!(MakeTransformation, "candidates must be increasing");
    }
    let (alpha_num, alpha_den) = alpha.into_frac(Some(size))?;
    if alpha_num > alpha_den || alpha_den == 0 {
        return fallible!(MakeTransformation, "alpha must be within [0, 1]");
    }

    // ensures that there is no overflow
    size.alerting_mul(&alpha_den)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new_all(), size),
        VectorDomain::new_all(),
        Function::new(move |arg: &Vec<TIA>| {
            score(arg.clone(), &candidates, alpha_num, alpha_den, size)
        }),
        SymmetricDistance::default(),
        InfDifferenceDistance::default(),
        StabilityMap::new_fallible(move |d_in| {
            usize::exact_int_cast(d_in / 2)?
                .alerting_mul(&4)?
                .alerting_mul(&alpha_den)
        }),
    ))
}

/// Compute score of each candidate on a dataset
/// Formula is -|#(x < c) - alpha * (n - #(x = c))| for each c in `candidates`.
/// Can be understood as -|observed_value - ideal_value|.
///     We want greater scores when observed value is near ideal value.
///     The further away the observed value is from the ideal value, the more negative it gets
///
/// # Arguments
/// * `x` - dataset to score against. Must be non-null
/// * `candidates` - values to be scored. Must be sorted
/// * `alpha` - parameter for quantile. {0: min, 0.5: median, 1: max, ...}
///
/// # Returns
/// Score of each candidate
fn score<TIA: PartialOrd>(
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
        let scores = score(x, &edges, 1, 2, 8);
        println!("{:?}", scores);

        let x = vec![0, 2, 2, 3, 4, 7, 7, 7];
        let scores = score(x, &edges, 1, 2, 8);
        println!("{:?}", scores);
        Ok(())
    }
}

// feature-gated because non-mpfr InfCast errors on numbers greater than 2^52
#[cfg(all(test, feature = "use-mpfr"))]
mod test_trans {
    use crate::measurements::{make_base_discrete_exponential, Optimize};

    use super::*;

    #[test]
    fn test_scorer() -> Fallible<()> {
        let candidates = vec![7, 12, 14, 72, 76];
        let trans = make_quantile_score_candidates(candidates.clone(), 0.75)?;
        let trans_sized = make_sized_quantile_score_candidates(100, candidates, 0.75)?;

        let _scores = trans.invoke(&(0..100).collect())?;
        let _scores_sized = trans_sized.invoke(&(0..100).collect())?;

        println!("      map: {:?}", trans.map(&1)?);
        println!("sized map: {:?}", trans_sized.map(&2)?);

        Ok(())
    }

    #[test]
    fn test_release() -> Fallible<()> {
        let candidates = vec![7, 12, 14, 72, 76];
        let trans = make_quantile_score_candidates(candidates, 0.75)?;
        let exp_mech = make_base_discrete_exponential(trans.map(&1)? as f64, Optimize::Min)?;

        let quantile_meas = (trans >> exp_mech.clone())?;
        let idx = quantile_meas.invoke(&(0..100).collect())?;
        println!("idx {:?}", idx);
        assert!(quantile_meas.check(&1, &1.)?);
        Ok(())
    }

    #[test]
    fn test_release_sized() -> Fallible<()> {
        let candidates = vec![7, 12, 14, 72, 76];
        let trans_sized = make_sized_quantile_score_candidates(100, candidates, 0.75)?;
        let exp_mech = make_base_discrete_exponential(trans_sized.map(&2)? as f64, Optimize::Min)?;

        let quantile_sized_meas = (trans_sized >> exp_mech)?;
        let idx = quantile_sized_meas.invoke(&(0..100).collect())?;
        println!("idx sized {:?}", idx);
        assert!(quantile_sized_meas.check(&1, &1.)?);

        Ok(())
    }
}
