use super::*;
use polars::prelude::*;

use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    metrics::SymmetricDistance,
    polars::PrivacyNamespace,
};

pub fn get_quantile_test_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("cycle_(..101f64)", AtomDomain::<i32>::default()),
        SeriesDomain::new("cycle_(..10i32)", AtomDomain::<f64>::default()),
    ])?
    .with_margin(
        Margin::default()
            .with_max_partition_length(1000)
            .with_public_keys(),
    )?
    .with_margin(
        Margin::by(["cycle_(..10i32)"])
            .with_max_partition_length(1000)
            .with_public_keys(),
    )?;

    let lf = df!(
        "cycle_(..101f64)" => (0..1010).map(|i| (i % 101) as f64).collect::<Vec<_>>(),
        "cycle_(..10i32)" => (0..1010).map(|i| (i % 10)).collect::<Vec<_>>()
    )?
    .lazy();

    Ok((lf_domain, lf))
}

#[test]
fn test_expr_discrete_quantile_score_float() -> Fallible<()> {
    let (lf_domain, lf) = get_quantile_test_data()?;
    let candidates = Series::new(
        "".into(),
        [0., 10., 20., 30., 40., 50., 60., 70., 80., 90., 100.],
    );

    let m_quant: Transformation<_, _, _, Parallel<LInfDistance<f64>>> = col("cycle_(..101f64)")
        .dp()
        .quantile_score(0.5, candidates)
        .make_stable(lf_domain.select(), PartitionDistance(SymmetricDistance))?;

    let dp_expr = m_quant.invoke(&lf.logical_plan)?.expr;

    let df_actual = lf.clone().select([dp_expr]).collect()?;
    let AnyValue::Array(series, _) = df_actual.column("cycle_(..101f64)")?.get(0)? else {
        panic!("Expected an array");
    };

    let actual: Vec<u64> = series
        .u64()?
        .downcast_iter()
        .flat_map(StaticArray::values_iter)
        .collect();

    let expected = vec![1000, 800, 600, 400, 200, 0, 200, 400, 600, 800, 1000];
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_expr_discrete_quantile_score_int() -> Fallible<()> {
    let (lf_domain, lf) = get_quantile_test_data()?;
    let expr_domain = lf_domain.select();
    let candidates = Series::new("".into(), [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    let m_quant: Transformation<_, _, _, Parallel<LInfDistance<f64>>> = col("cycle_(..10i32)")
        .dp()
        .quantile_score(0.5, candidates)
        .make_stable(expr_domain, PartitionDistance(SymmetricDistance))?;

    let dp_expr = m_quant.invoke(&lf.logical_plan)?.expr;

    let df_actual = lf.clone().select([dp_expr]).collect()?;
    let AnyValue::Array(series, _) = df_actual.column("cycle_(..10i32)")?.get(0)? else {
        panic!("Expected an array");
    };

    let actual: Vec<u64> = series
        .u64()?
        .downcast_iter()
        .flat_map(StaticArray::values_iter)
        .collect();

    // there are 101 occurrences of each of the 10 unique values
    // therefore scores follow the pattern of: 1000 - 101*k
    let expected = vec![909, 707, 505, 303, 101, 101, 303, 505, 707, 909];
    assert_eq!(actual, expected);

    Ok(())
}
