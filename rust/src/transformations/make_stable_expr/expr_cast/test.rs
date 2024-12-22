use crate::domains::{AtomDomain, LazyFrameDomain, SeriesDomain};
use crate::metrics::SymmetricDistance;
use crate::transformations::make_stable_lazyframe;

use super::*;

#[test]
fn test_make_expr_cast_String() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "String",
        AtomDomain::<String>::default(),
    )])?;
    let lf = df!(
        "String" => &["A".to_string(), "3.14".to_string(), "1e300".to_string(), "NaN".to_string()],
    )?
    .lazy();

    let casted = lf.clone().with_columns([
        col("String").cast(DataType::Float64).alias("String_f64"),
        col("String").cast(DataType::Int64).alias("String_i64"),
        // Polars doesn't support casting a string to a boolean
        // col("String").cast(DataType::Boolean).alias("String_bool"),
    ]);
    let t_casted = make_stable_lazyframe(lf_domain, SymmetricDistance, casted)?;

    let actual = t_casted.invoke(&lf)?.collect()?;
    let expected = df!(
        // repeat inputs
        "String" => &["A".to_string(), "3.14".to_string(), "1e300".to_string(), "NaN".to_string()],

        // string -> int casts can go to null and nan
        "String_f64" => &[None, Some(3.14), Some(1e300), Some(f64::NAN)],
        // No value is exactly integer, so all are None
        "String_i64" => &[None::<i64>, None, None, None],
        // non-strict casting still fails, but fails in a data-independent way
        // "String_bool" => &[],
    )?;
    assert_eq!(expected, actual);
    Ok(())
}

#[test]
fn test_make_expr_cast_f64() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("f64", AtomDomain::<f64>::default())])?;
    let lf = df!(
        "f64" => &[f64::NAN, 1., 1.0e300, -1.],
    )?
    .lazy();

    let casted = lf.clone().with_columns([
        col("f64").cast(DataType::String).alias("f64_String"),
        col("f64").cast(DataType::Int64).alias("f64_i64"),
        col("f64").cast(DataType::Boolean).alias("f64_bool"),
    ]);
    let t_casted = make_stable_lazyframe(lf_domain, SymmetricDistance, casted)?;

    let actual = t_casted.invoke(&lf)?.collect()?;
    let expected = df!(
        // repeat inputs
        "f64" => &[f64::NAN, 1., 1e300, -1.],
        "f64_String" => &["NaN".to_string(), "1.0".to_string(), "1e300".to_string(), "-1.0".to_string()],
        "f64_i64" => &[None, Some(1), None, Some(-1)],
        // NaN is considered truth-y
        "f64_bool" => &[true, true, true, true],
    )?;
    assert_eq!(expected, actual);
    Ok(())
}

#[test]
fn test_make_expr_cast_int() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("i64", AtomDomain::<i64>::default())])?;
    let lf = df!(
        "i64" => &[i64::MAX, i64::MIN, 0, 1],
    )?
    .lazy();

    let casted = lf.clone().with_columns([
        col("i64").cast(DataType::String).alias("i64_String"),
        col("i64").cast(DataType::Float64).alias("i64_f64"),
        col("i64").cast(DataType::Boolean).alias("i64_bool"),
    ]);
    let t_casted = make_stable_lazyframe(lf_domain, SymmetricDistance, casted)?;

    let actual = t_casted.invoke(&lf)?.collect()?;
    let expected = df!(
        // repeat inputs
        "i64" => &[i64::MAX, i64::MIN, 0, 1],
        "i64_String" => &["9223372036854775807".to_string(), "-9223372036854775808".to_string(), "0".to_string(), "1".to_string()],
        "i64_f64" => &[i64::MAX as f64, i64::MIN as f64, 0.0, 1.0],
        "i64_bool" => &[true, true, false, true],
    )?;
    assert_eq!(expected, actual);
    Ok(())
}

#[test]
fn test_make_expr_cast_bool() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "bool",
        AtomDomain::<bool>::default(),
    )])?;
    let lf = df!(
        "bool" => &[true, true, false, false]
    )?
    .lazy();

    let casted = lf.clone().with_columns([
        col("bool").cast(DataType::String).alias("bool_String"),
        col("bool").cast(DataType::Float64).alias("bool_f64"),
        col("bool").cast(DataType::Int64).alias("bool_i64"),
    ]);
    let t_casted = make_stable_lazyframe(lf_domain, SymmetricDistance, casted)?;

    let actual = t_casted.invoke(&lf)?.collect()?;
    let expected = df!(
        // repeat inputs
        "bool" => &[true, true, false, false],
        "bool_String" => &["true".to_string(), "true".to_string(), "false".to_string(), "false".to_string()],
        "bool_f64" => &[1.0f64, 1.0, 0.0, 0.0],
        "bool_i64" => &[1i64, 1, 0, 0],
    )?;
    assert_eq!(expected, actual);
    Ok(())
}
