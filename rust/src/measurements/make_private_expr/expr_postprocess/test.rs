use polars::df;
use polars_plan::dsl::{col, len, lit};

use crate::{
    measures::MaxDivergence,
    metrics::{PartitionDistance, SymmetricDistance},
    transformations::test_helper::get_test_data,
};

use super::*;

#[test]
fn test_postprocess_alias() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;

    let expr = len().alias("new name");

    let m_expr = expr.clone().make_private(
        lf_domain.aggregate(["chunk_2_bool"]),
        PartitionDistance(SymmetricDistance),
        MaxDivergence::default(),
        Some(0.),
    )?;

    let expr_p = m_expr.invoke(&lf.logical_plan)?.expr;
    let actual = lf.group_by([col("chunk_2_bool")]).agg([expr_p]).collect()?;
    let expected = df!("chunk_2_bool" => [false, true], "new name" => [500u32, 500])?;

    assert!(actual
        .sort(["chunk_2_bool"], Default::default())?
        .equals(&expected));

    Ok(())
}

#[test]
fn test_postprocess_binary() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;

    // any binary expression is fine
    let expr = (len() / lit(2)).eq(lit(23)).or(lit(false));

    let m_expr = expr.clone().make_private(
        lf_domain.aggregate(["chunk_2_bool"]),
        PartitionDistance(SymmetricDistance),
        MaxDivergence::default(),
        Some(0.),
    )?;

    let expr_p = m_expr.invoke(&lf.logical_plan)?.expr;
    let actual = lf.group_by([col("chunk_2_bool")]).agg([expr_p]).collect()?;
    let expected = df!("chunk_2_bool" => [false, true], "len" => [false, false])?;

    assert!(actual
        .sort(["chunk_2_bool"], Default::default())?
        .equals(&expected));

    Ok(())
}
