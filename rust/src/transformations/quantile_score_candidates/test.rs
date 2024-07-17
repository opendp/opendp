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

#[cfg(feature = "derive")]
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

        // because alpha is .75 = 3 / 4, sensitivity is d_in * max(α_num, α_den - α_num) = 3
        assert_eq!(trans.map(&1)?, 3);

        // sensitivity is d_in * α_den
        assert_eq!(trans_sized.map(&2)?, 8);

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
