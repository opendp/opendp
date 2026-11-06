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

    for (signed, expected_dtype) in [
        (None, DataType::Int64),
        (Some(true), DataType::Int64),
        (Some(false), DataType::UInt32),
    ] {
        let measurement = make_expr_dp_sum(
            input_domain.clone().select(),
            L0PInfDistance(SymmetricDistance),
            MaxDivergence,
            col("value")
                .dp()
                .sum((lit(0u32), lit(3u32)), Some(0.0), signed),
            None,
        )?;
        let expr = measurement.invoke(&lf.logical_plan)?.expr;
        let result = lf.clone().select([expr]).collect()?;

        assert_eq!(result.column("value")?.dtype(), &expected_dtype);
        assert_eq!(
            result
                .column("value")?
                .cast(&DataType::Int64)?
                .i64()?
                .get(0),
            Some(6)
        );
    }
    Ok(())
}

#[test]
fn test_u64_sum_retains_full_range() -> Fallible<()> {
    let max = i64::MAX as u64;
    let lf = df!("value" => &[max, 1u64])?.lazy();
    let input_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "value",
        AtomDomain::<u64>::default(),
    )])?
    .with_margin(Margin::select().with_max_length(2))?;

    for signed in [None, Some(false)] {
        let measurement = make_expr_dp_sum(
            input_domain.clone().select(),
            L0PInfDistance(SymmetricDistance),
            MaxDivergence,
            col("value")
                .dp()
                .sum((lit(0u64), lit(max)), Some(0.0), signed),
            None,
        )?;
        let expr = measurement.invoke(&lf.logical_plan)?.expr;
        let result = lf.clone().select([expr]).collect()?;
        assert_eq!(result.column("value")?.dtype(), &DataType::UInt64);
        assert_eq!(result.column("value")?.u64()?.get(0), Some(max + 1));
    }

    let error = make_expr_dp_sum(
        input_domain.select(),
        L0PInfDistance(SymmetricDistance),
        MaxDivergence,
        col("value")
            .dp()
            .sum((lit(0u64), lit(max)), Some(0.0), Some(true)),
        None,
    )
    .err()
    .expect("signed UInt64 sums must fail at construction");
    assert!(format!("{error:?}").contains("UInt64"));
    Ok(())
}
