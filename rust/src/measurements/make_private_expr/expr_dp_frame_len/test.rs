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

#[test]
fn test_make_expr_dp_frame_len_releases_negative() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;

    // the true length is 1000; with a large scale relative to that, roughly half of
    // the noisy releases land below zero, so over 100 samples at least one being
    // negative is a near-certainty (P(all non-negative) ~ 0.5^100).
    let m_len = make_expr_dp_frame_len(
        lf_domain.select(),
        L0PInfDistance(SymmetricDistance),
        MaxDivergence,
        dp_len(Some(1e7), true),
        None,
    )?;

    let dp_expr = m_len.invoke(&lf.logical_plan)?.expr;

    let mut saw_negative = false;
    for _ in 0..100 {
        let df_actual = lf.clone().select([dp_expr.clone()]).collect()?;
        let len = df_actual.column("len")?.i64()?.get(0).unwrap();
        if len < 0 {
            saw_negative = true;
            break;
        }
    }

    assert!(
        saw_negative,
        "expected at least one negative release over 100 samples with allow_negative=true"
    );

    Ok(())
}
