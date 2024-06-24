use crate::domains::{AtomDomain, Margin, SeriesDomain};
use crate::error::ErrorVariant::MakeMeasurement;
use crate::error::*;
use crate::measures::MaxDivergence;
use polars::prelude::*;

use crate::metrics::SymmetricDistance;

use super::*;

#[test]
fn test_aggregate() -> Fallible<()> {
    let lf_domain = DslPlanDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
        SeriesDomain::new("C", AtomDomain::<i32>::default()),
    ])?
    .with_margin(&["A", "C"], Margin::new().with_public_keys())?;

    let lf = df!(
        "A" => &[1i32, 2, 2],
        "B" => &[1.0f64, 2.0, 2.0],
        "C" => &[8i32, 9, 10],)?
    .lazy();

    let error_variant_res = make_private_group_by::<_, SymmetricDistance, _>(
        lf_domain,
        SymmetricDistance,
        MaxDivergence::<f64>::default(),
        lf.group_by(&[col("A"), col("C")])
            .agg(&[col("B").sum()])
            .logical_plan,
        Some(1.),
        None,
    )
    .map(|_| ())
    .unwrap_err()
    .variant;

    assert_eq!(MakeMeasurement, error_variant_res);

    Ok(())
}
