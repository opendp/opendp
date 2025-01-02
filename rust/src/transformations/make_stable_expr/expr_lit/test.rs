use polars::{df, lazy::frame::IntoLazy};
use polars_plan::dsl::lit;

use crate::{
    domains::{AtomDomain, LazyFrameDomain},
    metrics::SymmetricDistance,
    transformations::StableExpr,
};

use super::*;

#[test]
fn test_lit() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "bool",
        AtomDomain::<bool>::default(),
    )])?;
    let lf = df!("bool" => [true; 3])?.lazy();

    let t_const = lit(1.0).make_stable(lf_domain.row_by_row(), SymmetricDistance)?;
    let expr_const = t_const.invoke(&lf.logical_plan)?.expr;
    assert_eq!(expr_const, lit(1.0));

    let actual = lf.with_column(expr_const).collect()?;
    let expect = df!("bool" => [true; 3], "literal" => [1.0; 3])?;
    assert_eq!(actual, expect);

    let series_domain = &t_const.output_domain.column;
    assert_eq!(series_domain.atom_domain::<f64>()?.nullable(), false);
    assert_eq!(series_domain.nullable, false);

    Ok(())
}
