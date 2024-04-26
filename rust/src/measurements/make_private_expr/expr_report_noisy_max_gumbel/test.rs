use super::*;
use polars::prelude::*;

use crate::{
    core::PrivacyNamespaceHelper,
    measurements::make_private_expr,
    metrics::{PartitionDistance, SymmetricDistance},
    transformations::expr_discrete_quantile_score::test::get_quantile_test_data,
};

#[test]
fn test_rnm_gumbel_expr() -> Fallible<()> {
    let (lf_domain, lf) = get_quantile_test_data()?;
    let expr_domain = lf_domain.select();
    let scale: f64 = 1e-8;
    let candidates = vec![0., 10., 20., 30., 40., 50., 60., 70., 80., 90., 100.];

    let m_quant = make_private_expr(
        expr_domain,
        PartitionDistance(SymmetricDistance),
        MaxDivergence::default(),
        col("cycle_(..101f64)").dp().median(candidates, Some(scale)),
        None,
    )?;

    let dp_expr = m_quant.invoke(&(lf.logical_plan.clone(), all()))?;
    let df = lf.select([dp_expr]).collect()?;
    let actual = df.column("cycle_(..101f64)")?.u32()?.get(0).unwrap();
    assert_eq!(actual, 5);

    Ok(())
}
