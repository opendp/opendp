use crate::domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain};
use crate::metrics::SymmetricDistance;

use super::*;

fn get_f64_i64_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("f64", AtomDomain::<f64>::default()),
        SeriesDomain::new("i64", AtomDomain::<i64>::default()),
        SeriesDomain::new("f64_null", OptionDomain::new(AtomDomain::<f64>::default())),
        SeriesDomain::new("i64_null", OptionDomain::new(AtomDomain::<i64>::default())),
    ])?;

    let lf = df!(
        "f64" => [0., f64::NAN, f64::NAN, f64::INFINITY],
        "i64" => [0, 1, 2, 3],
        "f64_null" => [Some(0.), Some(f64::NAN), None, Some(f64::INFINITY)],
        "i64_null" => [Some(0), None, None, Some(3)],
    )?
    .lazy();

    Ok((lf_domain, lf))
}

// check if members of the output domain may have a nullable bitmask
// (unrelated to inherent nullity within AtomDomain)
macro_rules! is_nullable {
    ($col:expr, $op:ident, $domain:ident) => {
        $col.$op()
            .make_stable($domain.clone(), SymmetricDistance)?
            .output_domain
            .column
            .nullable
    };
}

#[test]
fn test_is_null() -> Fallible<()> {
    let (lf_domain, lf) = get_f64_i64_data()?;
    let expr_domain = lf_domain.row_by_row();
    assert_eq!(
        lf.with_column(all().is_null()).collect()?,
        df!(
            "f64" => [false, false, false, false],
            "i64" => [false, false, false, false],
            "f64_null" => [false, false, true, false],
            "i64_null" => [false, true, true, false],
        )?
    );

    assert!(!is_nullable!(col("f64"), is_null, expr_domain));
    assert!(!is_nullable!(col("i64"), is_null, expr_domain));
    assert!(!is_nullable!(col("f64_null"), is_null, expr_domain));
    assert!(!is_nullable!(col("i64_null"), is_null, expr_domain));

    Ok(())
}

#[test]
fn test_is_not_null() -> Fallible<()> {
    let (lf_domain, lf) = get_f64_i64_data()?;
    let expr_domain = lf_domain.row_by_row();
    assert_eq!(
        lf.with_column(all().is_not_null()).collect()?,
        df!(
            "f64" => [true, true, true, true],
            "i64" => [true, true, true, true],
            "f64_null" => [true, true, false, true],
            "i64_null" => [true, false, false, true],
        )?
    );

    assert!(!is_nullable!(col("f64"), is_not_null, expr_domain));
    assert!(!is_nullable!(col("i64"), is_not_null, expr_domain));
    assert!(!is_nullable!(col("f64_null"), is_not_null, expr_domain));
    assert!(!is_nullable!(col("i64_null"), is_not_null, expr_domain));

    Ok(())
}

#[test]
fn test_is_finite() -> Fallible<()> {
    let (lf_domain, lf) = get_f64_i64_data()?;
    let expr_domain = lf_domain.row_by_row();
    assert_eq!(
        lf.with_column(all().is_finite()).collect()?,
        df!(
            "f64" => [true, false, false, false],
            "i64" => [true, true, true, true],
            "f64_null" => [Some(true), Some(false), None, Some(false)],
            "i64_null" => [true, true, true, true],
        )?
    );

    assert!(!is_nullable!(col("f64"), is_finite, expr_domain));
    assert!(!is_nullable!(col("i64"), is_finite, expr_domain));
    assert!(is_nullable!(col("f64_null"), is_finite, expr_domain));
    assert!(is_nullable!(col("i64_null"), is_finite, expr_domain));

    Ok(())
}

#[test]
fn test_is_infinite() -> Fallible<()> {
    let (lf_domain, lf) = get_f64_i64_data()?;
    let expr_domain = lf_domain.row_by_row();
    assert_eq!(
        lf.with_column(all().is_infinite()).collect()?,
        df!(
            "f64" => [false, false, false, true],
            "i64" => [false, false, false, false],
            "f64_null" => [Some(false), Some(false), None, Some(true)],
            "i64_null" => [false, false, false, false],
        )?
    );

    assert!(!is_nullable!(col("f64"), is_infinite, expr_domain));
    assert!(!is_nullable!(col("i64"), is_infinite, expr_domain));
    assert!(is_nullable!(col("f64_null"), is_infinite, expr_domain));
    assert!(is_nullable!(col("i64_null"), is_infinite, expr_domain));

    Ok(())
}

#[test]
fn test_is_nan() -> Fallible<()> {
    let (lf_domain, lf) = get_f64_i64_data()?;
    let expr_domain = lf_domain.row_by_row();
    assert_eq!(
        lf.with_column(all().is_nan()).collect()?,
        df!(
            "f64" => [false, true, true, false],
            "i64" => [false, false, false, false],
            "f64_null" => [Some(false), Some(true), None, Some(false)],
            "i64_null" => [false, false, false, false],
        )?
    );

    assert!(!is_nullable!(col("f64"), is_nan, expr_domain));
    assert!(!is_nullable!(col("i64"), is_nan, expr_domain));
    assert!(is_nullable!(col("f64_null"), is_nan, expr_domain));
    assert!(is_nullable!(col("i64_null"), is_nan, expr_domain));

    Ok(())
}

#[test]
fn test_is_not_nan() -> Fallible<()> {
    let (lf_domain, lf) = get_f64_i64_data()?;
    let expr_domain = lf_domain.row_by_row();
    assert_eq!(
        lf.with_column(all().is_not_nan()).collect()?,
        df!(
            "f64" => [true, false, false, true],
            "i64" => [true, true, true, true],
            // nulls propagate through nan check on floats
            "f64_null" => [Some(true), Some(false), None, Some(true)],
            "i64_null" => [true, true, true, true],
        )?
    );

    assert!(!is_nullable!(col("f64"), is_not_nan, expr_domain));
    assert!(!is_nullable!(col("i64"), is_not_nan, expr_domain));
    assert!(is_nullable!(col("f64_null"), is_not_nan, expr_domain));
    assert!(is_nullable!(col("i64_null"), is_not_nan, expr_domain));

    Ok(())
}

fn get_bool_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf = df!(
        "bool" => [true, false, false],
        "bool_null" => [Some(true), Some(false), None],
    )?
    .lazy();

    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("bool", AtomDomain::<bool>::default()),
        SeriesDomain::new(
            "bool_null",
            OptionDomain::new(AtomDomain::<bool>::default()),
        ),
    ])?;

    Ok((lf_domain, lf))
}

#[test]
fn test_not() -> Fallible<()> {
    let (lf_domain, lf) = get_bool_data()?;
    let expr_domain = lf_domain.row_by_row();

    assert_eq!(
        lf.clone().with_column(all().not()).collect()?,
        df!(
            "bool" => [false, true, true],
            "bool_null" => [Some(false), Some(true), None]
        )?
    );

    assert!(!is_nullable!(col("bool"), not, expr_domain));
    assert!(is_nullable!(col("bool_null"), not, expr_domain));

    Ok(())
}

fn get_i64_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf = df!(
        "i64" => [0, 1, 2, 3],
        "i64_null" => [Some(0), None, None, Some(3)],
    )?
    .lazy();

    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("i64", AtomDomain::<i64>::default()),
        SeriesDomain::new("i64_null", OptionDomain::new(AtomDomain::<i64>::default())),
    ])?;

    Ok((lf_domain, lf))
}

#[test]
fn test_not_i64() -> Fallible<()> {
    let (lf_domain, lf) = get_i64_data()?;
    let expr_domain = lf_domain.row_by_row();

    assert_eq!(
        lf.clone().with_column(all().not()).collect()?,
        df!(
            "i64" => [-1, -2, -3, -4],
            "i64_null" => [Some(-1), None, None, Some(-4)],
        )?
    );

    assert!(!is_nullable!(col("i64"), not, expr_domain));
    assert!(is_nullable!(col("i64_null"), not, expr_domain));

    Ok(())
}
