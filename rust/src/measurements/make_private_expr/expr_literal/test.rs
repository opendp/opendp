use polars::df;
use polars_plan::dsl::lit;

use crate::{
    measurements::{PrivateExpr, make_private_lazyframe},
    measures::MaxDivergence,
    metrics::{FrameDistance, L0PInfDistance, SymmetricDistance},
    transformations::test_helper::get_test_data,
};

use super::*;

#[test]
fn test_make_expr_private_lit() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let expr_domain = lf_domain.select();

    let m_lit = lit(1).make_private(
        expr_domain,
        L0PInfDistance(SymmetricDistance),
        MaxDivergence,
        None,
    )?;

    let actual = m_lit.invoke(&lf.logical_plan)?;
    assert_eq!(actual.expr, lit(1));
    Ok(())
}

#[test]
fn test_make_expr_private_lit_groupby() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;

    let m_lit = make_private_lazyframe(
        lf_domain.cast_carrier(),
        FrameDistance(SymmetricDistance),
        MaxDivergence,
        lf.clone().group_by(["chunk_2_bool"]).agg([lit(1)]),
        None,
        None,
    )?;

    let actual = m_lit.invoke(&lf)?.collect()?;
    let expect = df!(
        "chunk_2_bool" => [false, true],
        "literal" => [1, 1]
    )?;
    assert_eq!(actual.sort(["chunk_2_bool"], Default::default())?, expect);
    Ok(())
}
