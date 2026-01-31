use num::Float;

use crate::{
    measurements::make_noisy_max, measures::ZeroConcentratedDivergence, metrics::SymmetricDistance,
    traits::samplers::sample_uniform_uint_below,
};

use super::*;

#[test]
fn test_quantile_score_candidates_median() -> Fallible<()> {
    let candidates = vec![20, 33, 40, 50, 72, 100];
    let t_qscore = make_quantile_score_candidates(
        VectorDomain::new(AtomDomain::<u32>::default()),
        SymmetricDistance,
        candidates.clone(),
        0.5,
    )?;

    assert_eq!(
        t_qscore.invoke(&(0..100).collect())?,
        vec![59, 33, 19, 1, 45, 100]
    );

    let m_rnm = make_noisy_max(
        t_qscore.output_domain.clone(),
        t_qscore.output_metric.clone(),
        ZeroConcentratedDivergence,
        1.0,
        true,
    )?;

    let m_quantile = (t_qscore >> m_rnm)?;
    let idx = m_quantile.invoke(&(0..100).collect())?;
    assert_eq!(candidates[idx], 50);
    assert_eq!(m_quantile.map(&1)?, 0.5);
    Ok(())
}

#[test]
fn test_quantile_score_candidates_offset() -> Fallible<()> {
    let trans = make_quantile_score_candidates(
        VectorDomain::new(AtomDomain::<u32>::default()),
        SymmetricDistance,
        vec![0, 25, 50, 75, 100],
        0.5,
    )?;
    // score works out to 2 * |50 - cand|
    assert_eq!(trans.invoke(&(0..101).collect())?, [100, 50, 0, 50, 100]);
    assert_eq!(trans.map(&1)?, 1);
    Ok(())
}

#[test]
fn test_validate_candidates() -> Fallible<()> {
    let candidates = vec![1, 2, 3];
    check_candidates(&candidates)?;

    let candidates = vec![1f32, 2f32, 3f32];
    check_candidates(&candidates)?;

    let candidates = vec![2, 1, 3];
    assert!(check_candidates(&candidates).is_err());

    let candidates = vec![2.0, 1.0, 3.0];
    assert!(check_candidates(&candidates).is_err());

    let candidates = vec![1.0, 2.0, f64::NAN];
    assert!(check_candidates(&candidates).is_err());

    let candidates = vec![f64::NAN, 1.0, 2.0];
    assert!(check_candidates(&candidates).is_err());

    Ok(())
}

#[test]
fn test_score_candidates_constants() -> Fallible<()> {
    // pass size, alpha, get alpha decomposition and upper bound for arithmetic
    let (alpha_num, alpha_den, size_limit) = score_candidates_constants(Some(10), 0.5)?;
    assert_eq!(alpha_num, 1);
    assert_eq!(alpha_den, 2);
    assert_eq!(size_limit, 10);

    let (alpha_num, alpha_den, size_limit) = score_candidates_constants(None, 0.5)?;
    assert_eq!(alpha_num, 1);
    assert_eq!(alpha_den, 2);
    assert_eq!(size_limit, u64::MAX / alpha_den);

    let (alpha_num, alpha_den, size_limit) = score_candidates_constants(None, 0.25)?;
    assert_eq!(alpha_num, 1);
    assert_eq!(alpha_den, 4);
    assert_eq!(size_limit, u64::MAX / alpha_den);

    let (alpha_num, alpha_den, size_limit) = score_candidates_constants(None, 2.0 - 2.0.sqrt())?;
    // 2 - sqrt(2) = 0.5857864376
    // take the first 4 digits of the decimal
    assert_eq!(alpha_num, 5857);
    assert_eq!(alpha_den, 10_000);
    assert_eq!(size_limit, u64::MAX / alpha_den);

    let (alpha_num, alpha_den, size_limit) = score_candidates_constants(None, 0.0)?;
    assert_eq!(alpha_num, 0);
    assert_eq!(alpha_den, 1);
    assert_eq!(size_limit, u64::MAX);

    let (alpha_num, alpha_den, size_limit) =
        score_candidates_constants(Some(u64::MAX / 4), 0.34434532242)?;
    // since the size bound is so large, the alpha decomposition is more lossy
    assert_eq!(alpha_num, 1);
    assert_eq!(alpha_den, 4);
    assert_eq!(size_limit, u64::MAX / 4);

    Ok(())
}

#[test]
fn test_score_candidates_function() {
    let c = vec![-1, 2, 4, 7, 12, 22];
    let x = vec![0, 2, 2, 3, 5, 7, 7, 7];
    // RO:       (-inf, -1], (-1, 2], (2, 4], (4, 7], (7, 12], (12, 22], (22, inf)
    // hist_ro = 0           3,       1,      4,      0,       0,        0
    // LO:       (-inf, -1), [-1, 2), [2, 4), [4, 7), [7, 12), [12, 22), [22, inf)
    // hist_lo = 0           1,       3,      1,      3,       0,        0
    let scores: Vec<_> = score_candidates(x.into_iter(), c, 1, 2, 7).collect();
    // lt = 0, le = 0, gt = 8 - 0 = 8
    // |1 * min(0, 7) - 1 * min(8, 7)| = |0 - 7| = 7

    // lt = 1, le = 3, gt = 8 - 3 = 5
    // |1 * min(1, 7) - 1 * min(5, 7)| = |1 - 5| = 4

    // lt = 4, le = 4, gt = 8 - 4 = 4
    // |1 * min(4, 7) - 1 * min(4, 7)| = |4 - 4| = 0

    // lt = 5, le = 8, gt = 8 - 8 = 0
    // |1 * min(5, 7) - 1 * min(0, 7)| = |5 - 0| = 5

    // lt = 8, le = 8, gt = 8 - 8 = 0
    // |1 * min(8, 7) - 1 * min(0, 7)| = |7 - 0| = 7

    // lt = 8, le = 8, gt = 8 - 8 = 0
    // |1 * min(8, 7) - 1 * min(0, 7)| = |7 - 0| = 7
    assert_eq!(scores, vec![7, 4, 0, 5, 7, 7]);
}

#[test]
fn test_scorer_edge_cases() {
    let c = vec![];
    let x = vec![1, 2, 3];
    assert!(Vec::from_iter(score_candidates(x.into_iter(), c, 1, 2, 10)).is_empty());

    let c = vec![1, 2, 3];
    let x = vec![];
    let scores = Vec::from_iter(score_candidates(x.into_iter(), c, 1, 2, 10));
    assert_eq!(scores, vec![0; 3]);
}

#[test]
fn test_score_candidates_stability() -> Fallible<()> {
    // test case one and two in change one model where one record is moved beyond candidate
    let alpha_num = 1;
    let alpha_den = 2;
    let s1 = score_candidates([0, 4].into_iter(), vec![1], alpha_num, alpha_den, 1000)
        .next()
        .unwrap();

    let s2 = score_candidates([4, 4].into_iter(), vec![1], alpha_num, alpha_den, 1000)
        .next()
        .unwrap();
    // score changes by alpha_den
    assert_eq!(s1, s2 - alpha_den);

    // test case three and four in change one model where one record is replaced with candidate
    let s3 = score_candidates([1, 4].into_iter(), vec![1], alpha_num, alpha_den, 1000)
        .next()
        .unwrap();
    // score changes by alpha_den
    assert_eq!(s1, s3 - alpha_num.max(alpha_den - alpha_num));
    Ok(())
}

#[test]
fn test_score_candidates_map() -> Fallible<()> {
    // d_in * max(alpha_num, alpha_den - alpha_num) = 1 * max(1, 2 - 1) = 1
    assert_eq!(score_candidates_map(1, 2, false)(&1)?, 1);
    assert_eq!(score_candidates_map(1, 2, false)(&2)?, 2);
    // d_in * max(alpha_num, alpha_den - alpha_num) = 1 * max(2, 3 - 2) = 2
    assert_eq!(score_candidates_map(2, 3, false)(&1)?, 2);

    // d_in // 2 * alpha_den = 1 // 2 * 2 = 0
    assert_eq!(score_candidates_map(1, 2, true)(&1)?, 0);
    // d_in // 2 * alpha_den = 2 // 2 * 2 = 2
    assert_eq!(score_candidates_map(1, 2, true)(&2)?, 2);
    assert_eq!(score_candidates_map(1, 2, true)(&3)?, 2);
    // d_in // 2 * alpha_den = 1 // 2 * 3 = 3
    assert_eq!(score_candidates_map(2, 3, true)(&2)?, 3);
    Ok(())
}

#[cfg(feature = "derive")]
mod integration_tests {
    use crate::{measurements::make_noisy_max, metrics::SymmetricDistance};

    use super::*;

    #[test]
    fn test_scorer() -> Fallible<()> {
        let candidates = vec![7, 12, 14, 72, 76];
        let input_domain = VectorDomain::new(AtomDomain::default());
        let input_metric = SymmetricDistance;
        let trans =
            make_quantile_score_candidates(input_domain, input_metric, candidates.clone(), 0.75)?;

        let input_domain = VectorDomain::new(AtomDomain::default()).with_size(100);
        let input_metric = SymmetricDistance;
        let trans_sized =
            make_quantile_score_candidates(input_domain, input_metric, candidates, 0.75)?;

        let _scores = trans.invoke(&(0..100).collect())?;
        let _scores_sized = trans_sized.invoke(&(0..100).collect())?;

        // because alpha is .75 = 3 / 4, sensitivity is d_in * max(α_num, α_den - α_num) = 3
        assert_eq!(trans.map(&1)?, 3);

        // sensitivity is d_in * α_den
        assert_eq!(trans_sized.map(&2)?, 4);

        Ok(())
    }

    #[test]
    fn test_release() -> Fallible<()> {
        let candidates = vec![7, 12, 14, 72, 76];
        let input_domain = VectorDomain::new(AtomDomain::default());
        let input_metric = SymmetricDistance;
        let trans = make_quantile_score_candidates(input_domain, input_metric, candidates, 0.75)?;
        let exp_mech = make_noisy_max(
            trans.output_domain.clone(),
            trans.output_metric.clone(),
            ZeroConcentratedDivergence,
            trans.map(&1)? as f64 * 2.,
            true,
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
        let input_metric = SymmetricDistance;
        let trans_sized =
            make_quantile_score_candidates(input_domain, input_metric, candidates, 0.75)?;
        let exp_mech = make_noisy_max(
            trans_sized.output_domain.clone(),
            trans_sized.output_metric.clone(),
            ZeroConcentratedDivergence,
            trans_sized.map(&2)? as f64 * 2.,
            true,
        )?;

        let quantile_sized_meas = (trans_sized >> exp_mech)?;
        let idx = quantile_sized_meas.invoke(&(0..100).collect())?;
        println!("idx sized {:?}", idx);
        assert!(quantile_sized_meas.check(&1, &1.)?);

        Ok(())
    }
}

mod alternative_impl {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn test_alternative_impl_is_equivalent() -> Fallible<()> {
        // There are two independent implementations of score_candidates that should be equivalent.
        // If this test fails, then at least one of the implementations is bugged.
        let data_max = 100_000u32;
        let size_lim = 10_000u64;

        let mut candidates = (0..1_000)
            .map(|_| sample_uniform_uint_below(data_max))
            .collect::<Fallible<Vec<u32>>>()?;
        candidates.sort_unstable();
        candidates.dedup();

        let x: Vec<u32> = (0..size_lim)
            .map(|_| sample_uniform_uint_below(data_max))
            .collect::<Fallible<Vec<u32>>>()?;

        let alpha_den = sample_uniform_uint_below(100_000)?;
        let alpha_num = sample_uniform_uint_below(alpha_den)?;

        let scores1: Vec<_> = score_candidates(
            x.clone().into_iter(),
            candidates.clone(),
            alpha_num,
            alpha_den,
            size_lim,
        )
        .collect();

        let scores2: Vec<_> =
            score_candidates2(x.into_iter(), candidates, alpha_num, alpha_den, size_lim).collect();

        assert_eq!(scores1, scores2);
        Ok(())
    }

    /// Shares the same signature as score_candidates, but uses a different algorithm.
    /// Runtime is O(n * ln(n)) for the sort phase, and O(ln(n) * c) for the score phase,
    /// where n is the size of the input vector and c is the number of candidates.
    /// Therefore, if n is already sorted, then overall runtime is O(ln(n) * c), otherwise O(n * ln(n)).
    ///
    /// score_candidates is O(ln(c) * n), which when data is not sorted, is faster for large n.
    pub(crate) fn score_candidates2<TIA: 'static + PartialOrd>(
        x: impl Iterator<Item = TIA>,
        candidates: Vec<TIA>,
        alpha_num: u64,
        alpha_den: u64,
        size_limit: u64,
    ) -> impl Iterator<Item = u64> {
        let alpha_num = alpha_num as usize;
        let alpha_den = alpha_den as usize;
        let size_limit = size_limit as usize;
        let mut x = x.collect::<Vec<_>>();
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
            .map(move |(lt, eq)| {
                // |α_den * #(x < c) - α_num * (n - #(x = c))|
                (alpha_den * lt.min(size_limit))
                    .abs_diff(alpha_num * (x.len() - eq).min(size_limit)) as u64
            })
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
}
