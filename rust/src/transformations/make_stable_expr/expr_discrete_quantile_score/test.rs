use super::*;
use polars::prelude::*;

use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    metrics::{L0PInfDistance, SymmetricDistance},
    polars::apply_anonymous_function,
};

pub fn get_quantile_test_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("cycle_(..101f64)", AtomDomain::<i32>::default()),
        SeriesDomain::new("cycle_(..10i32)", AtomDomain::<f64>::default()),
    ])?
    .with_margin(Margin::select().with_max_length(1000).with_invariant_keys())?
    .with_margin(
        Margin::by(["cycle_(..10i32)"])
            .with_max_length(1000)
            .with_invariant_keys(),
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

    let alpha = 0.5;
    let candidates = candidates;
    let expr = apply_anonymous_function(
        vec![col("cycle_(..101f64)"), lit(alpha), lit(candidates)],
        DiscreteQuantileScoreShim,
    );

    let m_quant: Transformation<_, _, _, L0InfDistance<LInfDistance<f64>>> =
        expr.make_stable(lf_domain.select(), L0PInfDistance(SymmetricDistance))?;

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

    let expr = col("cycle_(..10i32)").cast(DataType::Int64);
    let alpha = 0.5;
    let candidates = candidates;
    let expr = apply_anonymous_function(
        vec![expr, lit(alpha), lit(candidates)],
        DiscreteQuantileScoreShim,
    );

    let m_quant: Transformation<_, _, _, L0InfDistance<LInfDistance<f64>>> =
        expr.make_stable(expr_domain, L0PInfDistance(SymmetricDistance))?;

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
