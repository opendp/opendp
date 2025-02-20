use crate::{
    domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain},
    metrics::SymmetricDistance,
    transformations::make_stable_lazyframe,
};

use super::*;

#[test]
fn test_select_microdata() -> Fallible<()> {
    let lf = df!("chunk_2_null" => [Some(1i64), None])?.lazy();

    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "chunk_2_null",
        OptionDomain::new(AtomDomain::<i64>::default()),
    )])?;

    let t_select = make_stable_lazyframe(
        lf_domain.clone(),
        SymmetricDistance,
        lf.clone().select([lit(2).gt(col("chunk_2_null"))]),
    )?;

    let observed = t_select.invoke(&lf)?.collect()?;
    let expected = df!(
        "chunk_2_null" => [Some(1), None],
    )?;
    assert_eq!(observed, expected);
    assert!(t_select.output_domain.margins.is_empty());

    Ok(())
}
