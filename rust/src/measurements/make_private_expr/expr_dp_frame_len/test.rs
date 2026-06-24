use super::*;
use polars::prelude::*;

use crate::{
    measures::MaxDivergence,
    metrics::{L0PInfDistance, SymmetricDistance},
    polars::dp_len,
    transformations::test_helper::get_test_data,
};

#[test]
fn test_make_expr_dp_frame_len() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;

    let m_len = make_expr_dp_frame_len(
        lf_domain.select(),
        L0PInfDistance(SymmetricDistance),
        MaxDivergence,
        dp_len(Some(0.0), false),
        None,
    )?;

    let dp_expr = m_len.invoke(&lf.logical_plan)?.expr;
    let df_actual = lf.select([dp_expr]).collect()?;

    assert_eq!(df_actual, df!("len" => [1000u32])?);

    Ok(())
}

#[test]
fn test_make_expr_dp_frame_len_allow_negative() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;

    let m_len = make_expr_dp_frame_len(
        lf_domain.select(),
        L0PInfDistance(SymmetricDistance),
        MaxDivergence,
        dp_len(Some(0.0), true),
        None,
    )?;

    let dp_expr = m_len.invoke(&lf.logical_plan)?.expr;
    let df_actual = lf.select([dp_expr]).collect()?;

    // with allow_negative the length is released as a signed integer
    assert_eq!(df_actual, df!("len" => [1000i64])?);

    Ok(())
}
