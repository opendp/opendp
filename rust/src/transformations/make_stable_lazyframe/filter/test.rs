use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, OptionDomain, SeriesDomain},
    metrics::SymmetricDistance,
    transformations::make_stable_lazyframe,
};

use super::*;

#[test]
fn test_filter() -> Fallible<()> {
    let lf = df!("chunk_2_null" => [Some(1i64), None])?.lazy();

    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "chunk_2_null",
        OptionDomain::new(AtomDomain::<i64>::default()),
    )])?
    .with_margin(Margin::by(["chunk_2_null"]).with_public_keys())?;

    let t_filter = make_stable_lazyframe(
        lf_domain.clone(),
        SymmetricDistance,
        lf.clone().filter(col("chunk_2_null").is_not_null()),
    )?;

    let actual = t_filter.invoke(&lf)?.collect()?;
    assert_eq!(actual, df!("chunk_2_null" => [Some(1)])?);

    assert!(t_filter
        .output_domain
        .margins
        .iter()
        .all(|m| { m.public_info.is_none() }));

    Ok(())
}

#[test]
fn test_filter_fail_with_non_bool_predicate() -> Fallible<()> {
    let lf = df!("chunk_2_null" => [Some(1i64), None])?.lazy();

    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "chunk_2_null",
        OptionDomain::new(AtomDomain::<i64>::default()),
    )])?
    .with_margin(Margin::by(["chunk_2_null"]).with_public_keys())?;

    let variant = make_stable_lazyframe(
        lf_domain.clone(),
        SymmetricDistance,
        lf.clone().filter(col("chunk_2_null").fill_nan(0)),
    )
    .map(|_| ())
    .unwrap_err()
    .variant;

    assert_eq!(variant, ErrorVariant::MakeTransformation);

    Ok(())
}
