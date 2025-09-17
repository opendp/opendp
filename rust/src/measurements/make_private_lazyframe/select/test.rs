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
fn test_select_no_margin() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?;

    let lf = df!("A" => &[1i32, 2, 2])?.lazy();

    let m_select = make_private_lazyframe(
        lf_domain,
        FrameDistance(SymmetricDistance),
        MaxDivergence,
        lf.clone().select(&[len().dp().noise(Some(0.))]),
        Some(1.),
        None,
    )?;

    let actual = m_select.invoke(&lf)?.collect()?;
    let expect = df!("len" => [3])?;

    assert_eq!(actual, expect);
    Ok(())
}

#[test]
fn test_select() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?
            .with_margin(Margin::select().with_max_length(10))?;

    let lf = df!("A" => &[1i32, 2, 2])?.lazy();

    let m_select = make_private_lazyframe(
        lf_domain,
        FrameDistance(SymmetricDistance),
        MaxDivergence,
        lf.clone().select(&[
            col("A").dp().sum((lit(0), lit(3)), Some(0.)),
            len().dp().noise(Some(0.)),
        ]),
        Some(1.),
        None,
    )?;

    let actual = m_select.invoke(&lf)?.collect()?;
    let expect = df!("A" => [5], "len" => [3])?;

    assert_eq!(actual, expect);
    Ok(())
}

#[test]
fn test_fail_select_invalid_expression() -> Fallible<()> {
    let lf_domain = DslPlanDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?;

    let lf = df!("A" => &[1i32, 2, 2])?.lazy();

    let error_variant_res = make_private_select::<_, _>(
        lf_domain,
        FrameDistance(SymmetricDistance),
        MaxDivergence,
        // this expression cannot be parsed into a measurement
        lf.select(&[col("A").sum()]).logical_plan,
        Some(1.),
    )
    .map(|_| ())
    .unwrap_err()
    .variant;

    assert_eq!(MakeMeasurement, error_variant_res);

    Ok(())
}
