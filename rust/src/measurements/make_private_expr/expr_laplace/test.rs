use super::*;
use polars::prelude::*;

use crate::{
    core::PrivacyNamespaceHelper,
    measurements::{make_private_expr, make_private_lazyframe},
    metrics::{PartitionDistance, SymmetricDistance},
    transformations::test_helper::get_test_data,
};

#[test]
fn test_make_expr_laplace() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let expr_domain = lf_domain.select();
    let scale: f64 = 0.0;

    let m_quant = make_private_expr(
        expr_domain,
        PartitionDistance(SymmetricDistance),
        MaxDivergence::default(),
        col("const_1f64").dp().sum((0., 1.), Some(scale)),
        None,
    )?;

    let dp_expr = m_quant.invoke(&(lf.logical_plan.clone(), all()))?;
    let df_actual = lf.select([dp_expr]).collect()?;

    assert_eq!(df_actual, df!("const_1f64" => [1000.0])?);

    Ok(())
}

#[test]
fn test_make_laplace_grouped() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let scale: f64 = 0.0;

    let expr_exp = col("chunk_(..10u32)")
        .clip(lit(0), lit(8))
        .sum()
        .dp()
        .laplace(None);
    let m_lap = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        MaxDivergence::default(),
        lf.clone().group_by(["chunk_2_bool"]).agg([expr_exp]),
        Some(scale),
    )?;
    // sum([0, 1, 2, 3, 4]) * 100 = 1000
    // sum([5, 6, 7, 8, 8]) * 100 = 3400

    let df_act = m_lap.invoke(&lf)?.collect()?;
    let df_exp = df!(
        "chunk_2_bool" => [false, true],
        "chunk_(..10u32)" => [1000, 3400]
    )?;

    assert_eq!(
        df_act.sort(["chunk_2_bool"], false, false)?,
        df_exp.sort(["chunk_2_bool"], false, false)?
    );
    Ok(())
}
