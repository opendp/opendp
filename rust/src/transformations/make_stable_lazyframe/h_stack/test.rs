use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, OptionDomain, SeriesDomain},
    metrics::SymmetricDistance,
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
    .with_margin(&["chunk_2_null"], Margin::default().with_public_keys())?;

    let t_with_column = make_stable_lazyframe(
        lf_domain.clone(),
        SymmetricDistance,
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

    let series_domain = SeriesDomain::new("f64", AtomDomain::<f64>::new_nullable());
    let lf_domain = LazyFrameDomain::new(vec![series_domain])?;

    let t_with_column = make_stable_lazyframe(
        lf_domain.clone(),
        SymmetricDistance,
        lf.clone().fill_nan(lit(2.)),
    )?;

    let actual = t_with_column.invoke(&lf)?.collect()?;
    let expect = df!("f64" => [1f64, 2f64])?;

    assert_eq!(actual, expect);
    Ok(())
}
