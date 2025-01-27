use polars::df;
use polars::lazy::frame::IntoLazy;
use polars::prelude::{lit, NamedFrom};
use polars::series::Series;
use polars_plan::dsl::col;

use crate::domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain};
use crate::metrics::SymmetricDistance;

use super::*;

#[test]
fn test_make_expr_fill_null() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "i32",
        OptionDomain::new(AtomDomain::<i32>::default()),
    )])?;

    let lf = df!("i32" => [None, Some(1)])?.lazy();

    let t_fill_null = col("i32")
        .fill_null(0)
        .make_stable(lf_domain.clone().row_by_row(), SymmetricDistance)?;
    let expr_fill_null = t_fill_null.invoke(&lf.logical_plan)?.expr;
    println!("{:?}", expr_fill_null);
    let actual = lf.with_column(expr_fill_null).collect()?;

    assert_eq!(actual, df!("i32" => [0, 1])?);

    assert!(!t_fill_null.output_domain.column.nullable);

    Ok(())
}

#[test]
fn test_make_expr_fill_null_expr() -> Fallible<()> {
    let f64_null = SeriesDomain::new(
        "f64_null",
        OptionDomain::new(AtomDomain::<f64>::new_nullable()),
    );
    let f64_nonnull = SeriesDomain::new("f64_nonnull", AtomDomain::<f64>::default());
    let lf_domain = LazyFrameDomain::new(vec![f64_null, f64_nonnull])?;

    let lf = df!(
        "f64_null" => [None, Some(1.), Some(f64::NAN)],
        "f64_nonnull" => [0f64; 3]
    )?
    .lazy();

    let t_fill_null = col("f64_null")
        .fill_null(col("f64_nonnull"))
        .make_stable(lf_domain.clone().row_by_row(), SymmetricDistance)?;

    let expr_fill_null = t_fill_null.invoke(&lf.logical_plan)?.expr;
    let actual = lf.with_column(expr_fill_null).collect()?;
    let actual_f64_null = actual.column("f64_null")?.as_materialized_series();

    assert_eq!(
        actual_f64_null,
        &Series::new("f64_null".into(), [Some(0.), Some(1.), Some(f64::NAN)])
    );

    assert!(!t_fill_null.output_domain.column.nullable);

    Ok(())
}

#[test]
fn test_make_expr_fill_null_expr_filter_fail() -> Fallible<()> {
    let f64_null = SeriesDomain::new(
        "f64_null",
        OptionDomain::new(AtomDomain::<f64>::new_nullable()),
    );
    let f64_nonnull = SeriesDomain::new("f64_nonnull", AtomDomain::<f64>::default());
    let lf_domain = LazyFrameDomain::new(vec![f64_null, f64_nonnull])?;

    let err = col("f64_null")
        .fill_null(lit(0.0).filter(col("f64_nonnull").gt_eq(0.0)))
        .make_stable(lf_domain.clone().row_by_row(), SymmetricDistance)
        .unwrap_err();

    assert_eq!(
        err.message,
        Some("filter is not allowed in a row-by-row context".to_string())
    );
    Ok(())
}
