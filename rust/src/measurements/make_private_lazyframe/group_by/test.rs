use crate::domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain};
use crate::error::ErrorVariant::MakeMeasurement;
use crate::error::*;
use crate::measurements::make_private_lazyframe;
use crate::measures::MaxDivergence;
use crate::polars::PrivacyNamespace;
use crate::traits::samplers::test::kolmogorov_smirnov;
use polars::prelude::*;
use statrs::function::erf;

use crate::metrics::SymmetricDistance;

use super::*;

#[test]
fn test_aggregate() -> Fallible<()> {
    let lf_domain = DslPlanDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
        SeriesDomain::new("C", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["A", "C"]).with_public_keys())?;

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

#[test]
fn test_explicit_keys() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<u32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin(Margin::by(["A"]).with_max_partition_length(1))?;

    let lf = df!("A" => &[0u32], "B" => &[0.0f64])?.lazy();
    let keys = df!("A" => &(0u32..1000).collect::<Vec<_>>())?.lazy();

    let meas = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        ZeroConcentratedDivergence,
        lf.clone()
            .group_by(&[col("A")])
            .agg(&[col("B")
                .fill_nan(0.0)
                .fill_null(0.0)
                .dp()
                .sum((0.0, 1.0), None)])
            .join(keys, [col("A")], [col("A")], JoinType::Right.into()),
        Some(1.),
        None,
    )?;

    let release = meas.invoke(&lf)?.collect()?;
    let samples: Vec<f64> = release.column("B")?.f64()?.iter().flatten().collect();
    let samples = <[f64; 1000]>::try_from(samples).unwrap();

    pub fn normal_cdf(x: f64) -> f64 {
        (erf::erf(x / std::f64::consts::SQRT_2) + 1.0) / 2.0
    }

    kolmogorov_smirnov(samples, normal_cdf)?;
    Ok(())
}
