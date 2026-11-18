use polars::prelude::IntoLazy;
#[cfg(not(feature = "polars-ffi"))]
use polars::{
    df,
    prelude::{NamedFrom, col},
};

#[cfg(not(feature = "polars-ffi"))]
use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    measurements::make_private_lazyframe,
    measures::MaxDivergence,
    metrics::SymmetricDistance,
};

use super::*;

#[test]
fn test_take_lazy_exhausts_shared_queryable() -> Fallible<()> {
    let mut onceframe = OnceFrame::from(DataFrame::empty().lazy());
    let mut alias = onceframe.clone();

    let _: LazyFrame = onceframe.eval_internal(&ExtractLazyFrame)?;
    let _: LazyFrame = alias.eval_internal(&TakeLazyFrame)?;

    assert!(
        onceframe
            .eval_internal::<LazyFrame>(&TakeLazyFrame)
            .is_err()
    );
    assert!(alias.eval_internal::<LazyFrame>(&ExtractLazyFrame).is_err());
    Ok(())
}

#[test]
#[cfg(feature = "polars-ffi")]
fn test_rust_collect_unsupported() {
    let onceframe = OnceFrame::from(DataFrame::empty().lazy());
    assert!(
        onceframe
            .collect()
            .unwrap_err()
            .message
            .unwrap()
            .contains("unsupported")
    );
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_dp_quantile() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "A",
        AtomDomain::<f64>::new_non_nan(),
    )])?
    .with_margin(Margin::select().with_max_length(100))?;

    let lf = df!("A" => [50f64; 100])?.lazy();

    let candidates = Series::new(
        "".into(),
        [0., 10., 20., 30., 40., 50., 60., 70., 80., 90., 100.],
    );

    let m_quant = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        MaxDivergence,
        lf.clone().select([col("A").dp().median(candidates, None)]),
        Some(1.),
        None,
    )?;

    let observed = m_quant
        .invoke(&lf)?
        .collect()?
        .column("A")?
        .f64()?
        .get(0)
        .unwrap();

    let expected = 50.;
    assert_eq!(observed, expected);
    Ok(())
}
