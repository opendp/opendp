use polars::df;
use polars::lazy::frame::IntoLazy;
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
