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
    let lf = df!("A" => [true; 3])?.lazy();

    let t_const = lit(1.0).make_stable(lf_domain.clone().row_by_row(), SymmetricDistance)?;
    let expr_const = t_const.invoke(&(lf.logical_plan.clone(), all()))?.1;
    assert_eq!(expr_const, lit(1.0));

    let actual = lf.with_column(expr_const).collect()?;
    let expect = df!("A" => [true; 3], "literal" => [1.0; 3])?;
    assert_eq!(actual, expect);

    Ok(())
}
