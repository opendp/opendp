use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, OptionDomain, SeriesDomain},
    metrics::{Binding, FrameDistance, SymmetricDistance, SymmetricIdDistance},
    transformations::make_stable_lazyframe,
};

use super::*;

#[test]
fn test_with_column() -> Fallible<()> {
    let lf = df!("chunk_2_null" => [Some(1i64), None])?.lazy();

    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "chunk_2_null",
        OptionDomain::new(AtomDomain::<i64>::default()),
    )])?
    .with_margin(Margin::by(["chunk_2_null"]).with_invariant_keys())?;

    let t_with_column = make_stable_lazyframe(
        lf_domain.clone(),
        FrameDistance(SymmetricDistance),
        lf.clone().with_column(lit(2).gt(col("chunk_2_null"))),
    )?;

    let actual = t_with_column.invoke(&lf)?.collect()?;
    let expect = df!(
        "chunk_2_null" => [Some(1), None],
        "literal" => [Some(true), None]
    )?;
    assert_eq!(actual, expect);
    // while margins get filtered out, chunk_2_null should still be present
    assert_eq!(
        t_with_column.output_domain.margins,
        t_with_column.input_domain.margins
    );

    Ok(())
}

// LazyFrame::fill_nan works for "free", because it breaks down to pre-existing primitives:
//    hstack with expressions that use fill_nan and alias
#[test]
// TODO: ignored because Polars 1.0.0 changes behavior of fill_nan to de-sugar into .with_columns(all().fill_nan(v)),
// and we don't support all() at this time. See https://github.com/opendp/opendp/issues/1772
#[ignore]
fn test_fill_nan() -> Fallible<()> {
    let lf = df!("f64" => [1f64, f64::NAN])?.lazy();

    let series_domain = SeriesDomain::new("f64", AtomDomain::<f64>::default());
    let lf_domain = LazyFrameDomain::new(vec![series_domain])?;

    let t_with_column = make_stable_lazyframe(
        lf_domain.clone(),
        FrameDistance(SymmetricDistance),
        lf.clone().fill_nan(lit(2.)),
    )?;

    let actual = t_with_column.invoke(&lf)?.collect()?;
    let expect = df!("f64" => [1f64, 2f64])?;

    assert_eq!(actual, expect);
    Ok(())
}

#[test]
fn test_with_column_rejects_multisite_identifier_modification() -> Fallible<()> {
    let lf = df!("user_id" => [1i32, 2], "value" => [3i32, 4])?.lazy();

    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;

    let metric = SymmetricIdDistance {
        protect: "user".to_string(),
        bindings: vec![Binding {
            space: "user".to_string(),
            exprs: vec![col("user_id")],
        }],
    };

    let err = make_stable_lazyframe::<_, FrameDistance<SymmetricIdDistance>>(
        lf_domain,
        FrameDistance(metric),
        lf.with_column((col("user_id") + lit(1)).alias("user_id")),
    )
    .unwrap_err();

    assert!(format!("{err:?}").contains("identifiers"));
    Ok(())
}
