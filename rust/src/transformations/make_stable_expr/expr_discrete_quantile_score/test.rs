use super::*;
use polars::prelude::*;

use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    metrics::{L0PInfDistance, SymmetricDistance},
    polars::apply_anonymous_function,
};

pub fn get_quantile_test_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("cycle_(..101f64)", AtomDomain::<f64>::new(None, None)),
        SeriesDomain::new("cycle_(..10i32)", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::select().with_max_length(1000).with_invariant_keys())?
    .with_margin(
        Margin::by(["cycle_(..10i32)"])
            .with_max_length(1000)
            .with_invariant_keys(),
    )?;

    let lf = df!(
        "cycle_(..101f64)" => (0..1010).map(|i| (i % 101) as f64).collect::<Vec<_>>(),
        "cycle_(..10i32)" => (0..1010).map(|i| i % 10).collect::<Vec<_>>()
    )?
    .lazy();

    Ok((lf_domain, lf))
}

#[test]
fn test_quantile_candidate_casts() -> Fallible<()> {
    let floats = Series::new("".into(), [1.0f64, 2.0]);
    let casted = cast_quantile_candidates(&floats, &DataType::Int64)?;
    assert_eq!(
        casted.i64()?.into_no_null_iter().collect::<Vec<_>>(),
        vec![1, 2]
    );

    let ints = Series::new("".into(), [1i64, 2]);
    assert_eq!(
        cast_quantile_candidates(&ints, &DataType::Int32)?.dtype(),
        &DataType::Int32
    );

    let fractional = Series::new("".into(), [1.1f64, 2.9]);
    assert!(cast_quantile_candidates(&fractional, &DataType::Int64).is_err());

    // Large integers cannot always be represented exactly as f64.
    let large = Series::new("".into(), [(1u64 << 53) + 1]);
    assert!(cast_quantile_candidates(&large, &DataType::Float64).is_err());

    let max_uint = Series::new("".into(), [u64::MAX]);
    assert!(cast_quantile_candidates(&max_uint, &DataType::Float64).is_err());

    let out_of_range = Series::new("".into(), [i64::MAX]);
    assert!(cast_quantile_candidates(&out_of_range, &DataType::Int32).is_err());

    let rounded_floats = Series::new("".into(), [0.1f64, 0.2, 0.3]);
    let casted = cast_quantile_candidates(&rounded_floats, &DataType::Float32)?;
    assert_eq!(
        casted.f32()?.into_no_null_iter().collect::<Vec<_>>(),
        vec![0.1f32, 0.2, 0.3]
    );

    // Rounding is allowed, but overflowing a finite candidate to infinity is not.
    let overflowing_float = Series::new("".into(), [f64::MAX]);
    assert!(cast_quantile_candidates(&overflowing_float, &DataType::Float32).is_err());

    // Exactly representable values may be larger than 2**53.
    let exact_large = Series::new("".into(), [(1u64 << 53) + 2]);
    assert!(cast_quantile_candidates(&exact_large, &DataType::Float64).is_ok());

    let infinity = Series::new("".into(), [f32::INFINITY]);
    assert!(cast_quantile_candidates(&infinity, &DataType::Float64).is_ok());
    Ok(())
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
