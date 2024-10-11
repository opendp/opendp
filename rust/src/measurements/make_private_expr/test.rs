use polars::prelude::{col, lit};

use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    metrics::{InsertDeleteDistance, L2Distance},
    transformations::StableExpr,
};

use super::*;

#[test]
fn test_approximate_c_stability_unbounded() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?
            .with_margin::<&str>(&[], Margin::default().with_max_partition_length(100))?;
    let expr_domain = lf_domain.select();

    // Get resulting sum (expression result)
    let t_sum: Transformation<_, _, _, L2Distance<f64>> = col("A")
        .clip(lit(0), lit(2))
        .sum()
        .make_stable(expr_domain, PartitionDistance(InsertDeleteDistance))?;

    assert_eq!(approximate_c_stability(&t_sum)?, 2.0);
    Ok(())
}

#[test]
fn test_approximate_c_stability_bounded() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?
            .with_margin::<&str>(
                &[],
                Margin::default()
                    .with_max_partition_length(100)
                    .with_public_lengths(),
            )?;
    let expr_domain = lf_domain.select();

    // Get resulting sum (expression result)
    let t_sum: Transformation<_, _, _, L2Distance<f64>> = col("A")
        .clip(lit(4), lit(7))
        .sum()
        .make_stable(expr_domain, PartitionDistance(InsertDeleteDistance))?;

    assert_eq!(approximate_c_stability(&t_sum)?, 3.0);
    Ok(())
}
