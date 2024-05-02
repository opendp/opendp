use super::*;
use polars::prelude::*;

use crate::{
    error::ErrorVariant,
    measurements::PrivateExpr,
    measures::MaxDivergence,
    metrics::{PartitionDistance, SymmetricDistance},
    transformations::test_helper::get_test_data,
};

#[test]
fn test_make_count_expr_grouped() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    // This will succeed because there is a margin for "chunk_2_bool" that indicates that partition lengths are public.
    let expr_domain = lf_domain.aggregate(["chunk_2_bool"]);

    let m_lap = len().make_private(
        expr_domain,
        PartitionDistance(SymmetricDistance),
        MaxDivergence::default(),
        None,
    )?;

    let meas_res = m_lap.invoke(&(lf.logical_plan.clone(), all()))?;

    let df_actual = lf
        .clone()
        .group_by([col("chunk_2_bool")])
        .agg([meas_res])
        .collect()?;
    let df_exact = lf.group_by([col("chunk_2_bool")]).agg([len()]).collect()?;

    assert_eq!(
        df_actual.sort(["chunk_2_bool"], false, false)?,
        df_exact.sort(["chunk_2_bool"], false, false)?
    );
    Ok(())
}

#[test]
fn test_make_count_expr_no_length() -> Fallible<()> {
    let (lf_domain, _) = get_test_data()?;
    // This will fail because there is no margin for "cycle_5_alpha" that indicates that partition lengths are public.
    let expr_domain = lf_domain.aggregate(["cycle_5_alpha"]);

    let variant = len()
        .make_private(
            expr_domain,
            PartitionDistance(SymmetricDistance),
            MaxDivergence::default(),
            None,
        )
        .map(|_| ())
        .unwrap_err()
        .variant;

    assert_eq!(variant, ErrorVariant::MakeMeasurement);
    Ok(())
}
