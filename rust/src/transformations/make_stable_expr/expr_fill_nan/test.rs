use polars::df;
use polars::lazy::frame::IntoLazy;
use polars::prelude::{lit, NamedFrom};
use polars::series::Series;
use polars_plan::dsl::col;

use crate::domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain};
use crate::metrics::SymmetricDistance;

use super::*;

#[test]
fn test_make_expr_fill_nan_literal() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "f64",
        OptionDomain::new(AtomDomain::<f64>::default()),
    )])?;

    let lf = df!("f64" => [None, Some(1.), Some(f64::NAN)])?.lazy();

    let t_fill_nan = col("f64")
        .fill_nan(0.0)
        .make_stable(lf_domain.clone().row_by_row(), SymmetricDistance)?;
    let expr_fill_nan = t_fill_nan.invoke(&lf.logical_plan)?.expr;
    let actual = lf.with_column(expr_fill_nan).collect()?;

    assert_eq!(actual, df!("f64" => [None, Some(1.), Some(0.)])?);

    assert!(!t_fill_nan
        .output_domain
        .column
        .atom_domain::<f64>()?
        .nullable());

    Ok(())
}

#[test]
fn test_make_expr_fill_nan_expr() -> Fallible<()> {
    let f64_nan = SeriesDomain::new(
        "f64_nan",
        OptionDomain::new(AtomDomain::<f64>::new_nullable()),
    );
    let f64_nonnan = SeriesDomain::new("f64_nonnan", AtomDomain::<f64>::default());
    let lf_domain = LazyFrameDomain::new(vec![f64_nan, f64_nonnan])?;

    let lf = df!(
        "f64_nan" => [None, Some(1.), Some(f64::NAN)],
        "f64_nonnan" => [0f64; 3]
    )?
    .lazy();

    let t_fill_nan = col("f64_nan")
        .fill_nan(col("f64_nonnan"))
        .make_stable(lf_domain.clone().row_by_row(), SymmetricDistance)?;

    let expr_fill_nan = t_fill_nan.invoke(&lf.logical_plan)?.expr;
    let actual = lf.with_column(expr_fill_nan).collect()?;
    let actual_f64_nan = actual.column("f64_nan")?.as_materialized_series();

    assert_eq!(
        actual_f64_nan,
        &Series::new("f64_nan".into(), [None, Some(1.), Some(0.)])
    );

    assert!(!t_fill_nan
        .output_domain
        .column
        .atom_domain::<f64>()?
        .nullable());

    Ok(())
}

#[test]
fn test_make_expr_fill_nan_expr_filter_fail() -> Fallible<()> {
    let f64_nan = SeriesDomain::new(
        "f64_nan",
        OptionDomain::new(AtomDomain::<f64>::new_nullable()),
    );
    let f64_nonnan = SeriesDomain::new("f64_nonnan", AtomDomain::<f64>::default());
    let lf_domain = LazyFrameDomain::new(vec![f64_nan, f64_nonnan])?;

    let err = col("f64_nan")
        .fill_nan(lit(0.0).filter(col("f64_nonnan").gt_eq(0.0)))
        .make_stable(lf_domain.clone().row_by_row(), SymmetricDistance)
        .unwrap_err();

    assert_eq!(
        err.message,
        Some("filter is not allowed in a row-by-row context".to_string())
    );
    Ok(())
}
