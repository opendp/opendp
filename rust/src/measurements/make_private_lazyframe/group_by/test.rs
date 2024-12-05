use crate::domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain};
use crate::error::ErrorVariant::MakeMeasurement;
use crate::error::*;
use crate::measurements::make_private_lazyframe;
use crate::measures::MaxDivergence;
use crate::polars::PrivacyNamespace;
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
    .with_margin(&["A", "C"], Margin::default().with_public_keys())?;

    let lf = df!(
        "A" => &[1i32, 2, 2],
        "B" => &[1.0f64, 2.0, 2.0],
        "C" => &[8i32, 9, 10],)?
    .lazy();

    let error_variant_res = make_private_group_by::<_, SymmetricDistance, _>(
        lf_domain,
        SymmetricDistance,
        MaxDivergence,
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

#[test]
fn test_stable_keys_puredp() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?;

    let lf = df!("A" => [[1i32; 1000], [2; 1000]].concat())?.lazy();

    let meas = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        Approximate(MaxDivergence),
        lf.clone()
            .group_by(&[col("A")])
            .agg(&[len().dp().noise(None, None)]),
        Some(1.),
        Some(40),
    )?;

    let counts = meas.invoke(&lf)?;
    let params = meas.map(&1)?;

    println!("counts {}", counts.collect()?);
    println!("params {:?}", params);

    Ok(())
}

#[test]
fn test_stable_keys_zCDP() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?;

    let lf = df!("A" => [[1i32; 1000], [2; 1000]].concat())?.lazy();

    let meas = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        Approximate(ZeroConcentratedDivergence),
        lf.clone()
            .group_by(&[col("A")])
            .agg(&[len().dp().noise(None, None)]),
        Some(1.),
        Some(40),
    )?;

    let counts = meas.invoke(&lf)?;
    let params = meas.map(&1)?;

    println!("counts {}", counts.collect()?);
    println!("params {:?}", params);

    Ok(())
}
