use polars::df;
use polars::lazy::frame::IntoLazy;
use polars_plan::dsl::col;

use crate::domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain};
use crate::metrics::SymmetricDistance;

use super::*;

#[test]
fn test_make_expr_fill_nan() -> Fallible<()> {
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
