use polars::{
    df,
    prelude::{IntoLazy, NamedFrom, col},
};

use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    measurements::make_private_lazyframe,
    measures::MaxDivergence,
    metrics::SymmetricDistance,
};

use super::*;

fn assert_onceframe_exhausted(onceframe: &mut OnceFrame) {
    assert!(onceframe.eval(&OnceFrameQuery::Collect).is_err());
    assert!(
        onceframe
            .eval_internal::<LazyFrame>(&ExtractLazyFrame)
            .is_err()
    );
    assert!(
        onceframe
            .eval_internal::<LazyFrame>(&TakeLazyFrame)
            .is_err()
    );
}

#[test]
fn test_onceframe_take_consumes_shared_state() -> Fallible<()> {
    let mut onceframe = OnceFrame::from(df!("A" => [1, 2, 3])?.lazy());
    let mut alias = onceframe.clone();

    // Ordinary extraction remains non-consuming.
    for _ in 0..2 {
        let extracted: LazyFrame = onceframe.eval_internal(&ExtractLazyFrame)?;
        assert_eq!(extracted.collect()?.height(), 3);
    }

    let taken: LazyFrame = onceframe.eval_internal(&TakeLazyFrame)?;
    assert_onceframe_exhausted(&mut onceframe);
    assert_onceframe_exhausted(&mut alias);
    assert_eq!(taken.collect()?.height(), 3);
    Ok(())
}

#[test]
fn test_onceframe_take_stays_consumed_after_split_failure() -> Fallible<()> {
    // No private boundary: splitting must fail without execution or file creation.
    let mut onceframe = OnceFrame::from(df!("A" => [1, 2, 3])?.lazy());
    let taken: LazyFrame = onceframe.eval_internal(&TakeLazyFrame)?;
    assert!(split::split_plan(taken.logical_plan, std::path::Path::new("unused")).is_err());
    assert_onceframe_exhausted(&mut onceframe);
    Ok(())
}

#[test]
fn test_onceframe_collect_exhausts_take_and_extract() -> Fallible<()> {
    let mut onceframe = OnceFrame::from(df!("A" => [1, 2, 3])?.lazy());
    let OnceFrameAnswer::Collect(frame) = onceframe.eval(&OnceFrameQuery::Collect)?;
    assert_eq!(frame.height(), 3);
    assert_onceframe_exhausted(&mut onceframe);
    Ok(())
}

#[test]
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
