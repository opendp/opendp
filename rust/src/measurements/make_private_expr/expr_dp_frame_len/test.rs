use polars::prelude::*;

use crate::{
    measurements::make_private_expr::expr_dp_frame_len::make_expr_dp_frame_len,
    measures::MaxDivergence,
    metrics::{L0PInfDistance, SymmetricDistance},
    polars::dp_len,
    transformations::test_helper::get_test_data,
};

#[test]
fn test_dp_frame_len_default_dtype() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;

    let measurement = make_expr_dp_frame_len(
        lf_domain.select(),
        L01PInfDistance(SymmetricDistance),
        MaxDivergence,
        dp_len(Some(0.0)),
        None,
    )?;

    let dp_expr = measurement.invoke(&lf.logical_plan)?.expr;
    let df = lf.select([dp_expr]).collect()?;

    assert_eq!(df.column("len")?.dtype(), &DataType::UInt32);
    Ok(())
}