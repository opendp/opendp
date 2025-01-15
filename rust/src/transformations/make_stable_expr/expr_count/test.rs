use super::*;

use crate::{
    domains::{LazyFrameDomain, Margin, OptionDomain},
    metrics::{InsertDeleteDistance, L1Distance, L2Distance, SymmetricDistance},
};

use polars::{
    df,
    prelude::{col, IntoLazy},
};

#[test]
fn test_select_make_expr_counting() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "data",
        OptionDomain::new(AtomDomain::<i32>::default()),
    )])?;

    let lf = df!["data" => [Some(1i32), Some(2i32), None]]?.lazy();
    let expr_domain = lf_domain.select();

    let exprs = vec![
        (col("data").null_count(), 1),
        (col("data").count(), 2),
        (col("data").len(), 3),
        (col("data").n_unique(), 3),
    ];
    for (expr, expected) in exprs {
        let t_sum: Transformation<_, _, _, L1Distance<f64>> = expr
            .clone()
            .make_stable(expr_domain.clone(), PartitionDistance(SymmetricDistance))?;
        let expr_res = t_sum.invoke(&lf.logical_plan)?.expr;
        assert_eq!(expr_res, expr);

        let sensitivity = t_sum.map(&(1, 2, 2))?;
        println!("sens: {:?}", sensitivity);
        assert_eq!(sensitivity, 2.);
        assert_eq!(
            lf.clone().select([expr]).collect()?,
            df!["data" => [expected]]?
        );
    }
    Ok(())
}

#[test]
fn test_grouped_make_len_expr() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("data", OptionDomain::new(AtomDomain::<i32>::default())),
        SeriesDomain::new("by", AtomDomain::<i32>::default()),
    ])?;

    let lf = df![
        "data" => [Some(1i32), Some(2i32), None, Some(1i32), None],
        "by" => [1, 1, 1, 2, 2]
    ]?
    .lazy();
    let expr_domain = lf_domain.aggregate(["by"]);

    let exprs = vec![
        (col("data").null_count(), [1, 1]),
        (col("data").count(), [2, 1]),
        (col("data").len(), [3, 2]),
        (col("data").n_unique(), [3, 2]),
    ];
    for (expr, expected) in exprs {
        let t_sum: Transformation<_, _, _, L2Distance<f64>> = expr
            .clone()
            .make_stable(expr_domain.clone(), PartitionDistance(SymmetricDistance))?;
        let expr_res = t_sum.invoke(&lf.logical_plan)?.expr;
        assert_eq!(expr_res, expr);

        // assume we're in a grouping context.
        // By the following triple, we know
        // 1. an individual can influence up to 10 partitions (l0)
        // 2. an individual can contribute up to 10 records total (l1)
        // 3. an individual can contribute at most 1 record to any partition (linf)
        let sensitivity = t_sum.map(&(10, 10, 1))?;

        // The sensitivity d_out under the l2 distance in unbounded DP is given by the following formula:
        // = min(sqrt(l0) * map(linf)         , map(l1))
        // = min(sqrt(l0) * linf * max(|L|, U), l1 * max(|L|, U))
        // = min(sqrt(10) * 1, 10)
        // = min(3.16227, 10)
        // = 3.16227

        // that is, in the worst case, we know the sum will differ by at most 1 in 10 partitions,
        // so the l2 distance between any two outputs on neighboring data sets is at most 3.16227

        // The sensitivity is slightly higher to account for potential rounding errors.
        println!("sens: {:?}", sensitivity);
        assert!(sensitivity > (3.16227).into());
        assert!(sensitivity < (3.162278).into());

        let actual = lf
            .clone()
            .group_by(["by"])
            .agg([expr])
            .collect()?
            .sort(["by"], Default::default())?;
        assert_eq!(actual, df!["by" => [1, 2], "data" => expected]?);
    }
    Ok(())
}

#[test]
fn test_select_make_expr_count_row_by_row() -> Fallible<()> {
    // this transformation should refuse to build in a row-by-row context like `with_columns`
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "data",
        AtomDomain::<i32>::default(),
    )])?;
    let expr_domain = lf_domain.row_by_row();

    assert!(col("data")
        .count()
        .make_stable(expr_domain, InsertDeleteDistance)
        .map(|_: Transformation<_, _, _, InsertDeleteDistance>| ())
        .is_err());

    Ok(())
}

#[test]
fn test_expr_count_public_info() -> Fallible<()> {
    // this transformation should refuse to build in a row-by-row context like `with_columns`
    let series_domain = SeriesDomain::new("data", AtomDomain::<i32>::default());
    let lf_domain = LazyFrameDomain::new(vec![series_domain])?
        .with_margin::<&str>(&[], Margin::default().with_public_lengths())?;

    let t_count: Transformation<_, _, _, L2Distance<f64>> = col("data").count().make_stable(
        lf_domain.clone().select(),
        PartitionDistance(InsertDeleteDistance),
    )?;
    assert_eq!(t_count.map(&(10, 10, 1))?, 0.);

    let t_len: Transformation<_, _, _, L2Distance<f64>> = col("data").len().make_stable(
        lf_domain.clone().select(),
        PartitionDistance(InsertDeleteDistance),
    )?;

    assert_eq!(t_len.map(&(10, 10, 1))?, 0.);

    let t_null_count: Transformation<_, _, _, L2Distance<f64>> =
        col("data").null_count().make_stable(
            lf_domain.clone().select(),
            PartitionDistance(InsertDeleteDistance),
        )?;

    assert_ne!(t_null_count.map(&(10, 10, 1))?, 0.);

    Ok(())
}
