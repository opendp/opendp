use super::*;
use polars::prelude::*;
use polars_arrow::array::{FixedSizeListArray, UInt32Array};

use crate::{
    error::ErrorVariant,
    measurements::make_private_expr,
    metrics::{PartitionDistance, SymmetricDistance},
    polars::PrivacyNamespace,
    transformations::expr_discrete_quantile_score::test::get_quantile_test_data,
};

#[test]
fn test_report_noisy_max_gumbel_udf() -> Fallible<()> {
    // the scores are packed into a FixedSizeListArray with 3 elements per row
    // the max value in the first row is 3, the max value in the second row is 1, and the max value in the third row is 9
    // the indices of the max values are 0, 1, and 2 respectively
    let scores_slice = &[3, 1, 0, 0, 1, 0, 0, 0, 9];
    let expect_slice = &[0u32, 1, 2];

    let dtype = ArrowDataType::FixedSizeList(
        Box::new(ArrowField::new("item".into(), ArrowDataType::UInt32, true)),
        3,
    );

    let fsla = FixedSizeListArray::new(
        dtype,
        3,
        Box::new(UInt32Array::from_slice(scores_slice)),
        None,
    );
    let scores = Series::from(ArrayChunked::from(fsla)).into_column();

    let actual = super::report_noisy_max_gumbel_udf(
        &[scores],
        ReportNoisyMaxPlugin {
            optimize: Optimize::Max,
            scale: 0.0,
        },
    )?;

    let expect = Column::new("".into(), expect_slice);

    assert_eq!(actual, expect);
    Ok(())
}

#[test]
fn test_report_noisy_max_gumbel_expr() -> Fallible<()> {
    let (lf_domain, lf) = get_quantile_test_data()?;
    let scale: f64 = 1e-8;
    let candidates = Series::new(
        "".into(),
        [0., 10., 20., 30., 40., 50., 60., 70., 80., 90., 100.],
    );

    let m_quant = make_private_expr(
        lf_domain.select(),
        PartitionDistance(SymmetricDistance),
        MaxDivergence::default(),
        col("cycle_(..101f64)")
            .dp()
            .quantile_score(0.5, candidates)
            .dp()
            .report_noisy_max_gumbel(Optimize::Min, Some(scale)),
        None,
    )?;

    let dp_expr = m_quant.invoke(&lf.logical_plan)?.expr;
    let df = lf.select([dp_expr]).collect()?;
    let actual = df.column("cycle_(..101f64)")?.u32()?.get(0).unwrap();
    assert_eq!(actual, 5);

    Ok(())
}

#[test]
fn test_fail_report_noisy_max_gumbel_expr_nan_scale() -> Fallible<()> {
    let (lf_domain, _) = get_quantile_test_data()?;
    let scale: f64 = f64::NAN;
    let candidates = Series::new(
        "".into(),
        [0., 10., 20., 30., 40., 50., 60., 70., 80., 90., 100.],
    );

    let err_variant = make_private_expr(
        lf_domain.select(),
        PartitionDistance(SymmetricDistance),
        MaxDivergence::default(),
        col("cycle_(..101f64)").dp().median(candidates, Some(scale)),
        None,
    )
    .map(|_| ())
    .unwrap_err()
    .variant;

    assert_eq!(err_variant, ErrorVariant::MakeMeasurement);

    Ok(())
}
