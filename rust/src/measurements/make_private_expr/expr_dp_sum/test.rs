use polars::prelude::*;

use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::{L0PInfDistance, SymmetricDistance},
    polars::PrivacyNamespace,
};

use super::make_expr_dp_sum;

#[test]
fn test_signed_u32_sum() -> Fallible<()> {
    let lf = df!("value" => &[1u32, 2, 3])?.lazy();
    let input_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "value",
        AtomDomain::<u32>::default(),
    )])?
    .with_margin(Margin::select().with_max_length(3))?;

    let measurement = make_expr_dp_sum(
        input_domain.select(),
        L0PInfDistance(SymmetricDistance),
        MaxDivergence,
        col("value")
            .dp()
            .sum((lit(0u32), lit(3u32)), Some(0.0), true),
        None,
    )?;
    let expr = measurement.invoke(&lf.logical_plan)?.expr;
    let result = lf.select([expr]).collect()?;

    assert_eq!(result.column("value")?.dtype(), &DataType::Int64);
    assert_eq!(result.column("value")?.i64()?.get(0), Some(6));
    Ok(())
}

#[test]
fn test_signed_u64_sum_clips_before_cast() -> Fallible<()> {
    let max = i64::MAX as u64;
    let lf = df!("value" => &[max, 1u64])?.lazy();
    let input_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "value",
        AtomDomain::<u64>::default(),
    )])?
    .with_margin(Margin::select().with_max_length(2))?;

    let measurement = make_expr_dp_sum(
        input_domain.select(),
        L0PInfDistance(SymmetricDistance),
        MaxDivergence,
        col("value")
            .dp()
            .sum((lit(0u64), lit(max)), Some(0.0), true),
        None,
    )?;
    let expr = measurement.invoke(&lf.logical_plan)?.expr;
    let result = lf.select([expr]).collect()?;

    assert_eq!(result.column("value")?.dtype(), &DataType::Int64);
    assert_eq!(result.column("value")?.i64()?.get(0), Some(i64::MAX));
    Ok(())
}
